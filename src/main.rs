use ctrlc;
use gif::{Encoder, Frame, Repeat};
use image::{ImageBuffer, Rgb};
use std::collections::HashMap;
use std::env;
use std::fs::{self, File};
use std::io::{self, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "terminal-recorder", about = "Records terminal sessions")]
enum Cli {
    #[structopt(about = "Record a new terminal session")]
    Record {
        #[structopt(short, long, help = "Output file name", default_value = "demo.json")]
        output: String,
    },
    #[structopt(about = "Play back a recorded terminal session")]
    Play {
        #[structopt(help = "File to replay")]
        file: String,

        #[structopt(short, long, help = "Playback speed multiplier", default_value = "1.0")]
        speed: f32,
    },
    #[structopt(about = "Convert a recording to a GIF")]
    Export {
        #[structopt(help = "Input recording file")]
        input: String,

        #[structopt(help = "Output GIF file", default_value = "output.gif")]
        output: String,

        #[structopt(short, long, help = "Playback speed multiplier", default_value = "1.0")]
        speed: f32,

        #[structopt(short, long, help = "Terminal width", default_value = "80")]
        width: u16,

        #[structopt(short, long, help = "Terminal height", default_value = "24")]
        height: u16,

        #[structopt(short, long, help = "Font size (pixels)", default_value = "16")]
        font_size: u8,

        #[structopt(long, help = "Dark theme")]
        dark_theme: bool,
    },
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
struct RecordedFrame {
    content: String,
    timestamp: u128,
}

#[derive(Clone)]
struct Recording {
    frames: Vec<RecordedFrame>,
    start_time: Instant,
}

impl Recording {
    fn new() -> Self {
        Recording {
            frames: Vec::new(),
            start_time: Instant::now(),
        }
    }

    fn add_frame(&mut self, content: String) {
        if !content.is_empty() {
            let timestamp = self.start_time.elapsed().as_millis();
            self.frames.push(RecordedFrame { content, timestamp });
        }
    }

    fn save(&self, output_path: &Path) -> io::Result<()> {
        println!("Attempting to save recording to: {}", output_path.display());

        // Make sure we have frames to save
        if self.frames.is_empty() {
            println!("Warning: No frames recorded. Creating empty file anyway.");
        }

        // Get the directory part of the path
        if let Some(parent) = output_path.parent() {
            if !parent.exists() {
                println!("Creating directory: {}", parent.display());
                fs::create_dir_all(parent)?;
            }
        }

        // Try to create a temporary file first
        let temp_path = output_path.with_extension("json.tmp");

        // First, create the JSON string
        let json = serde_json::to_string_pretty(&self.frames).map_err(|e| {
            io::Error::new(
                io::ErrorKind::Other,
                format!("JSON serialization error: {}", e),
            )
        })?;

        // Write to the temporary file
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

        // Rename the temporary file to the final file
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

    fn load(path: &Path) -> io::Result<Vec<RecordedFrame>> {
        // Read the file contents
        let contents = fs::read_to_string(path).map_err(|e| {
            io::Error::new(
                e.kind(),
                format!("Failed to read {}: {}", path.display(), e),
            )
        })?;

        // Parse the JSON
        let frames: Vec<RecordedFrame> = serde_json::from_str(&contents).map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Invalid JSON in {}: {}", path.display(), e),
            )
        })?;

        Ok(frames)
    }
}

// Terminal color definitions
#[derive(Clone, Copy)]
struct TermColor {
    r: u8,
    g: u8,
    b: u8,
}

impl TermColor {
    fn to_rgb(&self) -> Rgb<u8> {
        Rgb([self.r, self.g, self.b])
    }
}

// Terminal cell - represents a single character with formatting
#[derive(Clone)]
struct TermCell {
    character: char,
    fg_color: TermColor,
    bg_color: TermColor,
    bold: bool,
    italic: bool,
    underline: bool,
}

impl Default for TermCell {
    fn default() -> Self {
        TermCell {
            character: ' ',
            fg_color: TermColor {
                r: 240,
                g: 240,
                b: 240,
            }, // Light gray text
            bg_color: TermColor {
                r: 30,
                g: 30,
                b: 30,
            }, // Dark background
            bold: false,
            italic: false,
            underline: false,
        }
    }
}

// Character bitmap for rendering text
type CharBitmap = Vec<Vec<bool>>;

// Virtual terminal to process ANSI escape sequences
struct VirtualTerminal {
    width: usize,
    height: usize,
    cells: Vec<Vec<TermCell>>,
    cursor_x: usize,
    cursor_y: usize,
    current_fg: TermColor,
    current_bg: TermColor,
    bold: bool,
    italic: bool,
    underline: bool,
    dark_theme: bool,
    // Character bitmap cache
    char_bitmaps: HashMap<char, CharBitmap>,
}

impl VirtualTerminal {
    fn new(width: usize, height: usize, dark_theme: bool) -> Self {
        let default_fg = if dark_theme {
            TermColor {
                r: 240,
                g: 240,
                b: 240,
            } // Light text for dark theme
        } else {
            TermColor {
                r: 30,
                g: 30,
                b: 30,
            } // Dark text for light theme
        };

        let default_bg = if dark_theme {
            TermColor {
                r: 30,
                g: 30,
                b: 30,
            } // Dark background
        } else {
            TermColor {
                r: 245,
                g: 245,
                b: 245,
            } // Light background
        };

        let mut cells = Vec::with_capacity(height);
        for _ in 0..height {
            let mut row = Vec::with_capacity(width);
            for _ in 0..width {
                let mut cell = TermCell::default();
                cell.fg_color = default_fg;
                cell.bg_color = default_bg;
                row.push(cell);
            }
            cells.push(row);
        }

        // Initialize with character bitmaps
        let char_bitmaps = create_character_bitmaps();

        VirtualTerminal {
            width,
            height,
            cells,
            cursor_x: 0,
            cursor_y: 0,
            current_fg: default_fg,
            current_bg: default_bg,
            bold: false,
            italic: false,
            underline: false,
            dark_theme,
            char_bitmaps,
        }
    }

