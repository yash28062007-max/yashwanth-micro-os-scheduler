//! Limine Linear Framebuffer Console Driver
//!
//! Provides pixel-based text rendering to the high-resolution graphical framebuffer
//! provided by the Limine bootloader protocol. Supports scrolling, colors, and embedded 8x8 font.

use core::fmt;
use crate::drivers::font::{FONT_DATA, FONT_HEIGHT, FONT_WIDTH};
use crate::kernel::limine_proto::FRAMEBUFFER_REQUEST;
use crate::sync::SpinLock;

/// Framebuffer Console abstraction.
pub struct FramebufferConsole {
    buffer: *mut u8,
    width: usize,
    height: usize,
    pitch: usize,
    bpp: usize,
    cursor_x: usize,
    cursor_y: usize,
    fg_color: u32,
    bg_color: u32,
    initialized: bool,
}

unsafe impl Send for FramebufferConsole {}
unsafe impl Sync for FramebufferConsole {}

impl FramebufferConsole {
    pub const fn new() -> Self {
        Self {
            buffer: core::ptr::null_mut(),
            width: 0,
            height: 0,
            pitch: 0,
            bpp: 0,
            cursor_x: 0,
            cursor_y: 0,
            fg_color: 0x0000E0E0, // Light Cyan
            bg_color: 0x00000000, // Black
            initialized: false,
        }
    }

    /// Initializes the console from the Limine framebuffer response if available.
    pub fn init(&mut self) {
        let hhdm = unsafe { core::ptr::read_volatile(&crate::kernel::limine_proto::HHDM_REQUEST.response) };
        if hhdm.is_null() {
            let mut serial = crate::drivers::serial::SERIAL_PORT.lock();
            let _ = serial.send_str("[FB  ] HHDM response is NULL (volatile)\r\n");
        } else {
            let mut serial = crate::drivers::serial::SERIAL_PORT.lock();
            let _ = serial.send_str("[FB  ] HHDM response is NOT NULL! (volatile)\r\n");
        }

        let response = unsafe { core::ptr::read_volatile(&FRAMEBUFFER_REQUEST.response) };
        if response.is_null() {
            let mut serial = crate::drivers::serial::SERIAL_PORT.lock();
            let _ = serial.send_str("[FB  ] Framebuffer response is NULL (volatile)\r\n");
            return;
        }

        let resp = unsafe { &*response };
        if resp.framebuffer_count == 0 || resp.framebuffers.is_null() {
            let mut serial = crate::drivers::serial::SERIAL_PORT.lock();
            let _ = serial.send_str("[FB  ] Framebuffer count is 0\r\n");
            return;
        }

        let fb_ptr = unsafe { *resp.framebuffers };
        if fb_ptr.is_null() {
            let mut serial = crate::drivers::serial::SERIAL_PORT.lock();
            let _ = serial.send_str("[FB  ] Framebuffer pointer is NULL\r\n");
            return;
        }

        let fb = unsafe { &*fb_ptr };
        self.buffer = fb.address;
        self.width = fb.width as usize;
        self.height = fb.height as usize;
        self.pitch = fb.pitch as usize;
        self.bpp = (fb.bpp / 8) as usize;
        self.cursor_x = 0;
        self.cursor_y = 0;
        self.initialized = true;

        {
            let mut serial = crate::drivers::serial::SERIAL_PORT.lock();
            let _ = serial.send_str("[FB  ] Framebuffer initialized successfully!\r\n");
        }

        self.clear();
    }

    /// Clears the entire framebuffer screen to background color.
    pub fn clear(&mut self) {
        if !self.initialized || self.buffer.is_null() {
            return;
        }

        for y in 0..self.height {
            let row_offset = y * self.pitch;
            for x in 0..self.width {
                unsafe {
                    let pixel_ptr = self.buffer.add(row_offset + x * self.bpp) as *mut u32;
                    core::ptr::write_volatile(pixel_ptr, self.bg_color);
                }
            }
        }
        self.cursor_x = 0;
        self.cursor_y = 0;
    }

    /// Writes a single character to the framebuffer using the embedded 8x8 font.
    pub fn write_char(&mut self, c: u8) {
        if !self.initialized || self.buffer.is_null() {
            return;
        }

        match c {
            b'\n' => self.new_line(),
            b'\r' => self.cursor_x = 0,
            c => {
                if self.cursor_x + FONT_WIDTH > self.width {
                    self.new_line();
                }

                let glyph_idx = if (c as usize) < FONT_DATA.len() {
                    c as usize
                } else {
                    0
                };
                let glyph = FONT_DATA[glyph_idx];

                for (row, byte) in glyph.iter().enumerate() {
                    let y = self.cursor_y + row;
                    if y >= self.height {
                        break;
                    }
                    let row_offset = y * self.pitch;

                    for col in 0..8 {
                        let x = self.cursor_x + col;
                        if x >= self.width {
                            break;
                        }
                        let is_set = (*byte & (1 << (7 - col))) != 0;
                        let color = if is_set { self.fg_color } else { self.bg_color };

                        unsafe {
                            let pixel_ptr = self.buffer.add(row_offset + x * self.bpp) as *mut u32;
                            core::ptr::write_volatile(pixel_ptr, color);
                        }
                    }
                }

                self.cursor_x += FONT_WIDTH;
            }
        }
    }

    /// Advances the cursor to the next line, scrolling if reaching screen bottom.
    fn new_line(&mut self) {
        self.cursor_x = 0;
        if self.cursor_y + FONT_HEIGHT * 2 <= self.height {
            self.cursor_y += FONT_HEIGHT;
        } else {
            // Scroll framebuffer up by FONT_HEIGHT pixels
            let scroll_bytes = FONT_HEIGHT * self.pitch;
            let total_bytes = self.height * self.pitch;
            unsafe {
                core::ptr::copy(
                    self.buffer.add(scroll_bytes),
                    self.buffer,
                    total_bytes - scroll_bytes,
                );
                // Clear the bottom line
                let bottom_ptr = self.buffer.add(total_bytes - scroll_bytes);
                core::ptr::write_bytes(bottom_ptr, 0, scroll_bytes);
            }
        }
    }

    /// Writes a string slice to the screen.
    pub fn write_str(&mut self, s: &str) {
        for b in s.bytes() {
            self.write_char(b);
        }
    }
}

impl fmt::Write for FramebufferConsole {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_str(s);
        Ok(())
    }
}

pub static FRAMEBUFFER_CONSOLE: SpinLock<FramebufferConsole> =
    SpinLock::new(FramebufferConsole::new());

/// Initializes the framebuffer console if Limine provided a framebuffer.
pub fn init_framebuffer() {
    FRAMEBUFFER_CONSOLE.lock().init();
}
