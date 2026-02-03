use std::{process::Command, thread, time};

fn buttonpress(x: &str, y: &str) {
    Command::new("adb")
        .arg("shell")
        .arg("input")
        .arg("tap")
        .arg(x)
        .arg(y)
        .spawn()
        .expect("failed to press on screen")
        .wait();
}

fn send_picture(img: &str) {}

fn rm_picture(img: &String) {}

pub fn print_pic(img: &str) {
    //send_picture(img);

    for _ in 0..2 {
        buttonpress("160", "1300");
        println!("buttonpress 1");
        thread::sleep(time::Duration::from_millis(250));
    }
    // doesnt finalize the printing for now, 2 more presses for that, new coordinates needed to
    // return to base state
    for _ in 0..1 {
        buttonpress("920", "2250");
        println!("buttonpress 2");
        thread::sleep(time::Duration::from_millis(750));
    }
}

