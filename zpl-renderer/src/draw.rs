use std::error::Error;

use intermediate_representation::Color;
use tiny_skia::{
    Color as SkiaColor, IntSize, Mask, Paint, PathBuilder, Pixmap, PixmapPaint, Rect as SkiaRect,
    Stroke, Transform,
};

use crate::{
    elements::{DrawCommand, PositionedGlyph, Rect},
    font::FontStore,
};

pub(crate) trait Drawable {
    fn draw(&self, target: &mut Pixmap, fonts: &FontStore) -> Result<(), Box<dyn Error>>;
    fn draw_inverted(&self, target: &mut Pixmap, fonts: &FontStore) -> Result<(), Box<dyn Error>> {
        // 1. Render field to mask
        let mut mask = Pixmap::new(target.width(), target.height()).ok_or("Invalid dimensions")?;
        self.draw(&mut mask, fonts).unwrap();

        // 2. Apply reverse print against destination
        self.invert_field(target, &mask);

        Ok(())
    }

    fn invert_field(&self, target: &mut Pixmap, mask: &Pixmap) {
        let white_premultiplied = SkiaColor::WHITE.premultiply().to_color_u8();
        let black_premultiplied = SkiaColor::BLACK.premultiply().to_color_u8();

        let h_range = 0..target.height();
        let w_range = 0..target.width();

        for x in w_range {
            for y in h_range.clone() {
                let mask_pixel = mask.pixel(x, y).unwrap();
                let dest_pixel = target.pixel(x, y).unwrap();

                if mask_pixel == black_premultiplied {
                    let dest_color = if dest_pixel == white_premultiplied {
                        black_premultiplied
                    } else {
                        white_premultiplied
                    };

                    let idx = target
                        .width()
                        .checked_mul(y)
                        .unwrap()
                        .checked_add(x)
                        .unwrap() as usize;

                    if let Some(p) = target.pixels_mut().get_mut(idx) {
                        *p = dest_color
                    }
                }
            }
        }
    }
}

impl Drawable for DrawCommand {
    fn draw(
        &self,
        target: &mut Pixmap,
        fonts: &FontStore,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            DrawCommand::Text {
                font,
                font_size,
                glyphs,
                bold,
                inverted,
            } => draw_text(font, *font_size, glyphs, fonts, target),
            DrawCommand::Rectangle {
                bounds,
                line_thickness,
                rounding,
                color,
                inverted,
            } => draw_rectangle(bounds, *line_thickness, *rounding, *color, target),
            DrawCommand::FilledRect {
                bounds,
                rounding,
                color,
                inverted,
            } => draw_filled_rect(bounds, *rounding, *color, target),
            DrawCommand::BitMap { bounds, data } => draw_bitmap(bounds, data, target),
        }
    }
}

fn draw_text(
    font: &String,
    font_size: f32,
    glyphs: &[PositionedGlyph],
    fonts: &FontStore,
    target: &mut Pixmap,
) -> Result<(), Box<dyn Error + 'static>> {
    let loaded_font = fonts.get(font);

    for glyph in glyphs {
        let (metrics, bitmap) = loaded_font
            .font
            .rasterize_indexed(glyph.glyph_index, font_size);
        let width = metrics.width;
        let height = metrics.height;

        // Whitespace glyphs have no bitmap — nothing to draw, skip them.
        if width == 0 || height == 0 {
            continue;
        }

        // glyph.y is the baseline; offset upward by height+ymin to get the bitmap top-left
        let draw_y = (glyph.y - height as f32 - metrics.ymin as f32).round() as i32;

        let data = {
            let mut buf = Vec::with_capacity((width * height * 4) as usize);
            for alpha in bitmap {
                buf.push(0);
                buf.push(0);
                buf.push(0);
                buf.push(alpha);
            }
            buf
        };
        let size = IntSize::from_wh(width as u32, height as u32).ok_or("Invalid size")?;
        let glyph_pixmap = Pixmap::from_vec(data, size).ok_or("Data not matching size")?;
        target.draw_pixmap(
            glyph.x.round() as i32,
            draw_y,
            glyph_pixmap.as_ref(),
            &PixmapPaint::default(),
            Transform::identity(),
            None,
        );
    }

    Ok(())
}

fn draw_bitmap(
    bound: &Rect,
    data: &[u8],
    target: &mut Pixmap,
) -> Result<(), Box<dyn Error + 'static>> {
    // Create a pixmap with the bitmap content
    let mut bitmap_pixmap =
        Pixmap::new(bound.width as u32, bound.height as u32).ok_or("Invalid size for bitmap")?;

    // Fill with black (or whatever color you want for the "1" pixels)
    bitmap_pixmap.fill(SkiaColor::BLACK);

    // Create and apply mask (0 = transparent, 255 = opaque)
    let mut mask =
        Mask::new(bound.width as u32, bound.height as u32).ok_or("Invalid size for mask")?;
    for (i, &pixel) in data.iter().enumerate() {
        // ZPL: 0 = white (transparent), 1 = black (opaque)
        mask.data_mut()[i] = if pixel == 1 { 255 } else { 0 };
    }
    bitmap_pixmap.apply_mask(&mask);

    // Draw the bitmap onto the target at position (x, y)
    target.draw_pixmap(
        bound.x as i32,
        bound.y as i32,
        bitmap_pixmap.as_ref(),
        &PixmapPaint::default(),
        Transform::identity(),
        None,
    );
    Ok(())
}

