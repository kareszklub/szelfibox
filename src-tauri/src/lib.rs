use std::{fs, io::Cursor, path::Path};

use crate::{axum::run_server, buttons::run_buttons};

use base64::{prelude::BASE64_URL_SAFE_NO_PAD, Engine};
use image::{ImageBuffer, Luma, Rgba};
use qrcode::QrCode;
use xxhash_rust::xxh3::xxh3_128;

mod axum;
mod buttons;
mod gst_cam;

#[tauri::command]
fn process_image(width: u32, height: u32, data: Vec<u8>) -> Vec<u8> {
    let hash128 = xxh3_128(&data);
    let mut hash = BASE64_URL_SAFE_NO_PAD.encode(hash128.to_be_bytes());
    hash.truncate(32);

    let img =
        ImageBuffer::<Rgba<u8>, _>::from_raw(width, height, data).expect("invalid RGBA buffer");

    let path = format!("../static/images/{}.png", hash);
    img.save_with_format(path, image::ImageFormat::Png)
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

    tauri::async_runtime::spawn(async {
        run_server().await;
    });

    tauri::async_runtime::spawn(async {
        run_buttons();
    });

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_handle = app.handle().clone();

            std::thread::spawn(move || {
                gst_cam::start_camera(app_handle);
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![process_image])
        .run(tauri::generate_context!())
        .expect("Error while running tauri application");
}