    fn process_content(&mut self, content: &str) {
        // Very basic ANSI escape sequence parser
        let mut chars = content.chars().peekable();

        while let Some(c) = chars.next() {
            match c {
                '\x1B' => {
                    // ESC character, start of escape sequence
                    if let Some('[') = chars.next() {
                        // This is a CSI sequence
                        let mut sequence = String::new();

                        // Collect the sequence
                        while let Some(&next) = chars.peek() {
                            if next.is_ascii_alphabetic() {
                                // Found the terminator
                                let command = chars.next().unwrap();
                                self.process_csi_sequence(&sequence, command);
                                break;
                            } else {
                                sequence.push(chars.next().unwrap());
                            }
                        }
                    }
                }
                '\n' => {
                    // New line
                    self.cursor_x = 0;
                    self.cursor_y = (self.cursor_y + 1) % self.height;

                    // If we're at the bottom, scroll up
                    if self.cursor_y == 0 {
                        self.scroll_up();
                        self.cursor_y = self.height - 1;
                    }
                }
                '\r' => {
                    // Carriage return - move cursor to start of line
                    self.cursor_x = 0;
                }
                '\t' => {
                    // Tab - move to next tab stop (usually 8 spaces)
                    self.cursor_x = (self.cursor_x + 8) & !7;
                    if self.cursor_x >= self.width {
                        self.cursor_x = 0;
                        self.cursor_y = (self.cursor_y + 1) % self.height;
                    }
                }
                '\x08' => {
                    // Backspace - move back one position
                    if self.cursor_x > 0 {
                        self.cursor_x -= 1;
                    }
                }
                _ => {
                    // Regular character - write to the current position
                    if self.cursor_x < self.width && self.cursor_y < self.height {
                        self.cells[self.cursor_y][self.cursor_x] = TermCell {
                            character: c,
                            fg_color: self.current_fg,
                            bg_color: self.current_bg,
                            bold: self.bold,
                            italic: self.italic,
                            underline: self.underline,
                        };

                        self.cursor_x += 1;
                        if self.cursor_x >= self.width {
                            self.cursor_x = 0;
                            self.cursor_y = (self.cursor_y + 1) % self.height;

                            // Check if we need to scroll
                            if self.cursor_y == 0 {
                                self.scroll_up();
                                self.cursor_y = self.height - 1;
                            }
                        }
                    }
                }
            }
        }
    }

    fn process_csi_sequence(&mut self, sequence: &str, command: char) {
        match command {
            'm' => {
                // SGR (Select Graphic Rendition) parameters
                let params: Vec<&str> = sequence.split(';').collect();

                if params.is_empty() || params[0].is_empty() || params[0] == "0" {
                    // Reset all attributes
                    self.reset_text_attributes();
                } else {
                    let mut i = 0;
                    while i < params.len() {
                        let param = params[i].parse::<u8>().unwrap_or(0);

                        match param {
                            0 => self.reset_text_attributes(),
                            1 => self.bold = true,
                            3 => self.italic = true,
                            4 => self.underline = true,
                            30..=37 => self.set_color(param - 30, true),
                            40..=47 => self.set_color(param - 40, false),
                            90..=97 => self.set_bright_color(param - 90, true),
                            100..=107 => self.set_bright_color(param - 100, false),
                            38 => {
                                // 38;5;n for 256 colors or 38;2;r;g;b for true color (foreground)
                                if i + 1 < params.len() {
                                    let mode = params[i + 1].parse::<u8>().unwrap_or(0);
                                    if mode == 5 && i + 2 < params.len() {
                                        // 256 color mode
                                        let color_idx = params[i + 2].parse::<u8>().unwrap_or(0);
                                        self.set_256_color(color_idx, true);
                                        i += 2;
                                    } else if mode == 2 && i + 4 < params.len() {
                                        // RGB color mode
                                        let r = params[i + 2].parse::<u8>().unwrap_or(0);
                                        let g = params[i + 3].parse::<u8>().unwrap_or(0);
                                        let b = params[i + 4].parse::<u8>().unwrap_or(0);
                                        self.current_fg = TermColor { r, g, b };
                                        i += 4;
                                    }
                                }
                                i += 1; // Skip the next param as we processed it
                            }
                            48 => {
                                // 48;5;n for 256 colors or 48;2;r;g;b for true color (background)
                                if i + 1 < params.len() {
                                    let mode = params[i + 1].parse::<u8>().unwrap_or(0);
                                    if mode == 5 && i + 2 < params.len() {
                                        // 256 color mode
                                        let color_idx = params[i + 2].parse::<u8>().unwrap_or(0);
                                        self.set_256_color(color_idx, false);
                                        i += 2;
                                    } else if mode == 2 && i + 4 < params.len() {
                                        // RGB color mode
                                        let r = params[i + 2].parse::<u8>().unwrap_or(0);
                                        let g = params[i + 3].parse::<u8>().unwrap_or(0);
                                        let b = params[i + 4].parse::<u8>().unwrap_or(0);
                                        self.current_bg = TermColor { r, g, b };
                                        i += 4;
                                    }
                                }
                                i += 1; // Skip the next param as we processed it
                            }
                            _ => {}
                        }

                        i += 1;
                    }
                }
            }
            'A' => {
                // Cursor Up
                let count = sequence.parse::<usize>().unwrap_or(1);
                if self.cursor_y >= count {
                    self.cursor_y -= count;
                } else {
                    self.cursor_y = 0;
                }
            }
            'B' => {
                // Cursor Down
                let count = sequence.parse::<usize>().unwrap_or(1);
                self.cursor_y = (self.cursor_y + count).min(self.height - 1);
            }
            'C' => {
                // Cursor Forward
                let count = sequence.parse::<usize>().unwrap_or(1);
                self.cursor_x = (self.cursor_x + count).min(self.width - 1);
            }
            'D' => {
                // Cursor Backward
                let count = sequence.parse::<usize>().unwrap_or(1);
                if self.cursor_x >= count {
                    self.cursor_x -= count;
                } else {
                    self.cursor_x = 0;
                }
            }
            'H' | 'f' => {
                // Cursor Position
                let parts: Vec<&str> = sequence.split(';').collect();
                let row = if parts.len() > 0 && !parts[0].is_empty() {
                    parts[0].parse::<usize>().unwrap_or(1).saturating_sub(1)
                } else {
                    0
                };

                let col = if parts.len() > 1 && !parts[1].is_empty() {
                    parts[1].parse::<usize>().unwrap_or(1).saturating_sub(1)
                } else {
                    0
                };

                self.cursor_y = row.min(self.height - 1);
                self.cursor_x = col.min(self.width - 1);
            }
            'J' => {
                // Erase in Display
                let mode = sequence.parse::<u8>().unwrap_or(0);

                match mode {
                    0 => {
                        // Erase from cursor to end of screen
                        for x in self.cursor_x..self.width {
                            self.clear_cell(self.cursor_y, x);
                        }

                        for y in (self.cursor_y + 1)..self.height {
                            for x in 0..self.width {
                                self.clear_cell(y, x);
                            }
                        }
                    }
                    1 => {
                        // Erase from start to cursor
                        for y in 0..self.cursor_y {
                            for x in 0..self.width {
                                self.clear_cell(y, x);
                            }
                        }

                        for x in 0..=self.cursor_x {
                            self.clear_cell(self.cursor_y, x);
                        }
                    }
                    2 | 3 => {
                        // Erase entire screen
                        for y in 0..self.height {
                            for x in 0..self.width {
                                self.clear_cell(y, x);
                            }
                        }
                    }
                    _ => {}
                }
            }
            'K' => {
                // Erase in Line
                let mode = sequence.parse::<u8>().unwrap_or(0);

                match mode {
                    0 => {
                        // Erase from cursor to end of line
                        for x in self.cursor_x..self.width {
                            self.clear_cell(self.cursor_y, x);
                        }
                    }
                    1 => {
                        // Erase from start of line to cursor
                        for x in 0..=self.cursor_x {
                            self.clear_cell(self.cursor_y, x);
                        }
                    }
                    2 => {
                        // Erase entire line
                        for x in 0..self.width {
                            self.clear_cell(self.cursor_y, x);
                        }
                    }
                    _ => {}
                }
            }
            _ => {
                // Unsupported command, ignore
            }
        }
    }