fn draw_rectangle(
    bounds: &Rect,
    line_thickness: f32,
    rounding: f32,
    color: Color,
    target: &mut Pixmap,
) -> Result<(), Box<dyn Error>> {
    let rect = SkiaRect::from_xywh(bounds.x, bounds.y, bounds.width, bounds.height)
        .ok_or("Invalid rect bounds")?;
    let inset = line_thickness / 2.0;

    // thickness from zpl is not equal to stroke width
    // for thickness value equally to width and height this would lead
    // to a single point rect (i.e. (x: 1, y: 1, widh: 1, height: 1), not be drawn)
    // hence we must correct the inset in such cases
    // TODO: catch this earlier in the interpreter
    let inset = match inset >= bounds.width / 2. && inset >= bounds.height / 2. {
        true => inset - 0.1,
        false => inset,
    };
    let rect = rect.inset(inset, inset).unwrap();
    let path = PathBuilder::from_rect(rect);

    let mut paint = Paint::default();
    match color {
        Color::Black => paint.set_color_rgba8(0, 0, 0, 255),
        Color::White => paint.set_color_rgba8(255, 255, 255, 255),
    }

    let mut stroke = Stroke::default();
    stroke.width = line_thickness;

    target.stroke_path(&path, &paint, &stroke, Transform::identity(), None);

    Ok(())
}

fn draw_filled_rect(
    bounds: &Rect,
    rounding: f32,
    color: Color,
    target: &mut Pixmap,
) -> Result<(), Box<dyn Error>> {
    let rect = SkiaRect::from_xywh(bounds.x, bounds.y, bounds.width, bounds.height)
        .ok_or("Invalid rect bounds")?;

    let mut paint = Paint::default();
    match color {
        Color::Black => paint.set_color_rgba8(0, 0, 0, 255),
        Color::White => paint.set_color_rgba8(255, 255, 255, 255),
    }

    target.fill_rect(rect, &paint, Transform::identity(), None);

    Ok(())
}

#[cfg(test)]
mod tests {
    use intermediate_representation::{ADWAITA_MONO, Color};
    use tiny_skia::Pixmap;

    use crate::{
        draw::Drawable,
        elements::{DrawCommand, PositionedGlyph, Rect},
        font::FontStore,
    };

    #[test]
    fn should_draw_glyph() {
        let fonts = FontStore::load_defaults();
        let font_size = 40.;
        let character = 'T';
        let loaded_font = fonts.get(ADWAITA_MONO);
        let glyph_index = loaded_font.font.lookup_glyph_index(character);

        let metrics = loaded_font.font.metrics_indexed(glyph_index, font_size);
        let baseline_y = metrics.height as f32 + metrics.ymin as f32; // exact top = pixel 0

        let glyph = PositionedGlyph {
            glyph_index,
            x: 0.,
            y: baseline_y,
            advance_width: metrics.advance_width,
        };

        let text = DrawCommand::Text {
            font: ADWAITA_MONO.to_string(),
            font_size,
            glyphs: vec![glyph],
            bold: false,
            inverted: false,
        };

        let mut pixmap = Pixmap::new(metrics.width as u32, metrics.height as u32).unwrap();
        text.draw(&mut pixmap, &fonts).unwrap();
        pixmap.save_png("text.png").unwrap();
    }

    #[test]
    fn should_draw_bitmap() {
        let data: Vec<u8> = vec![0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1];
        let mut target = Pixmap::new(4, 4).unwrap();

        let bounds = Rect {
            x: 0.,
            y: 0.,
            width: 4.,
            height: 4.,
        };

        let bitmap = DrawCommand::BitMap { bounds, data };

        let fonts = FontStore::load_defaults();
        bitmap.draw(&mut target, &fonts).unwrap();
        target.save_png("bitmap.png").unwrap();
    }

    #[test]
    fn should_draw_rect() {
        let mut target = Pixmap::new(20, 20).unwrap();

        let bounds = Rect {
            x: 0.,
            y: 0.,
            width: 20.,
            height: 20.,
        };

        let rect = DrawCommand::FilledRect {
            bounds,
            rounding: 0.,
            color: Color::Black,
            inverted: false,
        };

        let fonts = FontStore::load_defaults();
        rect.draw(&mut target, &fonts).unwrap();
        target.save_png("filled_rect.png").unwrap();
    }
}
