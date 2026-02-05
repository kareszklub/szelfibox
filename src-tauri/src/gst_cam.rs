use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use gstreamer as gst;
use gstreamer::prelude::*;
use gstreamer_app as gst_app;
use std::sync::atomic::{AtomicBool, Ordering};

use tauri::{ipc::Response, Emitter, Manager, State};

use crate::CameraState;

pub fn start_camera(app: tauri::AppHandle) {
    gst::init().unwrap();

    let pipeline = gstreamer::parse::launch(
        "v4l2src device=/dev/video4 \
     ! videoconvert \
     ! video/x-raw,format=RGBA,width=1920,height=1440 \
     ! appsink name=sink",
    )
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
                // let encoded = STANDARD.encode(bytes);

                // let app = app.clone();
                // tauri::async_runtime::spawn(async move {
                //     let _ = app.emit("frame", encoded);
                // });

                {
                    let mut lock = latest_frame.lock().unwrap();
                    *lock = Some(bytes);
                }

                // 3. Emit a lightweight signal to the frontend
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
    let mut frame_lock = state.latest_frame.lock().unwrap();

    if let Some(frame) = frame_lock.take() {
        Ok(Response::new(frame))
    } else {
        Err("No frame available".to_string())
    }
}
