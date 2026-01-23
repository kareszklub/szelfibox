use log::info;
use rppal::gpio::Gpio;
use std::{thread, time::Duration};

pub fn run_buttons() {
    let gpio = Gpio::new().unwrap();
    let mut button1 = gpio.get(2).unwrap().into_input_pullup();
    let mut button2 = gpio.get(3).unwrap().into_input_pullup();
    button1
        .set_async_interrupt(
            rppal::gpio::Trigger::FallingEdge,
            Some(Duration::from_millis(100)),
            |_| {
                info!("Button1 pressed");
            },
        )
        .unwrap();
    button2
        .set_async_interrupt(
            rppal::gpio::Trigger::FallingEdge,
            Some(Duration::from_millis(100)),
            |_| {
                info!("Button2 pressed");
            },
        )
        .unwrap();
    loop {
        thread::sleep(Duration::from_secs(1));
    }
}
