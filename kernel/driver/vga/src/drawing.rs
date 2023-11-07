use bresenham::Bresenham;
use graphics::{PrimitiveDrawing, Vec2};

use crate::{Color, Vga1280x800, HEIGHT, WIDTH};

impl PrimitiveDrawing<Color> for Vga1280x800 {
    fn screen_size(&self) -> Vec2 {
        static SCREEN_SIZE: Vec2 = Vec2 {
            x: WIDTH,
            y: HEIGHT,
        };
        SCREEN_SIZE
    }

    fn set_pixel(&mut self, point: Vec2, color: Color) {
        let pixel = color.into();
        let offset = self.compute_frame_buffer_offset(point);
        self.frame_buffer[offset] = pixel;
    }

    fn clear_screen(&mut self, color: Color) {
        self.frame_buffer.fill(color.into());
    }

    fn draw_line(&mut self, start: Vec2, end: Vec2, color: Color) {
        if start > end {
            return self.draw_line(end, start, color);
        }

        if start.y == end.y {
            let line_width = start.x.abs_diff(end.x);
            let offset = self.compute_frame_buffer_offset(start);
            self.frame_buffer[offset..offset + line_width].fill(color.into());
            return;
        }

        for (x, y) in Bresenham::new(start.into(), end.into()) {
            self.set_pixel((x, y).into(), color);
        }
    }
}
