pub mod playback;
pub mod recorder;

use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::Instant;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RecordedFrame {
    pub content: String,
    pub timestamp: u128,
}

#[derive(Clone)]
pub struct Recording {
    pub frames: Vec<RecordedFrame>,
    pub start_time: Instant,
}

impl Recording {
    pub fn new() -> Self {
        Recording {
            frames: Vec::new(),
            start_time: Instant::now(),
        }
    }

    pub fn add_frame(&mut self, content: String) {
        if !content.is_empty() {
            let timestamp = self.start_time.elapsed().as_millis();
            self.frames.push(RecordedFrame { content, timestamp });
        }
    }

    pub fn save(&self, output_path: &Path) -> io::Result<()> {
        println!("Attempting to save recording to: {}", output_path.display());

        if self.frames.is_empty() {
            println!("Warning: No frames recorded. Creating empty file anyway.");
        }

        if let Some(parent) = output_path.parent() {
            if !parent.exists() {
                println!("Creating directory: {}", parent.display());
                fs::create_dir_all(parent)?;
            }
        }

        let temp_path = output_path.with_extension("json.tmp");
        let json = serde_json::to_string_pretty(&self.frames).map_err(|e| {
            io::Error::new(
                io::ErrorKind::Other,
                format!("JSON serialization error: {}", e),
            )
        })?;

        fs::write(&temp_path, &json).map_err(|e| {
            io::Error::new(
                e.kind(),
                format!(
                    "Failed to write to temporary file {}: {}",
                    temp_path.display(),
                    e
                ),
            )
        })?;

        fs::rename(&temp_path, output_path).map_err(|e| {
            io::Error::new(
                e.kind(),
                format!(
                    "Failed to rename temporary file to {}: {}",
                    output_path.display(),
                    e
                ),
            )
        })?;

        println!(
            "Successfully saved {} frames ({} bytes) to {}",
            self.frames.len(),
            json.len(),
            output_path.display()
        );

        Ok(())
    }

    pub fn load(path: &Path) -> io::Result<Vec<RecordedFrame>> {
        let contents = fs::read_to_string(path).map_err(|e| {
            io::Error::new(
                e.kind(),
                format!("Failed to read {}: {}", path.display(), e),
            )
        })?;

        let frames: Vec<RecordedFrame> = serde_json::from_str(&contents).map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Invalid JSON in {}: {}", path.display(), e),
            )
        })?;

        Ok(frames)
    }
}
