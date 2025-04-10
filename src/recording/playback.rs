use crate::recording::Recording;
use crate::utils;
use std::io::{self, Write};
use std::path::Path;
use std::time::Duration;

pub fn play_session(file: &str, speed: f32) -> io::Result<()> {
    let file_path = utils::get_absolute_path(file);
    println!("Loading recording from {}", file_path.display());

    if !file_path.exists() {
        let autosave_path = file_path.with_extension("json.autosave");
        if autosave_path.exists() {
            println!(
                "Original file not found, but found autosave: {}",
                autosave_path.display()
            );
            return play_session_from_path(&autosave_path, speed);
        }

        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("File not found: {}", file_path.display()),
        ));
    }

    play_session_from_path(&file_path, speed)
}

fn play_session_from_path(file_path: &Path, speed: f32) -> io::Result<()> {
    let frames = Recording::load(&file_path)?;
    println!("Loaded {} frames", frames.len());

    let mut last_timestamp: u128 = 0;

    for frame in frames {
        if last_timestamp > 0 {
            let delay = frame.timestamp - last_timestamp;
            let sleep_time = Duration::from_millis((delay as f32 / speed) as u64);
            std::thread::sleep(sleep_time);
        }
        print!("{}", frame.content);
        io::stdout().flush()?;
        last_timestamp = frame.timestamp;
    }

    println!("\nPlayback complete");
    Ok(())
}
