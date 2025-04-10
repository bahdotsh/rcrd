use std::env;
use std::path::{Path, PathBuf};

pub fn get_absolute_path(filename: &str) -> PathBuf {
    if Path::new(filename).is_absolute() {
        Path::new(filename).to_path_buf()
    } else {
        env::current_dir()
            .expect("Failed to get current directory")
            .join(filename)
    }
}
