use crate::{Color, Vec2};
use bresenham::Bresenham;

/// This component can draw primitive shapes on the screen.
pub trait PrimitiveDrawing<C>
where
    C: Color,
{
    /// Returns the screen size of the screen that this component
    /// is responsible for.
    fn screen_size(&self) -> Vec2;

    /// Draws a single pixel on the screen with the given color.
    fn set_pixel(&mut self, point: Vec2, color: C);

    /// Fills the full screen with the given color.
    fn clear_screen(&mut self, color: C) {
        self.fill_rect(Vec2::ZERO, self.screen_size(), color)
    }

    /// Draws a line from `start` to `end` with the given `color`.
    ///
    /// Note:
    /// Both points may be located anywhere on the screen independent of each other.
    fn draw_line(&mut self, start: Vec2, end: Vec2, color: C) {
        if start > end {
            return self.draw_line(end, start, color);
        }

        for (x, y) in Bresenham::new(start.into(), end.into()) {
            self.set_pixel((x, y).into(), color);
        }
    }

    /// Draws a rectangle from `p1` to `p2` and fills it with the given `color`.
    ///
    /// Drawing operations may be buffered and not visible until [`PrimitiveDrawing::flush`] is called.
    ///
    /// Note:
    /// Both points may be located anywhere on the screen independent of each other.
    fn fill_rect(&mut self, p1: Vec2, p2: Vec2, color: C) {
        if p1 > p2 {
            return self.fill_rect(p2, p1, color);
        }

        for y in p1.y..p2.y {
            // Drawing lines is likely more efficient than plotting single pixels.
            // This is especially true as soon as an implementation provides a more
            // specialized method for drawing lines.
            self.draw_line((p1.x, y).into(), (p2.x, y).into(), color);
        }
    }

    fn fill_triangle(&mut self, p1: Vec2, p2: Vec2, p3: Vec2, color: C) {
        // sort points
        let mut pts = [p1, p2, p3];
        pts.sort_by(|l, r| l.y.cmp(&r.y));

        let (p1, p2, p3) = (pts[0], pts[1], pts[2]);

        if p2.y == p3.y {
            self.triangle_bottom_flat(p1, p2, p3, color);
        } else if p1.y == p2.y {
            self.triangle_top_flat(p1, p2, p3, color);
        } else {
            /* general case - split the triangle in a topflat and bottom-flat one */
            let p4 = Vec2 {
                x: p1.x
                    + (((p2.y as f64 - p1.y as f64) / (p3.y as f64 - p1.y as f64))
                        * (p3.x as f64 - p1.x as f64)) as usize,
                y: p2.y,
            };
            self.triangle_bottom_flat(p1, p2, p4, color);
            self.triangle_top_flat(p2, p4, p3, color);
        }
    }

    /// Flushes all buffered draw operations and makes sure that they are actually visible.
    /// Depending on the specific implementation, this may or may not be necessary to call
    /// to show **any** changes on the screen. Regardless, the caller should **always** call
    /// this at the end of a bunch of drawing operations (batching).
    fn flush(&mut self) {
        // no-op by default
    }
}

trait TriangleDrawing<C>: PrimitiveDrawing<C>
where
    C: Color,
{
    fn triangle_top_flat(&mut self, p1: Vec2, p2: Vec2, p3: Vec2, color: C) {
        let invslope1 = (p3.x as f64 - p1.x as f64) / (p3.y as f64 - p1.y as f64);
        let invslope2 = (p3.x as f64 - p2.x as f64) / (p3.y as f64 - p2.y as f64);

        let mut curx1 = p3.x as f64;
        let mut curx2 = p3.x as f64;

        for scanline_y in (p1.y..=p3.y).rev() {
            self.draw_line(
                (curx1 as usize, scanline_y).into(),
                (curx2 as usize, scanline_y).into(),
                color,
            );
            curx1 -= invslope1;
            curx2 -= invslope2;
        }
    }

    fn triangle_bottom_flat(&mut self, p1: Vec2, p2: Vec2, p3: Vec2, color: C) {
        let invslope1 = (p2.x as f64 - p1.x as f64) / (p2.y as f64 - p1.y as f64);
        let invslope2 = (p3.x as f64 - p1.x as f64) / (p3.y as f64 - p1.y as f64);

        let mut curx1 = p1.x as f64;
        let mut curx2 = p1.x as f64;

        for scanline_y in p1.y..=p2.y {
            self.draw_line(
                (curx1 as usize, scanline_y).into(),
                (curx2 as usize, scanline_y).into(),
                color,
            );
            curx1 += invslope1;
            curx2 += invslope2;
        }
    }
}

impl<T, C> TriangleDrawing<C> for T
where
    T: ?Sized + PrimitiveDrawing<C>,
    C: Color,
{
}
