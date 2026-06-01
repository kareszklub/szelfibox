use std::{fs, path::Path};

use dotenv_codegen::dotenv;

use crate::gst_cam::{fetch_frame, print_picture, take_picture};
use crate::pico_buttons::run_pico_buttons;
use crate::{axum::run_server, buttons::run_buttons};
use std::sync::{Arc, Mutex};

mod axum;
mod buttons;
mod gst_cam;
mod img_utils;
mod pico_buttons;

pub static PREVIEW_WIDTH: u32 = 720;
pub static PREVIEW_HEIGHT: u32 = 480;
pub static WIDTH: u32 = 3840;
pub static HEIGHT: u32 = 2160;
pub static VIDEO_DEVICE: &str = dotenv!("VIDEO_DEVICE");
pub static FPS: &str = dotenv!("FPS");

#[derive(Default)]
pub struct CameraState {
    pub latest_preview_frame: Arc<Mutex<Option<Vec<u8>>>>,

    pub main_sink: Arc<Mutex<Option<gstreamer_app::AppSink>>>,
    pub snapshot_valve: Arc<Mutex<Option<gstreamer::Element>>>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    simple_logger::init_with_level(log::Level::Info).unwrap();

    fs::create_dir_all(&Path::new("../static/images")).expect("failed to create static directory");

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
                    run_pico_buttons(app_handle);
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
        .invoke_handler(tauri::generate_handler![
            take_picture,
            fetch_frame,
            print_picture
        ])
        .run(tauri::generate_context!())
        .expect("Error while running tauri application");
}
