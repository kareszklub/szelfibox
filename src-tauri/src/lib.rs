use std::path::PathBuf;
use std::{fs, path::Path};

use crate::gst_cam::{fetch_frame, take_picture};
use crate::{axum::run_server, buttons::run_buttons};
use std::sync::{Arc, Mutex};

mod axum;
mod buttons;
mod gst_cam;
mod img_utils;
mod phone_utils;

pub static WIDTH: u32 = 1920;
pub static HEIGHT: u32 = 1440;
pub static VIDEO_DEVICE: &str = "/dev/video4";

#[derive(Default)]
pub struct CameraState {
    pub latest_frame: Arc<Mutex<Option<Vec<u8>>>>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    simple_logger::init_with_level(log::Level::Info).unwrap();

    fs::create_dir_all(&Path::new("../static/images")).expect("failed to create static directory");

    let phone_picture_dir = phone_utils::get_phone_picture_dir();
    if let Some(phone_picture_dir) = phone_picture_dir {
        let pi_picture_dir = PathBuf::from("../static/images");

        println!(
            "Pictures will be stored at: {:?}",
            phone_picture_dir.display()
        );

        let test_pic = pi_picture_dir.join("test.jpg");

        // phone_utils::print_pic(&test_pic, &phone_picture_dir);
    }

    tauri::async_runtime::spawn(async {
        run_server().await;
    });

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
        .invoke_handler(tauri::generate_handler![take_picture, fetch_frame])
        .run(tauri::generate_context!())
        .expect("Error while running tauri application");
}
