use glob;
use log::error;
use std::{fs::create_dir_all, path::PathBuf, process::Command, thread, time};

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
