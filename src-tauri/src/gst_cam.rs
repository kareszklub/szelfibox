use base64::prelude::BASE64_URL_SAFE_NO_PAD;
use base64::Engine;
use gstreamer as gst;
use gstreamer::prelude::*;
use gstreamer_app as gst_app;
use image::{ImageBuffer, Luma, Rgba};
use qrcode::QrCode;
use std::io::Cursor;
use xxhash_rust::xxh3::xxh3_128;

use tauri::{ipc::Response, Emitter, Manager, State};

use crate::img_utils::append_image_header;
use crate::{CameraState, HEIGHT, VIDEO_DEVICE, WIDTH};

pub fn start_camera(app: tauri::AppHandle) {
    gst::init().unwrap();

    let pipeline = gstreamer::parse::launch(&format!(
        "v4l2src device={} \
     ! videoconvert \
     ! video/x-raw,format=RGBA,width={},height={} \
     ! appsink name=sink",
        VIDEO_DEVICE, WIDTH, HEIGHT
    ))
    .unwrap();

    let pipeline = pipeline.downcast::<gst::Pipeline>().unwrap();

    let appsink = pipeline
        .by_name("sink")
        .unwrap()
        .downcast::<gst_app::AppSink>()
        .unwrap();

    let latest_frame = app.state::<CameraState>().latest_frame.clone();

    appsink.set_callbacks(
        gst_app::AppSinkCallbacks::builder()
            .new_sample(move |sink| {
                let sample = sink.pull_sample().unwrap();
                let buffer = sample.buffer().unwrap();
                let map = buffer.map_readable().unwrap();
                let bytes = map.to_vec();

                {
                    let mut lock = latest_frame.lock().unwrap();
                    *lock = Some(bytes);
                }

                app.emit("new-frame-ready", ()).unwrap();

                Ok(gst::FlowSuccess::Ok)
            })
            .build(),
    );

    pipeline.set_state(gst::State::Playing).unwrap();

    let bus = pipeline.bus().unwrap();
    for msg in bus.iter_timed(gst::ClockTime::NONE) {
        if let gst::MessageView::Eos(_) = msg.view() {
            break;
        }
        if let gst::MessageView::Error(_) = msg.view() {
            break;
        }
    }

    pipeline.set_state(gst::State::Null).unwrap();
}

#[tauri::command]
pub async fn fetch_frame(state: State<'_, CameraState>) -> Result<Response, String> {
    let frame_lock = state.latest_frame.lock().unwrap();

    if let Some(frame) = frame_lock.as_ref().cloned() {
        Ok(Response::new(frame))
    } else {
        Err("No frame available".to_string())
    }
}

fn create_payload(image_bytes: Vec<u8>, qr_bytes: Vec<u8>) -> Response {
    let mut combined = Vec::with_capacity(4 + image_bytes.len() + qr_bytes.len());

    combined.extend_from_slice(&((image_bytes.len() as u32).to_le_bytes()));

    combined.extend_from_slice(&image_bytes);
    combined.extend_from_slice(&qr_bytes);

    Response::new(combined)
}

#[tauri::command]
pub async fn take_picture(state: State<'_, CameraState>) -> Result<Response, String> {
    let data = {
        let mut frame_lock = state.latest_frame.lock().unwrap();
        frame_lock.take().ok_or("No frame available")?
    };

    // 1. Move heavy processing to a blocking thread to keep the executor free
    tauri::async_runtime::spawn_blocking(move || {
        let hash128 = xxh3_128(&data);
        let mut hash = BASE64_URL_SAFE_NO_PAD.encode(hash128.to_be_bytes());
        hash.truncate(32);

        let img =
            ImageBuffer::<Rgba<u8>, _>::from_raw(WIDTH, HEIGHT, data).expect("invalid RGBA buffer");

        // 2. Use Fast Compression for the Preview
        let mut img_png_bytes = Vec::new();
        let encoder = image::codecs::png::PngEncoder::new_with_quality(
            &mut img_png_bytes,
            image::codecs::png::CompressionType::Fast, // This is the speed boost!
            image::codecs::png::FilterType::NoFilter,
        );
        img.write_with_encoder(encoder)
            .expect("Failed to encode PNG");

        // 3. Save to disk in background (Don't await this for the response)
        let img_clone = img.clone();
        let hash_clone = hash.clone();
        std::thread::spawn(move || {
            let path = format!("../static/images/{}.png", hash_clone);
            img_clone
                .save_with_format(path, image::ImageFormat::Png)
                .ok();
        });

        // 4. QR Generation
        let qr_text = format!("http://0.0.0.0:8000/{}", hash);
        let qr = QrCode::new(qr_text.as_bytes()).map_err(|e| e.to_string())?;
        let qr_image: ImageBuffer<Luma<u8>, Vec<u8>> =
            qr.render::<Luma<u8>>().min_dimensions(256, 256).build();

        let mut qr_png_bytes = Vec::new();
        qr_image
            .write_to(&mut Cursor::new(&mut qr_png_bytes), image::ImageFormat::Png)
            .ok();

        Ok(create_payload(img_png_bytes, qr_png_bytes))
    })
    .await
    .map_err(|e| e.to_string())?
}
