use rppal::gpio::{Gpio, Trigger};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;
use std::time::Duration;

fn test() {
    println!("Button was pressed: test func");
}

pub fn look_for_buttonpress() {
    let gpio = Gpio::new().unwrap();
    let mut button1 = gpio.get(17).unwrap().into_input_pullup();
    let mut button2 = gpio.get(27).unwrap().into_input_pullup();

    let pressed = Arc::new(AtomicBool::new(false));
    let pressed_cb = pressed.clone();

    // Button 1 interrupt
    button1
        .set_async_interrupt(
            Trigger::FallingEdge,
            Some(Duration::from_millis(50)), // hardware debounce
            move |_| {
                // ISR-style: signal only
                pressed_cb.store(true, Ordering::Relaxed);
            },
        )
        .unwrap();

    // Main loop running in this thread
    loop {
        if pressed.swap(false, Ordering::Relaxed) {
            test();
        }

        thread::sleep(Duration::from_millis(10));
    }
}
