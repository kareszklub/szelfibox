use glob;
use log::{error, info};
use std::{
    fs::create_dir_all,
    path::{Path, PathBuf},
    process::Command,
    thread, time,
};

fn buttonpress(x: i32, y: i32) {
    Command::new("adb")
        .arg("shell")
        .arg("input")
        .arg("tap")
        .arg(x.to_string())
        .arg(y.to_string())
        .spawn()
        .expect("failed to press on screen")
        .wait()
        .unwrap();
}

pub fn get_phone_picture_dir() -> PathBuf {
    Path::new("mtp:/REDMAGIC 9 Pro/Internal shared storage/Pictures/szelfibox/").to_path_buf()
}

fn send_picture(img: &PathBuf, to: &PathBuf) {
    println!(
        "sending with: kioclient cp '{}' '{}'",
        img.to_str().unwrap(),
        to.to_str().unwrap()
    );

    Command::new("kioclient")
        .arg("cp")
        .arg(img.to_str().unwrap())
        .arg(to.to_str().unwrap())
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
}

pub fn print_pic(img: &PathBuf, to: &PathBuf) {
    send_picture(img, to);

    let positions = vec![
        (160, 700, 1, 500),
        (160, 1300, 1, 500),
        (1000, 2380, 3, 500),
        (300, 1450, 1, 15000),
    ];

    for (x, y, repeat, wait) in positions {
        for _ in 0..repeat {
            thread::sleep(time::Duration::from_millis(wait));
            buttonpress(x, y);
        }
    }
}
