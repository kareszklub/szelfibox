use glob;
use log::error;
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

pub fn get_phone_picture_dir() -> Option<PathBuf> {
    let mut paths = glob::glob("/run/user/1000/gvfs/*/*").expect("No phone found! Is it mounted?");

    let phone_path = match paths.next() {
        Some(paths) => paths,
        None => {
            error!("No phone path found. Printing will not be available.");
            return None;
        }
    }
    .expect("Failed to read phone path");
    println!("phone found at path: {:?}", phone_path.display());
    create_dir_all(phone_path.clone().join("Pictures/szelfibox"))
        .expect("failed to create static directory");
    Some(phone_path.join("Pictures/szelfibox"))
}

fn send_picture(img: &PathBuf, to: &PathBuf) {
    println!(
        "sending with: cp '{}' '{}'",
        img.to_str().unwrap(),
        to.to_str().unwrap()
    );

    Command::new("cp")
        .arg(img.to_str().unwrap())
        .arg(to.to_str().unwrap())
        .spawn()
        .unwrap()
        .wait();
}

pub fn print_pic(img: &PathBuf, to: &PathBuf) {
    send_picture(img, to);
    for _ in 0..2 {
        buttonpress("160", "1300");
        println!("buttonpress 1");
        thread::sleep(time::Duration::from_millis(500));
    }
    // doesnt finalize the printing for now, 1 more presses for that, new coordinates needed to
    // return to base state
    for _ in 0..3 {
        buttonpress("920", "2250");
        println!("buttonpress 2");
        thread::sleep(time::Duration::from_millis(750));
    }
}
