use base64::prelude::BASE64_URL_SAFE_NO_PAD;
use base64::Engine;
use gstreamer as gst;
use gstreamer::prelude::*;
use gstreamer_app as gst_app;
use image::{ImageBuffer, Luma, Rgba};
use log::{error, info};
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
        let mut command = Command::new("scrcpy");
        let command = command
            .arg("--video-source=camera")
            .arg(&format!("--v4l2-sink={}", VIDEO_DEVICE))
            .arg(&format!("--camera-size={}x{}", WIDTH, HEIGHT))
            .arg(&format!("--camera-fps={}", FPS))
            .arg("--no-window")
            .arg("--no-audio");

        loop {
            let status = command.status().unwrap();
            if !status.success() {
                error!("scrcpy instance failed with: {}", status);
                std::thread::sleep(Duration::from_secs(3));
                info!("Restarting scrcpy instance...");
            }
        }
    });

    // To let the scrcpy instance start
    std::thread::sleep(Duration::from_secs(5));

    let pipeline_str = format!(
        "v4l2src device={} ! tee name=t \
         t. ! queue max-size-buffers=1 leaky=downstream \
            ! valve name=snapshot_valve drop=true \
            ! videoconvert ! video/x-raw,format=RGBA,width={},height={} \
            ! appsink name=main_sink async=false sync=false max-buffers=1 drop=true \
         t. ! queue \
            ! videoconvert ! videoscale ! video/x-raw,format=RGBA,width={},height={} \
            ! appsink name=preview_sink async=false sync=false max-buffers=1 drop=true",
        VIDEO_DEVICE, WIDTH, HEIGHT, PREVIEW_WIDTH, PREVIEW_HEIGHT
    );

    let pipeline = gstreamer::parse::launch(&pipeline_str).unwrap();
    let pipeline = pipeline.downcast::<gst::Pipeline>().unwrap();

    let main_sink = pipeline
        .by_name("main_sink")
        .expect("Could not find main_sink")
        .downcast::<gst_app::AppSink>()
        .unwrap();

    let valve = pipeline
        .by_name("snapshot_valve")
        .expect("Could not find snapshot_valve");

    let state = app.state::<CameraState>();
    *state.main_sink.lock().unwrap() = Some(main_sink);
    *state.snapshot_valve.lock().unwrap() = Some(valve);

    let preview_sink = pipeline
        .by_name("preview_sink")
        .expect("Could not find preview_sink")
        .downcast::<gst_app::AppSink>()
        .unwrap();

    let latest_preview_frame = state.latest_preview_frame.clone();
    let app_handle_clone = app.clone();

    preview_sink.set_callbacks(
        gst_app::AppSinkCallbacks::builder()
            .new_sample(move |sink| {
                let sample = sink.pull_sample().map_err(|_| gst::FlowError::Eos)?;
                let buffer = sample.buffer().ok_or(gst::FlowError::Error)?;
                let map = buffer.map_readable().map_err(|_| gst::FlowError::Error)?;

                *latest_preview_frame.lock().unwrap() = Some(map.to_vec());

                app_handle_clone.emit("new-frame-ready", ()).unwrap();
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
    let (valve, main_sink) = {
        let v = state.snapshot_valve.lock().unwrap().clone();
        let s = state.main_sink.lock().unwrap().clone();
        if v.is_none() || s.is_none() {
            return Err("Camera not started".to_string());
        }
        (v.unwrap(), s.unwrap())
    };

    let preview_data = {
        let frame_lock = state.latest_preview_frame.lock().unwrap();
        frame_lock
            .as_ref()
            .cloned()
            .ok_or("No preview frame available")?
    };

    tauri::async_runtime::spawn_blocking(move || {
        valve.set_property("drop", false);

        let sample = main_sink
            .pull_sample()
            .map_err(|_| "Failed to pull sample")?;

        valve.set_property("drop", true);

        let buffer = sample.buffer().ok_or("No buffer in sample")?;
        let map = buffer.map_readable().map_err(|_| "Buffer not readable")?;
        let hd_data = map.to_vec();

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

        // Save high-res image
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
        let phone_picture_dir = phone_utils::get_phone_picture_dir();
        let picture_dir = PathBuf::from("../static/images");

        let picture_dir = get_newest_file(&picture_dir).unwrap();

        phone_utils::print_pic(&picture_dir, &phone_picture_dir);
    });
}
