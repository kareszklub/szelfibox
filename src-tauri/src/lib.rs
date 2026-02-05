use std::path::PathBuf;
use std::{fs, io::Cursor, path::Path};

use crate::gst_cam::fetch_frame;
use crate::img_utils::append_image_header;
use crate::{axum::run_server, buttons::run_buttons};
use std::sync::{Arc, Mutex};
use tauri::{ipc::Response, State};

use base64::{prelude::BASE64_URL_SAFE_NO_PAD, Engine};
use image::{ImageBuffer, Luma, Rgba};
use qrcode::QrCode;
use xxhash_rust::xxh3::xxh3_128;

mod axum;
mod buttons;
mod gst_cam;
mod img_utils;
mod phone_utils;
mod scrcpy;

#[derive(Default)]
pub struct CameraState {
    pub latest_frame: Arc<Mutex<Option<Vec<u8>>>>,
}

#[tauri::command]
fn process_image(width: u32, height: u32, data: Vec<u8>) -> Vec<u8> {
    let hash128 = xxh3_128(&data);
    let mut hash = BASE64_URL_SAFE_NO_PAD.encode(hash128.to_be_bytes());
    hash.truncate(32);

    let img =
        ImageBuffer::<Rgba<u8>, _>::from_raw(width, height, data).expect("invalid RGBA buffer");
    let img = append_image_header(img);

    let path = format!("../static/images/{}.png", hash);
    img.save_with_format(path.clone(), image::ImageFormat::Png)
        .expect("failed to save PNG");

    let qr_text = format!("http://127.0.0.1:8000/{}", hash);
    let qr = QrCode::new(qr_text.as_bytes()).expect("failed to generate QR");

    let qr_image: ImageBuffer<Luma<u8>, Vec<u8>> =
        qr.render::<Luma<u8>>().min_dimensions(256, 256).build();

    let mut png_bytes = Vec::new();
    qr_image
        .write_to(&mut Cursor::new(&mut png_bytes), image::ImageFormat::Png)
        .expect("failed to encode QR PNG");

    png_bytes
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    simple_logger::init_with_level(log::Level::Info).unwrap();

    fs::create_dir_all(&Path::new("../static/images")).expect("failed to create static directory");

    let phone_picture_dir = phone_utils::get_phone_picture_dir();
    let pi_picture_dir = PathBuf::from("../static/images");

    println!(
        "Pictures will be stored at: {:?}",
        phone_picture_dir.display()
    );

    let test_pic = pi_picture_dir.join("test.jpg");

    // phone_utils::print_pic(&test_pic, &phone_picture_dir);

    tauri::async_runtime::spawn(async {
        run_server().await;
    });

    // tauri::async_runtime::spawn(async {
    //     run_camera().await;
    // });

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(CameraState::default())
        .setup(|app| {
            {
                let app_handle = app.handle().clone();
                std::thread::spawn(move || {
                    gst_cam::start_camera(app_handle);
                });
            }
            {
                let app_handle = app.handle().clone();
                std::thread::spawn(|| {
                    run_buttons(app_handle);
                });
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![process_image, fetch_frame])
        .run(tauri::generate_context!())
        .expect("Error while running tauri application");
}
