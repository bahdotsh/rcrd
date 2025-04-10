use crate::recording::{RecordedFrame, Recording};
use crate::terminal::VirtualTerminal;
use crate::utils;
use gif::{Encoder, Frame, Repeat};
use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::Path;

pub fn export_to_gif(
    input_file: &str,
    output_file: &str,
    speed: f32,
    width: u16,
    height: u16,
    font_size: u8,
    dark_theme: bool,
) -> io::Result<()> {
    let input_path = utils::get_absolute_path(input_file);
    let output_path = utils::get_absolute_path(output_file);

    println!("Loading recording from {}", input_path.display());

    if !input_path.exists() {
        // Try with autosave extension if the original file doesn't exist
        let autosave_path = input_path.with_extension("json.autosave");
        if autosave_path.exists() {
            println!(
                "Original file not found, but found autosave: {}",
                autosave_path.display()
            );
            return export_to_gif_from_path(
                &autosave_path,
                &output_path,
                speed,
                width,
                height,
                font_size,
                dark_theme,
            );
        }

        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("File not found: {}", input_path.display()),
        ));
    }

    export_to_gif_from_path(
        &input_path,
        &output_path,
        speed,
        width,
        height,
        font_size,
        dark_theme,
    )
}

fn export_to_gif_from_path(
    input_path: &Path,
    output_path: &Path,
    speed: f32,
    width: u16,
    height: u16,
    font_size: u8,
    dark_theme: bool,
) -> io::Result<()> {
    println!("Converting terminal recording to GIF...");

    // Load the frames
    let frames = Recording::load(input_path)?;
    println!("Loaded {} frames", frames.len());

    if frames.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "No frames found in recording file",
        ));
    }

    // Create the virtual terminal
    let mut terminal = VirtualTerminal::new(width as usize, height as usize, dark_theme);

    // Enhanced frames with intro text
    let enhanced_frames = enhance_recording(frames);

    // Setup GIF encoder
    let file = File::create(output_path)?;
    let cell_width = font_size as u32;
    let cell_height = (font_size as f32 * 2.0) as u32;
    let image_width = width as u32 * cell_width;
    let image_height = height as u32 * cell_height;

    // Create the encoder
    let mut encoder = Encoder::new(
        BufWriter::new(file),
        image_width as u16,
        image_height as u16,
        &[],
    )
    .map_err(|e| {
        io::Error::new(
            io::ErrorKind::Other,
            format!("Failed to create GIF encoder: {}", e),
        )
    })?;

    // Configure the GIF encoder
    encoder.set_repeat(Repeat::Infinite).map_err(|e| {
        io::Error::new(
            io::ErrorKind::Other,
            format!("Failed to set GIF repeat mode: {}", e),
        )
    })?;

    println!(
        "Creating GIF with dimensions {}x{}",
        image_width, image_height
    );

    // Process frames and add to GIF
    let mut last_timestamp: u128 = 0;
    let mut frame_counter = 0;

    for frame in enhanced_frames {
        // Calculate delay since last frame
        let mut delay_centisecs = 10; // Default delay (0.1 seconds)

        if last_timestamp > 0 {
            let delay_ms = frame.timestamp - last_timestamp;
            // Convert to centiseconds and apply speed factor
            delay_centisecs = ((delay_ms as f32 / speed) / 10.0) as u16;

            // Limit delay to reasonable bounds (0.02s to 5s)
            delay_centisecs = delay_centisecs.clamp(2, 500);
        }

        // Process this frame's content
        terminal.process_content(&frame.content);

        // Render the terminal to an image
        let img = terminal.render_to_image(font_size);

        // Convert to GIF frame format
        let mut buffer = Vec::new();
        for pixel in img.pixels() {
            buffer.push(pixel[0]);
            buffer.push(pixel[1]);
            buffer.push(pixel[2]);
        }

        // Add frame to GIF
        let mut gif_frame = Frame::from_rgb(image_width as u16, image_height as u16, &buffer);

        gif_frame.delay = delay_centisecs;

        encoder.write_frame(&gif_frame).map_err(|e| {
            io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to write frame to GIF: {}", e),
            )
        })?;

        frame_counter += 1;
        if frame_counter % 10 == 0 {
            print!(".");
            io::stdout().flush()?;
        }

        last_timestamp = frame.timestamp;
    }

    println!("\nGIF successfully created at {}", output_path.display());
    println!("Frames processed: {}", frame_counter);

    Ok(())
}

fn enhance_recording(frames: Vec<RecordedFrame>) -> Vec<RecordedFrame> {
    let mut enhanced = Vec::new();

    // Add intro frame
    enhanced.push(RecordedFrame {
        content: "\x1B[H\x1B[2J\x1B[1;32m# Terminal Recording\x1B[0m\n\n".to_string(),
        timestamp: 0,
    });

    // Add a small delay
    enhanced.push(RecordedFrame {
        content: "\x1B[1;34m$ \x1B[0m".to_string(), // Colored prompt
        timestamp: 1000,                            // 1 second after welcome
    });

    // Add the original frames, adjusting timestamps
    let time_offset = 1500; // 1.5 seconds of intro time
    for frame in frames {
        enhanced.push(RecordedFrame {
            content: frame.content,
            timestamp: frame.timestamp + time_offset,
        });
    }

    // Add outro frame
    let last_timestamp = enhanced.last().map(|f| f.timestamp).unwrap_or(0);
    enhanced.push(RecordedFrame {
        content: "\n\n\x1B[1;32m# End of Recording\x1B[0m\n".to_string(),
        timestamp: last_timestamp + 1000, // 1 second after the last frame
    });

    enhanced
}
