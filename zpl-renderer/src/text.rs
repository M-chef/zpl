use fontdue::Font;
use tiny_skia::{IntSize, Pixmap, PixmapPaint, Transform};
use zpl_parser::Justification;

use crate::{Drawable, Position};

// tb be closer to zebra font
const ZEBRA_SPACING_CORRECTION: f32 = 0.85;

impl Position {
    pub(crate) fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone)]
pub struct FontConfig {
    font: Font,
    font_width: f32,
    font_height: f32,
    bold: bool,
}

impl FontConfig {
    pub(crate) fn new(font: Font, font_width: f32, font_height: f32, bold: bool) -> Self {
        Self {
            font,
            font_width,
            font_height,
            bold,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct TextFieldProps {
    width: f32,
    min_y: f32,
}

#[derive(Debug, Clone)]
pub struct Text {
    content: String,
    font_config: FontConfig,
    position: Position,
    justification: Justification,
}

impl Text {
    pub(crate) fn new(
        content: String,
        font_config: FontConfig,
        position: Position,
        justification: Justification,
    ) -> Self {
        Self {
            content,
            font_config,
            position,
            justification,
        }
    }

    fn measure_text_dimensions(&self) -> TextFieldProps {
        let width_scale =
            self.font_config.font_width / self.font_config.font_height * ZEBRA_SPACING_CORRECTION;
        let mut total_width = 0.0;
        let mut min_y: f32 = 0.0;

        for ch in self.content.chars() {
            let (metrics, _) = self
                .font_config
                .font
                .rasterize(ch, self.font_config.font_height);
            total_width += metrics.advance_width * width_scale;
            let top = -(metrics.height as f32 + metrics.ymin as f32);
            min_y = min_y.min(top)
        }

        TextFieldProps {
            width: total_width,
            min_y,
        }
    }

    fn adjust_position(&self, textfield_props: &TextFieldProps) -> f32 {
        match self.justification {
            Justification::Left => self.position.x as f32,
            Justification::Right => self.position.x as f32 - textfield_props.width,
            Justification::Auto => self.position.x as f32 - textfield_props.width / 2.0,
        }
    }
}

impl Drawable for Text {
    fn draw(&self, target: &mut tiny_skia::Pixmap) -> Result<(), Box<dyn std::error::Error>> {
        // Calculate text width for justification
        let textfield_props = self.measure_text_dimensions();

        // Adjust starting x position based on justification
        let adjusted_x = self.adjust_position(&textfield_props);

        // Calculate scaling factors
        // Use font_height as the base scale for rasterization
        let base_scale = self.font_config.font_height;
        let width_scale =
            self.font_config.font_width / self.font_config.font_height * ZEBRA_SPACING_CORRECTION;

        let mut pen_x = adjusted_x as f32;
        let pen_y = self.position.y as f32 - textfield_props.min_y;

        // For bold, we'll render multiple times with slight offsets
        let bold_offsets = if self.font_config.bold {
            vec![0.0, 0.4, 0.8]
        } else {
            vec![0.0]
        };

        for ch in self.content.chars() {
            let (metrics, bitmap) = self.font_config.font.rasterize(ch, base_scale);

            if metrics.width == 0 || metrics.height == 0 {
                pen_x += metrics.advance_width; // * width_scale;
                continue;
            }

            // Create an alpha-only pixmap from bitmap
            let w = metrics.width as u32;
            let h = metrics.height as u32;

            for x_offset in &bold_offsets {
                // Construct RGBA buffer with black color and alpha from bitmap
                let mut buf = Vec::with_capacity((w * h * 4) as usize);
                for &alpha in &bitmap {
                    buf.push(0); // R
                    buf.push(0); // G
                    buf.push(0); // B
                    buf.push(alpha); // A
                }

                let glyph_pixmap = match Pixmap::from_vec(
                    buf,
                    IntSize::from_wh(w, h).expect("Invalid glyph dimensions"),
                ) {
                    Some(pm) => pm,
                    None => {
                        pen_x += metrics.advance_width;
                        continue;
                    }
                };

                // Apply transform with width scaling
                let glyph_x = pen_x + metrics.xmin as f32 + x_offset;
                let glyph_y = pen_y - metrics.height as f32 - metrics.ymin as f32;
                let transform =
                    Transform::from_translate(glyph_x, glyph_y).pre_scale(width_scale, 1.0); // Scale width independently
                let pixmap_paint = PixmapPaint::default();

                target.draw_pixmap(0, 0, glyph_pixmap.as_ref(), &pixmap_paint, transform, None);
            }
            pen_x += metrics.advance_width * width_scale;
        }

        Ok(())
    }
}
