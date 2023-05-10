#![no_std]

use bresenham::Bresenham;
use core::slice::from_raw_parts_mut;
use font8x8::UnicodeFonts;
use vga::colors::DEFAULT_PALETTE;
use vga::drawing::Point;
use vga::registers::PlaneMask;
use vga::vga::VGA;
use vga::writers::{GraphicsWriter, Screen};

pub fn do_stuff(framebuffer_start: usize) {
    VGA.lock().set_memory_start(framebuffer_start);

    let mode = Graphics1280x800x256::new();
    mode.set_mode();
    mode.clear_screen(0x00_ff_ff_ff);
    mode.draw_line((80, 60), (80, 420), 0x00_00_00_00);
    mode.draw_line((80, 60), (540, 60), 0x00_00_00_00);
    mode.draw_line((80, 420), (540, 420), 0x00_00_00_00);
    mode.draw_line((540, 420), (540, 60), 0x00_00_00_00);
    mode.draw_line((80, 90), (540, 90), 0x00_00_00_00);
    for (offset, character) in "Hello World!".chars().enumerate() {
        mode.draw_character(270 + offset * 8, 72, character, 0x00_77_77_77)
    }

    mode.draw_rect((80, 90), (540, 420), 0x00_00_00_00);

    for y in (600..780).step_by(20) {
        mode.draw_rect((80, y), (540, y + 10), 0x00);
    }

    for l in (100..600).step_by(20) {
        for r in (100..600).step_by(20) {
            mode.draw_line((600, l), (1100, r), 0x00);
        }
    }

    for r in (0_usize..255) {
        for g in (0_usize..255) {
            for b in (0_usize..255) {
                for y in (600..780).step_by(20) {
                    mode.draw_rect((80, y), (540, y + 10), (r << 16 | g << 8 | b) as u32);
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct Graphics1280x800x256;

const WIDTH: usize = 1280;
const HEIGHT: usize = 800;
const BYTES_PER_PIXEL: usize = 4;
const SIZE: usize = WIDTH * HEIGHT * BYTES_PER_PIXEL;

impl Screen for Graphics1280x800x256 {
    const WIDTH: usize = WIDTH;
    const HEIGHT: usize = HEIGHT;
    const SIZE: usize = SIZE;
}

impl Graphics1280x800x256 {
    const fn new() -> Self {
        Self
    }

    pub fn draw_rect(&self, start: Point<usize>, end: Point<usize>, color: u32) {
        let frame_buffer = self.get_frame_buffer() as *mut u32;
        let line_width = end.0.abs_diff(start.0);

        (start.1..end.1)
            .map(|y| WIDTH * y + start.0)
            .map(|offset| unsafe { frame_buffer.add(offset) })
            .map(|ptr| unsafe { from_raw_parts_mut(ptr, line_width) })
            .for_each(|line| line.fill(color));
    }
}

impl GraphicsWriter<u32> for Graphics1280x800x256 {
    fn clear_screen(&self, color: u32) {
        let frame_buffer = self.get_frame_buffer() as *mut u32;
        VGA.lock()
            .sequencer_registers
            .set_plane_mask(PlaneMask::ALL_PLANES);
        unsafe {
            from_raw_parts_mut(frame_buffer, Self::SIZE / BYTES_PER_PIXEL).fill(color);
        }
    }

    fn draw_line(&self, start: Point<isize>, end: Point<isize>, color: u32) {
        for (x, y) in Bresenham::new(start, end) {
            self.set_pixel(x as usize, y as usize, color);
        }
    }

    fn draw_character(&self, x: usize, y: usize, character: char, color: u32) {
        let character = match font8x8::BASIC_FONTS.get(character) {
            Some(character) => character,
            // Default to a filled block if the character isn't found
            None => font8x8::unicode::BLOCK_UNICODE[8].byte_array(),
        };

        for (row, byte) in character.iter().enumerate() {
            for bit in 0..8 {
                match *byte & 1 << bit {
                    0 => (),
                    _ => self.set_pixel(x + bit, y + row, color),
                }
            }
        }
    }

    fn set_pixel(&self, x: usize, y: usize, color: u32) {
        let frame_buffer = self.get_frame_buffer() as *mut u32;
        let offset = WIDTH * y + x;
        unsafe {
            frame_buffer.add(offset).write_volatile(color);
        }
    }

    fn set_mode(&self) {
        let mut vga = VGA.lock();

        // Some bios mess up the palette when switching modes,
        // so explicitly set it.
        vga.color_palette_registers.load_palette(&DEFAULT_PALETTE);
    }
}
