use tiny_skia::{Color, Mask, Pixmap, PixmapPaint, Transform};

use crate::{Drawable, Position};

#[derive(Debug)]
pub(crate) struct BitMap {
    position: Position,
    width: u32,
    height: u32,
    pixels: Vec<u8>,
}

impl BitMap {
    pub(crate) fn new(position: Position, width: u32, height: u32, pixels: Vec<u8>) -> Self {
        Self {
            position,
            width,
            height,
            pixels,
        }
    }
}

impl Drawable for BitMap {
    fn draw(&self, target: &mut tiny_skia::Pixmap) -> Result<(), Box<dyn std::error::Error>> {
        // Create a pixmap with the bitmap content
        let mut bitmap_pixmap = Pixmap::new(self.width, self.height).unwrap();

        // Fill with black (or whatever color you want for the "1" pixels)
        bitmap_pixmap.fill(Color::BLACK);

        // Create and apply mask (0 = transparent, 255 = opaque)
        let mut mask = Mask::new(self.width, self.height).unwrap();
        for (i, &pixel) in self.pixels.iter().enumerate() {
            // ZPL: 0 = white (transparent), 1 = black (opaque)
            mask.data_mut()[i] = if pixel == 1 { 255 } else { 0 };
        }
        bitmap_pixmap.apply_mask(&mask);

        // Draw the bitmap onto the target at position (x, y)
        target.draw_pixmap(
            self.position.x as i32,
            self.position.y as i32,
            bitmap_pixmap.as_ref(),
            &PixmapPaint::default(),
            Transform::identity(),
            None,
        );
        Ok(())
    }
}
