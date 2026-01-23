use log::info;
use rppal::gpio::Gpio;
use std::{thread, time::Duration};
use tauri::Emitter;

pub fn run_buttons(app: tauri::AppHandle) {
    let gpio = Gpio::new().unwrap();
    let mut button1 = gpio.get(2).unwrap().into_input_pullup();
    let mut button2 = gpio.get(3).unwrap().into_input_pullup();

    {
        let app = app.clone();
        button1
            .set_async_interrupt(
                rppal::gpio::Trigger::FallingEdge,
                Some(Duration::from_millis(100)),
                move |_| {
                    app.emit::<i32>("button", 1);
                    info!("Button1 pressed");
                },
            )
            .unwrap();
    }
    button2
        .set_async_interrupt(
            rppal::gpio::Trigger::FallingEdge,
            Some(Duration::from_millis(100)),
            move |_| {
                app.emit::<i32>("button", 2);
                info!("Button2 pressed");
            },
        )
        .unwrap();
    loop {
        thread::sleep(Duration::from_secs(1));
    }
}
