use crate::export::bitmap::{create_character_bitmaps, scale_bitmap, CharBitmap};
use crate::terminal::TermColor;
use image::{ImageBuffer, Rgb};
use std::collections::HashMap;

// Terminal cell - represents a single character with formatting
#[derive(Clone)]
pub struct TermCell {
    pub character: char,
    pub fg_color: TermColor,
    pub bg_color: TermColor,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
}

impl Default for TermCell {
    fn default() -> Self {
        TermCell {
            character: ' ',
            fg_color: TermColor {
                r: 240,
                g: 240,
                b: 240,
            },
            bg_color: TermColor {
                r: 30,
                g: 30,
                b: 30,
            },
            bold: false,
            italic: false,
            underline: false,
        }
    }
}

// Virtual terminal to process ANSI escape sequences
pub struct VirtualTerminal {
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
    pub fn new(width: usize, height: usize, dark_theme: bool) -> Self {
        let default_fg = if dark_theme {
            TermColor {
                r: 240,
                g: 240,
                b: 240,
            }
        } else {
            TermColor {
                r: 30,
                g: 30,
                b: 30,
            }
        };

        let default_bg = if dark_theme {
            TermColor {
                r: 30,
                g: 30,
                b: 30,
            }
        } else {
            TermColor {
                r: 245,
                g: 245,
                b: 245,
            }
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

    pub fn process_content(&mut self, content: &str) {
        let mut chars = content.chars().peekable();

        while let Some(c) = chars.next() {
            match c {
                '\x1B' => {
                    if let Some('[') = chars.next() {
                        let mut sequence = String::new();

                        while let Some(&next) = chars.peek() {
                            if next.is_ascii_alphabetic() {
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
                    self.cursor_x = 0;
                    self.cursor_y = (self.cursor_y + 1) % self.height;

                    if self.cursor_y == 0 {
                        self.scroll_up();
                        self.cursor_y = self.height - 1;
                    }
                }
                '\r' => {
                    self.cursor_x = 0;
                }
                '\t' => {
                    self.cursor_x = (self.cursor_x + 8) & !7;
                    if self.cursor_x >= self.width {
                        self.cursor_x = 0;
                        self.cursor_y = (self.cursor_y + 1) % self.height;
                    }
                }
                '\x08' => {
                    if self.cursor_x > 0 {
                        self.cursor_x -= 1;
                    }
                }
                _ => {
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
                let params: Vec<&str> = sequence.split(';').collect();

                if params.is_empty() || params[0].is_empty() || params[0] == "0" {
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
                                if i + 1 < params.len() {
                                    let mode = params[i + 1].parse::<u8>().unwrap_or(0);
                                    if mode == 5 && i + 2 < params.len() {
                                        let color_idx = params[i + 2].parse::<u8>().unwrap_or(0);
                                        self.set_256_color(color_idx, true);
                                        i += 2;
                                    } else if mode == 2 && i + 4 < params.len() {
                                        let r = params[i + 2].parse::<u8>().unwrap_or(0);
                                        let g = params[i + 3].parse::<u8>().unwrap_or(0);
                                        let b = params[i + 4].parse::<u8>().unwrap_or(0);
                                        self.current_fg = TermColor { r, g, b };
                                        i += 4;
                                    }
                                }
                                i += 1;
                            }
                            48 => {
                                if i + 1 < params.len() {
                                    let mode = params[i + 1].parse::<u8>().unwrap_or(0);
                                    if mode == 5 && i + 2 < params.len() {
                                        let color_idx = params[i + 2].parse::<u8>().unwrap_or(0);
                                        self.set_256_color(color_idx, false);
                                        i += 2;
                                    } else if mode == 2 && i + 4 < params.len() {
                                        let r = params[i + 2].parse::<u8>().unwrap_or(0);
                                        let g = params[i + 3].parse::<u8>().unwrap_or(0);
                                        let b = params[i + 4].parse::<u8>().unwrap_or(0);
                                        self.current_bg = TermColor { r, g, b };
                                        i += 4;
                                    }
                                }
                                i += 1;
                            }
                            _ => {}
                        }

                        i += 1;
                    }
                }
            }
            'A' => {
                let count = sequence.parse::<usize>().unwrap_or(1);
                if self.cursor_y >= count {
                    self.cursor_y -= count;
                } else {
                    self.cursor_y = 0;
                }
            }
            'B' => {
                let count = sequence.parse::<usize>().unwrap_or(1);
                self.cursor_y = (self.cursor_y + count).min(self.height - 1);
            }
            'C' => {
                let count = sequence.parse::<usize>().unwrap_or(1);
                self.cursor_x = (self.cursor_x + count).min(self.width - 1);
            }
            'D' => {
                let count = sequence.parse::<usize>().unwrap_or(1);
                if self.cursor_x >= count {
                    self.cursor_x -= count;
                } else {
                    self.cursor_x = 0;
                }
            }
            'H' | 'f' => {
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
                let mode = sequence.parse::<u8>().unwrap_or(0);

                match mode {
                    0 => {
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
                let mode = sequence.parse::<u8>().unwrap_or(0);

                match mode {
                    0 => {
                        for x in self.cursor_x..self.width {
                            self.clear_cell(self.cursor_y, x);
                        }
                    }
                    1 => {
                        for x in 0..=self.cursor_x {
                            self.clear_cell(self.cursor_y, x);
                        }
                    }
                    2 => {
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

        if self.dark_theme {
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
        let color = match color_index {
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
        let color = match color_index {
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

    pub fn render_to_image(&self, font_size: u8) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
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
