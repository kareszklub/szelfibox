use gstreamer::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize GStreamer
    gstreamer::init()?;

    // Use the parse::launch API to build the pipeline from a gst-launch-style string
    let pipeline =
        gstreamer::parse::launch("v4l2src device=/dev/video4 ! videoconvert ! autovideosink")?;

    // Downcast to Pipeline so we can manipulate its state
    let pipeline = pipeline
        .downcast::<gstreamer::Pipeline>()
        .expect("Element is not a Pipeline");

    // Start playing
    pipeline.set_state(gstreamer::State::Playing)?;

    // Listen for messages on the bus
    let bus = pipeline.bus().expect("Pipeline without bus");
    for msg in bus.iter_timed(gstreamer::ClockTime::NONE) {
        use gstreamer::MessageView;
        match msg.view() {
            MessageView::Eos(..) => {
                // End of stream: exit loop
                break;
            }
            MessageView::Error(err) => {
                // Handle error
                eprintln!(
                    "Error from {:?}: {} ({:?})",
                    err.src().map(|s| s.path_string()),
                    err.error(),
                    err.debug()
                );
                break;
            }
            _ => {}
        }
    }

    // Clean up
    pipeline.set_state(gstreamer::State::Null)?;

    Ok(())
}