    fn reset_text_attributes(&mut self) {
        self.bold = false;
        self.italic = false;
        self.underline = false;

        // Reset colors to defaults
        if self.dark_theme {
            // Dark theme defaults
            self.current_fg = TermColor {
                r: 240,
                g: 240,
                b: 240,
            };
            self.current_bg = TermColor {
                r: 30,
                g: 30,
                b: 30,
            };
        } else {
            // Light theme defaults
            self.current_fg = TermColor {
                r: 30,
                g: 30,
                b: 30,
            };
            self.current_bg = TermColor {
                r: 245,
                g: 245,
                b: 245,
            };
        }
    }

    fn set_color(&mut self, color_index: u8, is_foreground: bool) {
        // Basic ANSI colors
        let color = match color_index {
            0 => TermColor { r: 0, g: 0, b: 0 },   // Black
            1 => TermColor { r: 170, g: 0, b: 0 }, // Red
            2 => TermColor { r: 0, g: 170, b: 0 }, // Green
            3 => TermColor {
                r: 170,
                g: 85,
                b: 0,
            }, // Yellow
            4 => TermColor { r: 0, g: 0, b: 170 }, // Blue
            5 => TermColor {
                r: 170,
                g: 0,
                b: 170,
            }, // Magenta
            6 => TermColor {
                r: 0,
                g: 170,
                b: 170,
            }, // Cyan
            7 => TermColor {
                r: 170,
                g: 170,
                b: 170,
            }, // White
            _ => {
                if is_foreground {
                    self.current_fg
                } else {
                    self.current_bg
                }
            }
        };

        if is_foreground {
            self.current_fg = color;
        } else {
            self.current_bg = color;
        }
    }

    fn set_bright_color(&mut self, color_index: u8, is_foreground: bool) {
        // Bright ANSI colors
        let color = match color_index {
            0 => TermColor {
                r: 85,
                g: 85,
                b: 85,
            }, // Bright Black (gray)
            1 => TermColor {
                r: 255,
                g: 85,
                b: 85,
            }, // Bright Red
            2 => TermColor {
                r: 85,
                g: 255,
                b: 85,
            }, // Bright Green
            3 => TermColor {
                r: 255,
                g: 255,
                b: 85,
            }, // Bright Yellow
            4 => TermColor {
                r: 85,
                g: 85,
                b: 255,
            }, // Bright Blue
            5 => TermColor {
                r: 255,
                g: 85,
                b: 255,
            }, // Bright Magenta
            6 => TermColor {
                r: 85,
                g: 255,
                b: 255,
            }, // Bright Cyan
            7 => TermColor {
                r: 255,
                g: 255,
                b: 255,
            }, // Bright White
            _ => {
                if is_foreground {
                    self.current_fg
                } else {
                    self.current_bg
                }
            }
        };

        if is_foreground {
            self.current_fg = color;
        } else {
            self.current_bg = color;
        }
    }

    fn set_256_color(&mut self, color_index: u8, is_foreground: bool) {
        let color = if color_index < 16 {
            // Standard ANSI colors (0-15)
            let is_bright = color_index >= 8;
            let base_index = color_index % 8;

            if is_bright {
                match base_index {
                    0 => TermColor {
                        r: 85,
                        g: 85,
                        b: 85,
                    },
                    1 => TermColor {
                        r: 255,
                        g: 85,
                        b: 85,
                    },
                    2 => TermColor {
                        r: 85,
                        g: 255,
                        b: 85,
                    },
                    3 => TermColor {
                        r: 255,
                        g: 255,
                        b: 85,
                    },
                    4 => TermColor {
                        r: 85,
                        g: 85,
                        b: 255,
                    },
                    5 => TermColor {
                        r: 255,
                        g: 85,
                        b: 255,
                    },
                    6 => TermColor {
                        r: 85,
                        g: 255,
                        b: 255,
                    },
                    7 => TermColor {
                        r: 255,
                        g: 255,
                        b: 255,
                    },
                    _ => unreachable!(),
                }
            } else {
                match base_index {
                    0 => TermColor { r: 0, g: 0, b: 0 },
                    1 => TermColor { r: 170, g: 0, b: 0 },
                    2 => TermColor { r: 0, g: 170, b: 0 },
                    3 => TermColor {
                        r: 170,
                        g: 85,
                        b: 0,
                    },
                    4 => TermColor { r: 0, g: 0, b: 170 },
                    5 => TermColor {
                        r: 170,
                        g: 0,
                        b: 170,
                    },
                    6 => TermColor {
                        r: 0,
                        g: 170,
                        b: 170,
                    },
                    7 => TermColor {
                        r: 170,
                        g: 170,
                        b: 170,
                    },
                    _ => unreachable!(),
                }
            }
        } else if color_index < 232 {
            // 6x6x6 color cube (16-231)
            let index = color_index - 16;
            let r = (index / 36) % 6;
            let g = (index / 6) % 6;
            let b = index % 6;

            TermColor {
                r: if r == 0 { 0 } else { r * 40 + 55 },
                g: if g == 0 { 0 } else { g * 40 + 55 },
                b: if b == 0 { 0 } else { b * 40 + 55 },
            }
        } else {
            // Grayscale (232-255)
            let value = (color_index - 232) * 10 + 8;
            TermColor {
                r: value,
                g: value,
                b: value,
            }
        };

        if is_foreground {
            self.current_fg = color;
        } else {
            self.current_bg = color;
        }
    }

    fn clear_cell(&mut self, y: usize, x: usize) {
        if y < self.height && x < self.width {
            self.cells[y][x].character = ' ';
            self.cells[y][x].fg_color = self.current_fg;
            self.cells[y][x].bg_color = self.current_bg;
            self.cells[y][x].bold = false;
            self.cells[y][x].italic = false;
            self.cells[y][x].underline = false;
        }
    }

    fn scroll_up(&mut self) {
        // Move all lines up one position
        for y in 1..self.height {
            self.cells[y - 1] = self.cells[y].clone();
        }

        // Clear the bottom line
        for x in 0..self.width {
            self.clear_cell(self.height - 1, x);
        }
    }

    fn render_to_image(&self, font_size: u8) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
        // Cell dimensions in pixels
        let cell_width = font_size as u32;
        let cell_height = (font_size as f32 * 2.0) as u32;

        // Create the image buffer
        let width = (self.width as u32) * cell_width;
        let height = (self.height as u32) * cell_height;
        let mut img = ImageBuffer::new(width, height);

