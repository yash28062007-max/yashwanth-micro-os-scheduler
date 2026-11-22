//! VGA Text Mode Buffer Driver
//!
//! Provides text rendering to the standard 80x25 IBM VGA text buffer at 0xb8000.
//! Supports colors, scrolling, backspace, and newlines.

use core::fmt;
use crate::sync::SpinLock;

const VGA_BUFFER_PHYSICAL: usize = 0xb8000;
const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorCode(u8);

impl ColorCode {
    const fn new(foreground: Color, background: Color) -> Self {
        Self((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

/// VGA Text Buffer representation.
pub struct VgaWriter {
    column_position: usize,
    row_position: usize,
    color_code: ColorCode,
    #[allow(dead_code)]
    buffer: *mut ScreenChar,
}

unsafe impl Send for VgaWriter {}
unsafe impl Sync for VgaWriter {}

impl VgaWriter {
    /// Creates a new VGA Writer mapped to the standard VGA buffer.
    pub const fn new(buffer_ptr: usize) -> Self {
        Self {
            column_position: 0,
            row_position: 0,
            color_code: ColorCode::new(Color::LightCyan, Color::Black),
            buffer: buffer_ptr as *mut ScreenChar,
        }
    }

    /// Sets the text foreground and background color.
    pub fn set_color(&mut self, foreground: Color, background: Color) {
        self.color_code = ColorCode::new(foreground, background);
    }

    /// Writes a single byte to the screen.
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = self.row_position;
                let col = self.column_position;
                let color_code = self.color_code;

                #[cfg(any(target_os = "none", feature = "kernel_bin"))]
                unsafe {
                    let offset = row * BUFFER_WIDTH + col;
                    core::ptr::write_volatile(
                        self.buffer.add(offset),
                        ScreenChar {
                            ascii_character: byte,
                            color_code,
                        },
                    );
                }
                #[cfg(not(any(target_os = "none", feature = "kernel_bin")))]
                let _ = (row, col, color_code, byte);

                self.column_position += 1;
            }
        }
    }

    /// Writes an ASCII string to the screen.
    pub fn write_str(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                _ => self.write_byte(0xfe), // Render unprintable characters as square
            }
        }
    }

    /// Advances to the next line, scrolling buffer if at bottom.
    fn new_line(&mut self) {
        if self.row_position < BUFFER_HEIGHT - 1 {
            self.row_position += 1;
        } else {
            // Scroll screen up by 1 row
            #[cfg(any(target_os = "none", feature = "kernel_bin"))]
            unsafe {
                for r in 1..BUFFER_HEIGHT {
                    for c in 0..BUFFER_WIDTH {
                        let src_offset = r * BUFFER_WIDTH + c;
                        let dst_offset = (r - 1) * BUFFER_WIDTH + c;
                        let character = core::ptr::read_volatile(self.buffer.add(src_offset));
                        core::ptr::write_volatile(self.buffer.add(dst_offset), character);
                    }
                }
                // Clear the bottom row
                let blank = ScreenChar {
                    ascii_character: b' ',
                    color_code: self.color_code,
                };
                for c in 0..BUFFER_WIDTH {
                    let offset = (BUFFER_HEIGHT - 1) * BUFFER_WIDTH + c;
                    core::ptr::write_volatile(self.buffer.add(offset), blank);
                }
            }
        }
        self.column_position = 0;
    }

    /// Clears the entire VGA text screen.
    pub fn clear_screen(&mut self) {
        #[cfg(any(target_os = "none", feature = "kernel_bin"))]
        {
            let blank = ScreenChar {
                ascii_character: b' ',
                color_code: self.color_code,
            };
            unsafe {
                for offset in 0..(BUFFER_WIDTH * BUFFER_HEIGHT) {
                    core::ptr::write_volatile(self.buffer.add(offset), blank);
                }
            }
        }
        self.row_position = 0;
        self.column_position = 0;
    }
}

impl fmt::Write for VgaWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_str(s);
        Ok(())
    }
}

/// Global VGA writer singleton.
pub static VGA_WRITER: SpinLock<VgaWriter> = SpinLock::new(VgaWriter::new(VGA_BUFFER_PHYSICAL));

/// Clears the VGA text buffer screen.
pub fn clear_screen() {
    VGA_WRITER.lock().clear_screen();
}
