use crate::recording::Recording;
use crate::utils;
use ctrlc;
use std::fs;
use std::io::{self, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub fn record_session(output_file: &str) -> io::Result<()> {
    let output_path = utils::get_absolute_path(output_file);
    println!("Starting terminal recording session");
    println!("All input and output will be recorded");
    println!("Type 'exit' or press Ctrl+C to end the recording");
    println!("Output will be saved to: {}", output_path.display());

    {
        let _test_file = std::fs::File::create(&output_path)?;
        println!("Verified write permissions to output file");
    }

    let recording = Arc::new(Mutex::new(Recording::new()));
    let running = Arc::new(AtomicBool::new(true));

    let r_clone = recording.clone();
    let path_clone = output_path.clone();
    let running_clone = running.clone();

    ctrlc::set_handler(move || {
        println!("\nCtrl+C detected, saving recording and exiting...");
        running_clone.store(false, Ordering::SeqCst);

        thread::sleep(Duration::from_millis(500));

        let rec = r_clone.lock().unwrap().clone();
        if let Err(e) = rec.save(&path_clone) {
            eprintln!("Error saving recording on Ctrl+C: {}", e);
        }

        std::process::exit(0);
    })
    .expect("Error setting Ctrl+C handler");

    let shell = if cfg!(target_os = "windows") {
        "cmd"
    } else {
        "bash"
    };

    let mut child = Command::new(shell)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let mut child_stdin = child.stdin.take().expect("Failed to open stdin");
    let child_stdout = child.stdout.take().expect("Failed to open stdout");
    let child_stderr = child.stderr.take().expect("Failed to open stderr");

    let running_stdout = running.clone();
    let recording_stdout = recording.clone();

    let stdout_handle = thread::spawn(move || {
        let mut buffer = [0; 1024];
        let mut stdout_reader = child_stdout;

        while running_stdout.load(Ordering::SeqCst) {
            match stdout_reader.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => {
                    let content = String::from_utf8_lossy(&buffer[0..n]).to_string();
                    if !content.is_empty() {
                        print!("{}", content);
                        io::stdout().flush().unwrap_or_default();
                        recording_stdout.lock().unwrap().add_frame(content);
                    }
                }
                Err(e) => {
                    eprintln!("Error reading from child stdout: {}", e);
                    break;
                }
            }
        }
    });

    let running_stderr = running.clone();
    let recording_stderr = recording.clone();

    let stderr_handle = thread::spawn(move || {
        let mut buffer = [0; 1024];
        let mut stderr_reader = child_stderr;

        while running_stderr.load(Ordering::SeqCst) {
            match stderr_reader.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => {
                    let content = String::from_utf8_lossy(&buffer[0..n]).to_string();
                    if !content.is_empty() {
                        eprint!("{}", content);
                        io::stderr().flush().unwrap_or_default();
                        recording_stderr.lock().unwrap().add_frame(content);
                    }
                }
                Err(e) => {
                    eprintln!("Error reading from child stderr: {}", e);
                    break;
                }
            }
        }
    });

    let stdin = io::stdin();
    let mut input = String::new();

    thread::sleep(Duration::from_millis(200));

    let autosave_recording = recording.clone();
    let autosave_path = output_path.with_extension("json.autosave");
    let autosave_running = running.clone();

    let autosave_handle = thread::spawn(move || {
        let mut counter = 0;
        while autosave_running.load(Ordering::SeqCst) {
            thread::sleep(Duration::from_secs(30));
            counter += 1;

            let current_recording = {
                let recording_lock = autosave_recording.lock().unwrap();
                recording_lock.clone()
            };

            if !current_recording.frames.is_empty() {
                if let Err(e) = current_recording.save(&autosave_path) {
                    eprintln!("Error during autosave #{}: {}", counter, e);
                } else {
                    println!("\n[Autosave #{} completed]", counter);
                }
            }
        }
    });

    while running.load(Ordering::SeqCst) {
        input.clear();
        match stdin.read_line(&mut input) {
            Ok(_) => {
                if input.trim() == "exit" {
                    println!("Exit command detected, ending recording...");
                    break;
                }

                match child_stdin.write_all(input.as_bytes()) {
                    Ok(_) => {
                        child_stdin.flush().unwrap_or_default();
                    }
                    Err(e) => {
                        eprintln!("Failed to write to child stdin: {}", e);
                        break;
                    }
                }
            }
            Err(e) => {
                eprintln!("Error reading from stdin: {}", e);
                break;
            }
        }
    }

    println!("Shutting down recording...");
    running.store(false, Ordering::SeqCst);

    let _ = child.kill();

    thread::sleep(Duration::from_millis(200));

    let _ = stdout_handle.join();
    let _ = stderr_handle.join();
    let _ = autosave_handle.join();

    let final_recording_data = {
        let recording_lock = recording.lock().unwrap();
        recording_lock.clone()
    };

    println!(
        "Preparing to save recording with {} frames",
        final_recording_data.frames.len()
    );

    fs::write(
        &output_path,
        serde_json::to_string_pretty(&final_recording_data.frames).unwrap_or_default(),
    )?;

    println!("Recording saved to {}", output_path.display());
    println!(
        "You can convert this to a GIF with: terminal-recorder export {} output.gif",
        output_path.display()
    );

    Ok(())
}
