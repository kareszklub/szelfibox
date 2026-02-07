use base64::prelude::BASE64_URL_SAFE_NO_PAD;
use base64::Engine;
use gstreamer as gst;
use gstreamer::prelude::*;
use gstreamer_app as gst_app;
use image::{ImageBuffer, Luma, Rgba};
use log::error;
use qrcode::QrCode;
use std::io::Cursor;
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;
use xxhash_rust::xxh3::xxh3_128;

use tauri::{ipc::Response, Emitter, Manager, State};

use crate::img_utils::get_newest_file;
use crate::{
    phone_utils, CameraState, FPS, HEIGHT, PREVIEW_HEIGHT, PREVIEW_WIDTH, VIDEO_DEVICE, WIDTH,
};

static CROP: u32 = 300;

pub fn start_camera(app: tauri::AppHandle) {
    gst::init().unwrap();

    // Start scrcpy on a separate thread
    std::thread::spawn(|| {
        let status = Command::new("scrcpy")
            .arg("--video-source=camera")
            .arg(&format!("--v4l2-sink={}", VIDEO_DEVICE))
            .arg(&format!("--camera-size={}x{}", WIDTH, HEIGHT))
            .arg(&format!("--camera-fps={}", FPS))
            .arg("--no-window")
            .arg("--no-audio")
            .status()
            .unwrap();

        if !status.success() {
            error!("scrcpy instance failed with: {}", status);
        }
    });

    // To let the scrcpy instance start
    std::thread::sleep(Duration::from_secs(3));

    let pipeline = gstreamer::parse::launch(&format!(
        "v4l2src device={} ! tee name=t \
         t. ! queue ! videoconvert ! video/x-raw,format=RGBA,width={},height={} ! appsink name=main_sink \
         t. ! queue ! videoconvert ! videoscale ! video/x-raw,format=RGBA,width={},height={} ! appsink name=preview_sink",
        VIDEO_DEVICE, WIDTH, HEIGHT, PREVIEW_WIDTH, PREVIEW_HEIGHT
    ))
    .unwrap();

    let pipeline = pipeline.downcast::<gst::Pipeline>().unwrap();

    let main_sink = pipeline
        .by_name("main_sink")
        .expect("Could not find main_sink")
        .downcast::<gst_app::AppSink>()
        .unwrap();

    let latest_preview_frame = app.state::<CameraState>().latest_preview_frame.clone();
    let latest_hd_frame = app.state::<CameraState>().latest_hd_frame.clone();
    let app_handle_clone = app.clone();

    main_sink.set_callbacks(
        gst_app::AppSinkCallbacks::builder()
            .new_sample(move |sink| {
                let sample = sink.pull_sample().map_err(|_| gst::FlowError::Eos)?;
                let buffer = sample.buffer().ok_or(gst::FlowError::Error)?;
                let map = buffer.map_readable().map_err(|_| gst::FlowError::Error)?;
                let bytes = map.to_vec();

                {
                    let mut lock = latest_hd_frame.lock().unwrap();
                    *lock = Some(bytes);
                }

                app_handle_clone.emit("new-frame-ready", ()).unwrap();

                Ok(gst::FlowSuccess::Ok)
            })
            .build(),
    );

    let preview_sink = pipeline
        .by_name("preview_sink")
        .expect("Could not find preview_sink")
        .downcast::<gst_app::AppSink>()
        .unwrap();

    preview_sink.set_callbacks(
        gst_app::AppSinkCallbacks::builder()
            .new_sample(move |sink| {
                let sample = sink.pull_sample().map_err(|_| gst::FlowError::Eos)?;
                let buffer = sample.buffer().ok_or(gst::FlowError::Error)?;
                let map = buffer.map_readable().map_err(|_| gst::FlowError::Error)?;
                let bytes = map.to_vec();

                {
                    let mut lock = latest_preview_frame.lock().unwrap();
                    *lock = Some(bytes);
                }

                Ok(gst::FlowSuccess::Ok)
            })
            .build(),
    );

    pipeline.set_state(gst::State::Playing).unwrap();

    let bus = pipeline.bus().unwrap();
    for msg in bus.iter_timed(gst::ClockTime::NONE) {
        use gst::MessageView;
        match msg.view() {
            MessageView::Eos(..) => break,
            MessageView::Error(err) => {
                eprintln!(
                    "Error from {:?}: {} ({:?})",
                    err.src().map(|s| s.path_string()),
                    err.error(),
                    err.debug()
                );
                break;
            }
            _ => (),
        }
    }

    pipeline.set_state(gst::State::Null).unwrap();
}

#[tauri::command]
pub async fn fetch_frame(state: State<'_, CameraState>) -> Result<Response, String> {
    let frame_lock = state.latest_preview_frame.lock().unwrap();

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
    let hd_data = {
        let mut frame_lock = state.latest_hd_frame.lock().unwrap();
        frame_lock.take().ok_or("No frame available")?
    };
    let preview_data = {
        let frame_lock = state.latest_preview_frame.lock().unwrap();
        frame_lock.as_ref().cloned().ok_or("No frame available")?
    };

    tauri::async_runtime::spawn_blocking(move || {
        let hash128 = xxh3_128(&preview_data);
        let mut hash = BASE64_URL_SAFE_NO_PAD.encode(hash128.to_be_bytes());
        hash.truncate(32);

        let preview_img =
            ImageBuffer::<Rgba<u8>, _>::from_raw(PREVIEW_WIDTH, PREVIEW_HEIGHT, preview_data)
                .expect("invalid RGBA buffer");

        let mut img_png_bytes = Vec::new();
        let encoder = image::codecs::png::PngEncoder::new_with_quality(
            &mut img_png_bytes,
            image::codecs::png::CompressionType::Fast,
            image::codecs::png::FilterType::NoFilter,
        );
        preview_img
            .write_with_encoder(encoder)
            .expect("Failed to encode PNG");

        let hash_clone = hash.clone();
        std::thread::spawn(move || {
            let mut img = ImageBuffer::<Rgba<u8>, _>::from_raw(WIDTH, HEIGHT, hd_data)
                .expect("invalid RGBA buffer");

            let mut img =
                image::imageops::crop(&mut img, CROP, 0, WIDTH - 2 * CROP, HEIGHT).to_image();

            let overlay = image::open("../static/overlay.png").unwrap().to_rgba8();

            image::imageops::overlay(&mut img, &overlay, 0, 0);

            let path = format!("../static/images/{}.png", hash_clone);
            img.save_with_format(path, image::ImageFormat::Png).ok();
        });

        let qr_text = format!("https://box.kende.dev/{}", hash);
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

#[tauri::command]
pub fn print_picture() {
    std::thread::spawn(|| {
        let phone_picture_dir = phone_utils::get_phone_picture_dir().unwrap();
        let picture_dir = PathBuf::from("../static/images");

        let picture_dir = get_newest_file(&picture_dir).unwrap();

        phone_utils::print_pic(&picture_dir, &phone_picture_dir);
    });
}