        // Scale factor for bitmap adjustment
        let scale_factor = (font_size as f32 / 8.0).max(1.0) as usize;

        // Fill the image with cells
        for y in 0..self.height {
            for x in 0..self.width {
                let cell = &self.cells[y][x];

                // Calculate pixel positions
                let px_start = x as u32 * cell_width;
                let py_start = y as u32 * cell_height;

                // Draw background
                for py in py_start..py_start + cell_height {
                    for px in px_start..px_start + cell_width {
                        if px < width && py < height {
                            img.put_pixel(px, py, cell.bg_color.to_rgb());
                        }
                    }
                }

                // Draw character using bitmap approach
                if cell.character != ' ' {
                    // Get bitmap for this character, or use the default if not available
                    let bitmap = if let Some(bitmap) = self.char_bitmaps.get(&cell.character) {
                        bitmap
                    } else if let Some(bitmap) = self.char_bitmaps.get(&'?') {
                        // Fallback to question mark for unknown characters
                        bitmap
                    } else {
                        // Skip if we don't have a bitmap at all
                        continue;
                    };

                    // Compute scaled bitmap dimensions
                    let scaled_bitmap = scale_bitmap(bitmap, scale_factor);
                    let bitmap_width = scaled_bitmap[0].len() as u32;
                    let bitmap_height = scaled_bitmap.len() as u32;

                    // Center the character in the cell
                    let offset_x = (cell_width - bitmap_width) / 2;
                    let offset_y = (cell_height - bitmap_height) / 2;

                    // Draw the character bitmap
                    for (dy, row) in scaled_bitmap.iter().enumerate() {
                        for (dx, &pixel) in row.iter().enumerate() {
                            if pixel {
                                let px = px_start + offset_x + dx as u32;
                                let py = py_start + offset_y + dy as u32;

                                if px < width && py < height {
                                    img.put_pixel(px, py, cell.fg_color.to_rgb());
                                }
                            }
                        }
                    }

                    // If underlined, draw a line at the bottom
                    if cell.underline {
                        let underline_y = py_start + cell_height - 2;
                        for dx in 0..cell_width {
                            let px = px_start + dx;
                            if px < width && underline_y < height {
                                img.put_pixel(px, underline_y, cell.fg_color.to_rgb());
                            }
                        }
                    }
                }
            }
        }

        img
    }
}

