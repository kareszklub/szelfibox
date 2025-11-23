use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use gstreamer as gst;
use gstreamer::prelude::*;
use gstreamer_app as gst_app;

use tauri::Emitter;

pub fn start_camera(app: tauri::AppHandle) {
    gst::init().unwrap();

    let pipeline = gstreamer::parse::launch(
        "v4l2src device=/dev/video0 ! videoconvert ! video/x-raw,format=RGBA ! appsink name=sink",
    )
    .unwrap();
    let pipeline = pipeline.downcast::<gst::Pipeline>().unwrap();

    let appsink = pipeline
        .by_name("sink")
        .unwrap()
        .downcast::<gst_app::AppSink>()
        .unwrap();

    appsink.set_callbacks(
        gst_app::AppSinkCallbacks::builder()
            .new_sample(move |sink| {
                let sample = sink.pull_sample().unwrap();
                let buffer = sample.buffer().unwrap();
                let map = buffer.map_readable().unwrap();
                let bytes = map.as_slice();
                let encoded = STANDARD.encode(bytes);
                app.emit("frame", encoded).unwrap();
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
