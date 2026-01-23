use log::info;
use rppal::gpio::Gpio;
use std::{thread, time::Duration};

pub fn run_buttons() {
    let gpio = Gpio::new().unwrap();
    let pin = gpio.get(3).unwrap().into_input_pullup();
    loop {
        info!("Pin is_low: {}", pin.is_low());
        thread::sleep(Duration::from_secs(1));
    }
}
