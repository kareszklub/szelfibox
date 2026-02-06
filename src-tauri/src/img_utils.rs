use std::{
    fs,
    path::{Path, PathBuf},
};

use image::{imageops::FilterType, GenericImage, ImageBuffer, Rgba};

pub fn append_image_header(img: ImageBuffer<Rgba<u8>, Vec<u8>>) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let header = image::open("../static/header.png")
        .expect("failed to load header.png")
        .into_rgba8();

    let target_height = header.height();

    let scale = target_height as f32 / img.height() as f32;
    let new_width = (img.width() as f32 * scale).round() as u32;

    let resized_img = image::imageops::resize(&img, new_width, target_height, FilterType::Lanczos3);

    let total_width = resized_img.width() + header.width();

    let mut output = ImageBuffer::new(total_width, target_height);

    output
        .copy_from(&resized_img, 0, 0)
        .expect("failed to copy resized image");

    output
        .copy_from(&header, resized_img.width(), 0)
        .expect("failed to copy header image");

    output
}

pub fn get_newest_file(dir: &Path) -> Option<PathBuf> {
    let entries = fs::read_dir(dir).ok()?;

    entries
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let metadata = entry.metadata().ok()?;

            // Only consider actual files (skip directories/symlinks)
            if !metadata.is_file() {
                return None;
            }

            // Attempt to get creation time; some systems/filesystems
            // do not support this and will return an error.
            let creation_time = metadata.created().ok()?;

            Some((entry.path(), creation_time))
        })
        // Compare by the SystemTime
        .max_by_key(|&(_, time)| time)
        // Return only the PathBuf
        .map(|(path, _)| path)
}
