use std::{
    fs,
    path::{Path, PathBuf},
};

pub fn get_newest_file(dir: &Path) -> Option<PathBuf> {
    let entries = fs::read_dir(dir).ok()?;

    entries
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let metadata = entry.metadata().ok()?;

            if !metadata.is_file() {
                return None;
            }

            let creation_time = metadata.created().ok()?;

            Some((entry.path(), creation_time))
        })
        .max_by_key(|&(_, time)| time)
        .map(|(path, _)| path)
}