// Create bitmap representations of characters
fn create_character_bitmaps() -> HashMap<char, CharBitmap> {
    let mut maps = HashMap::new();

    // Uppercase letters
    maps.insert(
        'A',
        vec![
            vec![false, true, true, false],
            vec![true, false, false, true],
            vec![true, false, false, true],
            vec![true, true, true, true],
            vec![true, false, false, true],
            vec![true, false, false, true],
            vec![true, false, false, true],
        ],
    );

    maps.insert(
        'B',
        vec![
            vec![true, true, true, false],
            vec![true, false, false, true],
            vec![true, false, false, true],
            vec![true, true, true, false],
            vec![true, false, false, true],
            vec![true, false, false, true],
            vec![true, true, true, false],
        ],
    );

    maps.insert(
        'C',
        vec![
            vec![false, true, true, true],
            vec![true, false, false, false],
            vec![true, false, false, false],
            vec![true, false, false, false],
            vec![true, false, false, false],
            vec![true, false, false, false],
            vec![false, true, true, true],
        ],
    );

    maps.insert(
        'D',
        vec![
            vec![true, true, true, false],
            vec![true, false, false, true],
            vec![true, false, false, true],
            vec![true, false, false, true],
            vec![true, false, false, true],
            vec![true, false, false, true],
            vec![true, true, true, false],
        ],
    );

    maps.insert(
        'E',
        vec![
            vec![true, true, true, true],
            vec![true, false, false, false],
            vec![true, false, false, false],
            vec![true, true, true, false],
            vec![true, false, false, false],
            vec![true, false, false, false],
            vec![true, true, true, true],
        ],
    );

    maps.insert(
        'F',
        vec![
            vec![true, true, true, true],
            vec![true, false, false, false],
            vec![true, false, false, false],
            vec![true, true, true, false],
            vec![true, false, false, false],
            vec![true, false, false, false],
            vec![true, false, false, false],
        ],
    );

    maps.insert(
        'G',
        vec![
            vec![false, true, true, true],
            vec![true, false, false, false],
            vec![true, false, false, false],
            vec![true, false, true, true],
            vec![true, false, false, true],
            vec![true, false, false, true],
            vec![false, true, true, false],
        ],
    );

    maps.insert(
        'H',
        vec![
            vec![true, false, false, true],
            vec![true, false, false, true],
            vec![true, false, false, true],
            vec![true, true, true, true],
            vec![true, false, false, true],
            vec![true, false, false, true],
            vec![true, false, false, true],
        ],
    );

    maps.insert(
        'I',
        vec![
            vec![true, true, true],
            vec![false, true, false],
            vec![false, true, false],
            vec![false, true, false],
            vec![false, true, false],
            vec![false, true, false],
            vec![true, true, true],
        ],
    );

    maps.insert(
        'J',
        vec![
            vec![false, false, true, true],
            vec![false, false, false, true],
            vec![false, false, false, true],
            vec![false, false, false, true],
            vec![true, false, false, true],
            vec![true, false, false, true],
            vec![false, true, true, false],
        ],
    );

    maps.insert(
        'K',
        vec![
            vec![true, false, false, true],
            vec![true, false, true, false],
            vec![true, true, false, false],
            vec![true, false, false, false],
            vec![true, true, false, false],
            vec![true, false, true, false],
            vec![true, false, false, true],
        ],
    );

    maps.insert(
        'L',
        vec![
            vec![true, false, false, false],
            vec![true, false, false, false],
            vec![true, false, false, false],
            vec![true, false, false, false],
            vec![true, false, false, false],
            vec![true, false, false, false],
            vec![true, true, true, true],
        ],
    );

    maps.insert(
        'M',
        vec![
            vec![true, false, false, false, true],
            vec![true, true, false, true, true],
            vec![true, false, true, false, true],
            vec![true, false, false, false, true],
            vec![true, false, false, false, true],
            vec![true, false, false, false, true],
            vec![true, false, false, false, true],
        ],
    );

    maps.insert(
        'N',
        vec![
            vec![true, false, false, false, true],
            vec![true, true, false, false, true],
            vec![true, false, true, false, true],
            vec![true, false, false, true, true],
            vec![true, false, false, false, true],
            vec![true, false, false, false, true],
            vec![true, false, false, false, true],
        ],
    );

    maps.insert(
        'O',
        vec![
            vec![false, true, true, false],
            vec![true, false, false, true],
            vec![true, false, false, true],
            vec![true, false, false, true],
            vec![true, false, false, true],
            vec![true, false, false, true],
            vec![false, true, true, false],
        ],
    );

    maps.insert(
        'P',
        vec![
            vec![true, true, true, false],
            vec![true, false, false, true],
            vec![true, false, false, true],
            vec![true, true, true, false],
            vec![true, false, false, false],
            vec![true, false, false, false],
            vec![true, false, false, false],
        ],
    );

    maps.insert(
        'Q',
        vec![
            vec![false, true, true, false],
            vec![true, false, false, true],
            vec![true, false, false, true],
            vec![true, false, false, true],
            vec![true, false, true, true],
            vec![true, false, false, true],
            vec![false, true, true, true],
        ],
    );

    maps.insert(
        'R',
        vec![
            vec![true, true, true, false],
            vec![true, false, false, true],
            vec![true, false, false, true],
            vec![true, true, true, false],
            vec![true, true, false, false],
            vec![true, false, true, false],
            vec![true, false, false, true],
        ],
    );

    maps.insert(
        'S',
        vec![
            vec![false, true, true, true],
            vec![true, false, false, false],
            vec![true, false, false, false],
            vec![false, true, true, false],
            vec![false, false, false, true],
            vec![false, false, false, true],
            vec![true, true, true, false],
        ],
    );

    maps.insert(
        'T',
        vec![
            vec![true, true, true, true, true],
            vec![false, false, true, false, false],
            vec![false, false, true, false, false],
            vec![false, false, true, false, false],
            vec![false, false, true, false, false],
            vec![false, false, true, false, false],
            vec![false, false, true, false, false],
        ],
    );

    maps.insert(
        'U',
        vec![
            vec![true, false, false, true],
            vec![true, false, false, true],
            vec![true, false, false, true],
            vec![true, false, false, true],
            vec![true, false, false, true],
            vec![true, false, false, true],
            vec![false, true, true, false],
        ],
    );

    maps.insert(
        'V',
        vec![
            vec![true, false, false, true],
            vec![true, false, false, true],
            vec![true, false, false, true],
            vec![true, false, false, true],
            vec![true, false, false, true],
            vec![false, true, true, false],
            vec![false, false, true, false],
        ],
    );

    maps.insert(
        'W',
        vec![
            vec![true, false, false, false, true],
            vec![true, false, false, false, true],
            vec![true, false, false, false, true],
            vec![true, false, true, false, true],
            vec![true, false, true, false, true],
            vec![true, true, false, true, true],
            vec![true, false, false, false, true],
        ],
    );

    maps.insert(
        'X',
        vec![
            vec![true, false, false, true],
            vec![true, false, false, true],
            vec![false, true, true, false],
            vec![false, false, false, false],
            vec![false, true, true, false],
            vec![true, false, false, true],
            vec![true, false, false, true],
        ],
    );

    maps.insert(
        'Y',
        vec![
            vec![true, false, false, true],
            vec![true, false, false, true],
            vec![false, true, true, false],
            vec![false, false, true, false],
            vec![false, false, true, false],
            vec![false, false, true, false],
            vec![false, false, true, false],
        ],
    );

    maps.insert(
        'Z',
        vec![
            vec![true, true, true, true],
            vec![false, false, false, true],
            vec![false, false, true, false],
            vec![false, true, false, false],
            vec![true, false, false, false],
            vec![true, false, false, false],
            vec![true, true, true, true],
        ],
    );

    // Lowercase letters
    maps.insert(
        'a',
        vec![
            vec![false, false, false, false],
            vec![false, false, false, false],
            vec![false, true, true, false],
            vec![false, false, false, true],
            vec![false, true, true, true],
            vec![true, false, false, true],
            vec![false, true, true, true],
        ],
    );

    maps.insert(
        'b',
        vec![
            vec![true, false, false, false],
            vec![true, false, false, false],
            vec![true, true, true, false],
            vec![true, false, false, true],
            vec![true, false, false, true],
            vec![true, false, false, true],
            vec![true, true, true, false],
        ],
    );

    maps.insert(
        'c',
        vec![
            vec![false, false, false, false],
            vec![false, false, false, false],
            vec![false, true, true, true],
            vec![true, false, false, false],
            vec![true, false, false, false],
            vec![true, false, false, false],
            vec![false, true, true, true],
        ],
    );

    maps.insert(
        'd',
        vec![
            vec![false, false, false, true],
            vec![false, false, false, true],
            vec![false, true, true, true],
            vec![true, false, false, true],
            vec![true, false, false, true],
            vec![true, false, false, true],
            vec![false, true, true, true],
        ],
    );

    maps.insert(
        'e',
        vec![
            vec![false, false, false, false],
            vec![false, false, false, false],
            vec![false, true, true, false],
            vec![true, false, false, true],
            vec![true, true, true, true],
            vec![true, false, false, false],
            vec![false, true, true, true],
        ],
    );

    maps.insert(
        'f',
        vec![
            vec![false, false, true, true],
            vec![false, true, false, false],
            vec![true, true, true, false],
            vec![false, true, false, false],
            vec![false, true, false, false],
            vec![false, true, false, false],
            vec![false, true, false, false],
        ],
    );

    maps.insert(
        'g',
        vec![
            vec![false, false, false, false],
            vec![false, true, true, true],
            vec![true, false, false, true],
            vec![true, false, false, true],
            vec![false, true, true, true],
            vec![false, false, false, true],
            vec![false, true, true, false],
        ],
    );

    maps.insert(
        'h',
        vec![
            vec![true, false, false, false],
            vec![true, false, false, false],
            vec![true, true, true, false],
            vec![true, false, false, true],
            vec![true, false, false, true],
            vec![true, false, false, true],
            vec![true, false, false, true],
        ],
    );

    maps.insert(
        'i',
        vec![
            vec![false, true, false],
            vec![false, false, false],
            vec![true, true, false],
            vec![false, true, false],
            vec![false, true, false],
            vec![false, true, false],
            vec![true, true, true],
        ],
    );

    maps.insert(
        'j',
        vec![
            vec![false, false, true],
            vec![false, false, false],
            vec![false, true, true],
            vec![false, false, true],
            vec![false, false, true],
            vec![true, false, true],
            vec![false, true, false],
        ],
    );

    maps.insert(
        'k',
        vec![
            vec![true, false, false, false],
            vec![true, false, false, false],
            vec![true, false, true, false],
            vec![true, true, false, false],
            vec![true, true, false, false],
            vec![true, false, true, false],
            vec![true, false, false, true],
        ],
    );

    maps.insert(
        'l',
        vec![
            vec![true, true, false],
            vec![false, true, false],
            vec![false, true, false],
            vec![false, true, false],
            vec![false, true, false],
            vec![false, true, false],
            vec![true, true, true],
        ],
    );

    maps.insert(
        'm',
        vec![
            vec![false, false, false, false, false],
            vec![false, false, false, false, false],
            vec![true, true, false, true, false],
            vec![true, false, true, false, true],
            vec![true, false, true, false, true],
            vec![true, false, true, false, true],
            vec![true, false, true, false, true],
        ],
    );

    maps.insert(
        'n',
        vec![
            vec![false, false, false, false],
            vec![false, false, false, false],
            vec![true, true, true, false],
            vec![true, false, false, true],
            vec![true, false, false, true],
            vec![true, false, false, true],
            vec![true, false, false, true],
        ],
    );

    maps.insert(
        'o',
        vec![
            vec![false, false, false, false],
            vec![false, false, false, false],
            vec![false, true, true, false],
            vec![true, false, false, true],
            vec![true, false, false, true],
            vec![true, false, false, true],
            vec![false, true, true, false],
        ],
    );

    maps.insert(
        'p',
        vec![
            vec![false, false, false, false],
            vec![false, false, false, false],
            vec![true, true, true, false],
            vec![true, false, false, true],
            vec![true, true, true, false],
            vec![true, false, false, false],
            vec![true, false, false, false],
        ],
    );

    maps.insert(
        'q',
        vec![
            vec![false, false, false, false],
            vec![false, false, false, false],
            vec![false, true, true, true],
            vec![true, false, false, true],
            vec![false, true, true, true],
            vec![false, false, false, true],
            vec![false, false, false, true],
        ],
    );

    maps.insert(
        'r',
        vec![
            vec![false, false, false, false],
            vec![false, false, false, false],
            vec![true, false, true, true],
            vec![true, true, false, false],
            vec![true, false, false, false],
            vec![true, false, false, false],
            vec![true, false, false, false],
        ],
    );

    maps.insert(
        's',
        vec![
            vec![false, false, false, false],
            vec![false, false, false, false],
            vec![false, true, true, true],
            vec![true, false, false, false],
            vec![false, true, true, false],
            vec![false, false, false, true],
            vec![true, true, true, false],
        ],
    );

    maps.insert(
        't',
        vec![
            vec![false, true, false, false],
            vec![false, true, false, false],
            vec![true, true, true, false],
            vec![false, true, false, false],
            vec![false, true, false, false],
            vec![false, true, false, false],
            vec![false, false, true, true],
        ],
    );

    maps.insert(
        'u',
        vec![
            vec![false, false, false, false],
            vec![false, false, false, false],
            vec![true, false, false, true],
            vec![true, false, false, true],
            vec![true, false, false, true],
            vec![true, false, false, true],
            vec![false, true, true, true],
        ],
    );

    maps.insert(
        'v',
        vec![
            vec![false, false, false, false],
            vec![false, false, false, false],
            vec![true, false, false, true],
            vec![true, false, false, true],
            vec![true, false, false, true],
            vec![false, true, true, false],
            vec![false, false, true, false],
        ],
    );

    maps.insert(
        'w',
        vec![
            vec![false, false, false, false, false],
            vec![false, false, false, false, false],
            vec![true, false, false, false, true],
            vec![true, false, false, false, true],
            vec![true, false, true, false, true],
            vec![true, false, true, false, true],
            vec![false, true, false, true, false],
        ],
    );

    maps.insert(
        'x',
        vec![
            vec![false, false, false, false],
            vec![false, false, false, false],
            vec![true, false, false, true],
            vec![false, true, true, false],
            vec![false, false, false, false],
            vec![false, true, true, false],
            vec![true, false, false, true],
        ],
    );

    maps.insert(
        'y',
        vec![
            vec![false, false, false, false],
            vec![false, false, false, false],
            vec![true, false, false, true],
            vec![true, false, false, true],
            vec![false, true, true, true],
            vec![false, false, false, true],
            vec![false, true, true, false],
        ],
    );

    maps.insert(
        'z',
        vec![
            vec![false, false, false, false],
            vec![false, false, false, false],
            vec![true, true, true, true],
            vec![false, false, true, false],
            vec![false, true, false, false],
            vec![true, false, false, false],
            vec![true, true, true, true],
        ],
    );

    // Numbers 0-9
    maps.insert(
        '0',
        vec![
            vec![false, true, true, false],
            vec![true, false, false, true],
            vec![true, false, true, true],
            vec![true, true, false, true],
            vec![true, false, false, true],
            vec![true, false, false, true],
            vec![false, true, true, false],
        ],
    );

    maps.insert(
        '1',
        vec![
            vec![false, true, false],
            vec![true, true, false],
            vec![false, true, false],
            vec![false, true, false],
            vec![false, true, false],
            vec![false, true, false],
            vec![true, true, true],
        ],
    );

    maps.insert(
        '2',
        vec![
            vec![false, true, true, false],
            vec![true, false, false, true],
            vec![false, false, false, true],
            vec![false, false, true, false],
            vec![false, true, false, false],
            vec![true, false, false, false],
            vec![true, true, true, true],
        ],
    );

    maps.insert(
        '3',
        vec![
            vec![false, true, true, false],
            vec![true, false, false, true],
            vec![false, false, false, true],
            vec![false, true, true, false],
            vec![false, false, false, true],
            vec![true, false, false, true],
            vec![false, true, true, false],
        ],
    );

    maps.insert(
        '4',
        vec![
            vec![false, false, true, false],
            vec![false, true, true, false],
            vec![true, false, true, false],
            vec![true, false, true, false],
            vec![true, true, true, true],
            vec![false, false, true, false],
            vec![false, false, true, false],
        ],
    );

    maps.insert(
        '5',
        vec![
            vec![true, true, true, true],
            vec![true, false, false, false],
            vec![true, true, true, false],
            vec![false, false, false, true],
            vec![false, false, false, true],
            vec![true, false, false, true],
            vec![false, true, true, false],
        ],
    );

    maps.insert(
        '6',
        vec![
            vec![false, true, true, false],
            vec![true, false, false, false],
            vec![true, false, false, false],
            vec![true, true, true, false],
            vec![true, false, false, true],
            vec![true, false, false, true],
            vec![false, true, true, false],
        ],
    );

    maps.insert(
        '7',
        vec![
            vec![true, true, true, true],
            vec![false, false, false, true],
            vec![false, false, true, false],
            vec![false, true, false, false],
            vec![true, false, false, false],
            vec![true, false, false, false],
            vec![true, false, false, false],
        ],
    );

    maps.insert(
        '8',
        vec![
            vec![false, true, true, false],
            vec![true, false, false, true],
            vec![true, false, false, true],
            vec![false, true, true, false],
            vec![true, false, false, true],
            vec![true, false, false, true],
            vec![false, true, true, false],
        ],
    );

    maps.insert(
        '9',
        vec![
            vec![false, true, true, false],
            vec![true, false, false, true],
            vec![true, false, false, true],
            vec![false, true, true, true],
            vec![false, false, false, true],
            vec![false, false, true, false],
            vec![false, true, false, false],
        ],
    );

    // Special characters
    maps.insert(
        '!',
        vec![
            vec![true],
            vec![true],
            vec![true],
            vec![true],
            vec![true],
            vec![false],
            vec![true],
        ],
    );

    maps.insert(
        '@',
        vec![
            vec![false, true, true, true, false],
            vec![true, false, false, false, true],
            vec![true, false, true, true, true],
            vec![true, false, true, false, true],
            vec![true, false, true, true, true],
            vec![true, false, false, false, false],
            vec![false, true, true, true, false],
        ],
    );

    maps.insert(
        '#',
        vec![
            vec![false, true, false, true, false],
            vec![false, true, false, true, false],
            vec![true, true, true, true, true],
            vec![false, true, false, true, false],
            vec![true, true, true, true, true],
            vec![false, true, false, true, false],
            vec![false, true, false, true, false],
        ],
    );

    maps.insert(
        '$',
        vec![
            vec![false, true, false],
            vec![true, true, true],
            vec![true, false, false],
            vec![false, true, false],
            vec![false, false, true],
            vec![true, true, true],
            vec![false, true, false],
        ],
    );

    maps.insert(
        '%',
        vec![
            vec![true, true, false, false, true],
            vec![true, true, false, true, false],
            vec![false, false, true, false, false],
            vec![false, true, false, false, false],
            vec![true, false, false, true, true],
            vec![false, false, false, true, true],
            vec![false, false, false, false, false],
        ],
    );

    maps.insert(
        '^',
        vec![
            vec![false, true, false],
            vec![true, false, true],
            vec![false, false, false],
            vec![false, false, false],
            vec![false, false, false],
            vec![false, false, false],
            vec![false, false, false],
        ],
    );

    maps.insert(
        '&',
        vec![
            vec![false, true, true, false, false],
            vec![true, false, false, true, false],
            vec![true, false, true, false, false],
            vec![false, true, false, false, false],
            vec![true, false, true, false, true],
            vec![true, false, false, true, false],
            vec![false, true, true, false, true],
        ],
    );

    maps.insert(
        '*',
        vec![
            vec![false, false, false],
            vec![true, false, true],
            vec![false, true, false],
            vec![true, true, true],
            vec![false, true, false],
            vec![true, false, true],
            vec![false, false, false],
        ],
    );

    maps.insert(
        '(',
        vec![
            vec![false, true],
            vec![true, false],
            vec![true, false],
            vec![true, false],
            vec![true, false],
            vec![true, false],
            vec![false, true],
        ],
    );

    maps.insert(
        ')',
        vec![
            vec![true, false],
            vec![false, true],
            vec![false, true],
            vec![false, true],
            vec![false, true],
            vec![false, true],
            vec![true, false],
        ],
    );

    maps.insert(
        '-',
        vec![
            vec![false, false, false],
            vec![false, false, false],
            vec![false, false, false],
            vec![true, true, true],
            vec![false, false, false],
            vec![false, false, false],
            vec![false, false, false],
        ],
    );

    maps.insert(
        '_',
        vec![
            vec![false, false, false],
            vec![false, false, false],
            vec![false, false, false],
            vec![false, false, false],
            vec![false, false, false],
            vec![false, false, false],
            vec![true, true, true],
        ],
    );

    maps.insert(
        '=',
        vec![
            vec![false, false, false],
            vec![false, false, false],
            vec![true, true, true],
            vec![false, false, false],
            vec![true, true, true],
            vec![false, false, false],
            vec![false, false, false],
        ],
    );

    maps.insert(
        '+',
        vec![
            vec![false, false, false],
            vec![false, true, false],
            vec![false, true, false],
            vec![true, true, true],
            vec![false, true, false],
            vec![false, true, false],
            vec![false, false, false],
        ],
    );

    maps.insert(
        '[',
        vec![
            vec![true, true],
            vec![true, false],
            vec![true, false],
            vec![true, false],
            vec![true, false],
            vec![true, false],
            vec![true, true],
        ],
    );

    maps.insert(
        ']',
        vec![
            vec![true, true],
            vec![false, true],
            vec![false, true],
            vec![false, true],
            vec![false, true],
            vec![false, true],
            vec![true, true],
        ],
    );

    maps.insert(
        '{',
        vec![
            vec![false, true, true],
            vec![true, false, false],
            vec![true, false, false],
            vec![true, true, false],
            vec![true, false, false],
            vec![true, false, false],
            vec![false, true, true],
        ],
    );

    maps.insert(
        '}',
        vec![
            vec![true, true, false],
            vec![false, false, true],
            vec![false, false, true],
            vec![false, true, true],
            vec![false, false, true],
            vec![false, false, true],
            vec![true, true, false],
        ],
    );

    maps.insert(
        '|',
        vec![
            vec![true],
            vec![true],
            vec![true],
            vec![true],
            vec![true],
            vec![true],
            vec![true],
        ],
    );

    maps.insert(
        '\\',
        vec![
            vec![true, false, false],
            vec![true, false, false],
            vec![false, true, false],
            vec![false, true, false],
            vec![false, true, false],
            vec![false, false, true],
            vec![false, false, true],
        ],
    );

    maps.insert(
        ':',
        vec![
            vec![false],
            vec![true],
            vec![true],
            vec![false],
            vec![true],
            vec![true],
            vec![false],
        ],
    );

    maps.insert(
        ';',
        vec![
            vec![false],
            vec![true],
            vec![true],
            vec![false],
            vec![true],
            vec![true],
            vec![true],
        ],
    );

    maps.insert(
        '"',
        vec![
            vec![true, false, true],
            vec![true, false, true],
            vec![false, false, false],
            vec![false, false, false],
            vec![false, false, false],
            vec![false, false, false],
            vec![false, false, false],
        ],
    );

    maps.insert(
        '\'',
        vec![
            vec![true],
            vec![true],
            vec![false],
            vec![false],
            vec![false],
            vec![false],
            vec![false],
        ],
    );

    maps.insert(
        ',',
        vec![
            vec![false],
            vec![false],
            vec![false],
            vec![false],
            vec![false],
            vec![true],
            vec![true],
        ],
    );

    maps.insert(
        '.',
        vec![
            vec![false],
            vec![false],
            vec![false],
            vec![false],
            vec![false],
            vec![true],
            vec![false],
        ],
    );

    maps.insert(
        '<',
        vec![
            vec![false, false, true],
            vec![false, true, false],
            vec![true, false, false],
            vec![false, true, false],
            vec![false, false, true],
            vec![false, false, false],
            vec![false, false, false],
        ],
    );

    maps.insert(
        '>',
        vec![
            vec![true, false, false],
            vec![false, true, false],
            vec![false, false, true],
            vec![false, true, false],
            vec![true, false, false],
            vec![false, false, false],
            vec![false, false, false],
        ],
    );

    maps.insert(
        '/',
        vec![
            vec![false, false, true],
            vec![false, false, true],
            vec![false, true, false],
            vec![false, true, false],
            vec![false, true, false],
            vec![true, false, false],
            vec![true, false, false],
        ],
    );

    maps.insert(
        '?',
        vec![
            vec![false, true, true, false],
            vec![true, false, false, true],
            vec![false, false, false, true],
            vec![false, false, true, false],
            vec![false, true, false, false],
            vec![false, false, false, false],
            vec![false, true, false, false],
        ],
    );

    maps.insert(
        ' ',
        vec![
            vec![false, false],
            vec![false, false],
            vec![false, false],
            vec![false, false],
            vec![false, false],
            vec![false, false],
            vec![false, false],
        ],
    );

    maps.insert(
        '~',
        vec![
            vec![false, false, false, false],
            vec![false, false, false, false],
            vec![false, true, false, false],
            vec![true, false, true, false],
            vec![false, false, true, false],
            vec![false, false, false, false],
            vec![false, false, false, false],
        ],
    );

    maps.insert(
        '`',
        vec![
            vec![true, false],
            vec![false, true],
            vec![false, false],
            vec![false, false],
            vec![false, false],
            vec![false, false],
            vec![false, false],
        ],
    );

    // Add more special character patterns here

    maps
}

