use glob;
use std::{fs::create_dir_all, path::PathBuf, process::Command, thread, time};

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

pub fn get_phone_picture_dir() -> PathBuf {
    let mut paths = glob::glob("/run/user/1000/gvfs/*/*").expect("No phone found! Is it mounted?");
    let phone_path = paths
        .next()
        .expect("No phone found!")
        .expect("Failed to read phone path");
    println!("phone found at path: {:?}", phone_path.display());
    create_dir_all(phone_path.clone().join("Pictures/szelfibox"))
        .expect("failed to create static directory");
    phone_path.clone().join("Pictures/szelfibox")
}

pub fn send_picture(img: &PathBuf) {}

pub fn print_pic(img: &PathBuf) {
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