// Scale a bitmap to the desired size
fn scale_bitmap(bitmap: &CharBitmap, scale: usize) -> CharBitmap {
    if scale <= 1 {
        return bitmap.clone();
    }

    let mut scaled = Vec::with_capacity(bitmap.len() * scale);

    for row in bitmap {
        let mut scaled_rows = vec![vec![false; row.len() * scale]; scale];

        for (x, &pixel) in row.iter().enumerate() {
            for sy in 0..scale {
                for sx in 0..scale {
                    scaled_rows[sy][x * scale + sx] = pixel;
                }
            }
        }

        scaled.extend(scaled_rows);
    }

    scaled
}

fn get_absolute_path(filename: &str) -> PathBuf {
    if Path::new(filename).is_absolute() {
        Path::new(filename).to_path_buf()
    } else {
        env::current_dir()
            .expect("Failed to get current directory")
            .join(filename)
    }
}

fn record_session(output_file: &str) -> io::Result<()> {
    // Get the absolute path for the output file
    let output_path = get_absolute_path(output_file);
    println!("Starting terminal recording session");
    println!("All input and output will be recorded");
    println!("Type 'exit' or press Ctrl+C to end the recording");
    println!("Output will be saved to: {}", output_path.display());

    // Try to create an empty file to check permissions early
    {
        let _test_file = File::create(&output_path)?;
        println!("Verified write permissions to output file");
    }

    // Create a shared recording object
    let recording = Arc::new(Mutex::new(Recording::new()));
    let running = Arc::new(AtomicBool::new(true));

    // Set up Ctrl+C handler to gracefully save recording on interrupt
    let r_clone = recording.clone();
    let path_clone = output_path.clone();
    let running_clone = running.clone();

    ctrlc::set_handler(move || {
        println!("\nCtrl+C detected, saving recording and exiting...");
        running_clone.store(false, Ordering::SeqCst);

        // Wait a moment for threads to finish
        thread::sleep(Duration::from_millis(500));

        // Save the recording
        let rec = r_clone.lock().unwrap().clone();
        if let Err(e) = rec.save(&path_clone) {
            eprintln!("Error saving recording on Ctrl+C: {}", e);
        }

        // Exit more forcefully after saving
        std::process::exit(0);
    })
    .expect("Error setting Ctrl+C handler");

    // Start a shell process
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

    // Clone references for threads
    let running_stdout = running.clone();
    let recording_stdout = recording.clone();

    // Thread to capture stdout from the child process
    let stdout_handle = thread::spawn(move || {
        let mut buffer = [0; 1024];
        let mut stdout_reader = child_stdout;

        while running_stdout.load(Ordering::SeqCst) {
            match stdout_reader.read(&mut buffer) {
                Ok(0) => break, // EOF
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

    // Thread to capture stderr from the child process
    let running_stderr = running.clone();
    let recording_stderr = recording.clone();

    let stderr_handle = thread::spawn(move || {
        let mut buffer = [0; 1024];
        let mut stderr_reader = child_stderr;

        while running_stderr.load(Ordering::SeqCst) {
            match stderr_reader.read(&mut buffer) {
                Ok(0) => break, // EOF
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

    // Handle user input
    let stdin = io::stdin();
    let mut input = String::new();

    // Small delay to allow the shell prompt to appear
    thread::sleep(Duration::from_millis(200));

    // Periodically save the recording
    let autosave_recording = recording.clone();
    let autosave_path = output_path.with_extension("json.autosave");
    let autosave_running = running.clone();

    let autosave_handle = thread::spawn(move || {
        let mut counter = 0;
        while autosave_running.load(Ordering::SeqCst) {
            thread::sleep(Duration::from_secs(30)); // Autosave every 30 seconds
            counter += 1;

            // Get current recording data
            let current_recording = {
                let recording_lock = autosave_recording.lock().unwrap();
                recording_lock.clone()
            };

            // Save to autosave file if we have frames
            if !current_recording.frames.is_empty() {
                if let Err(e) = current_recording.save(&autosave_path) {
                    eprintln!("Error during autosave #{}: {}", counter, e);
                } else {
                    println!("\n[Autosave #{} completed]", counter);
                }
            }
        }
    });

    // Main input loop
    while running.load(Ordering::SeqCst) {
        input.clear();
        match stdin.read_line(&mut input) {
            Ok(_) => {
                // Check if user wants to exit
                if input.trim() == "exit" {
                    println!("Exit command detected, ending recording...");
                    break;
                }

                // Send the input to the child process
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

    // Signal all threads to stop
    println!("Shutting down recording...");
    running.store(false, Ordering::SeqCst);

    // Wait for the child process to finish
    let _ = child.kill();

    // Wait a moment for threads to process final output
    thread::sleep(Duration::from_millis(200));

    // Wait for threads to finish
    let _ = stdout_handle.join();
    let _ = stderr_handle.join();
    let _ = autosave_handle.join();

    // Get a clone of the recording data
    let final_recording_data = {
        let recording_lock = recording.lock().unwrap();
        recording_lock.clone()
    };

    // Save the recording
    println!(
        "Preparing to save recording with {} frames",
        final_recording_data.frames.len()
    );

    // Make one final forceful attempt to save
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

fn play_session(file: &str, speed: f32) -> io::Result<()> {
    let file_path = get_absolute_path(file);
    println!("Loading recording from {}", file_path.display());

    if !file_path.exists() {
        // Try with autosave extension if the original file doesn't exist
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

fn export_to_gif(
    input_file: &str,
    output_file: &str,
    speed: f32,
    width: u16,
    height: u16,
    font_size: u8,
    dark_theme: bool,
) -> io::Result<()> {
    let input_path = get_absolute_path(input_file);
    let output_path = get_absolute_path(output_file);

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

fn main() -> io::Result<()> {
    let opt = Cli::from_args();

    match opt {
        Cli::Record { output } => {
            println!("Current directory: {:?}", env::current_dir()?);
            record_session(&output)?
        }
        Cli::Play { file, speed } => play_session(&file, speed)?,
        Cli::Export {
            input,
            output,
            speed,
            width,
            height,
            font_size,
            dark_theme,
        } => export_to_gif(&input, &output, speed, width, height, font_size, dark_theme)?,
    }

    Ok(())
}
