use fontdue::Font;
use tiny_skia::{IntSize, Pixmap, PixmapPaint, Transform};
use zpl_parser::Justification;

use crate::{Drawable, Position};

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
    scale: f32,
}

impl FontConfig {
    pub(crate) fn new(
        font: Font,
        font_width: f32,
        font_height: f32,
        scale: f32,
        bold: bool,
    ) -> Self {
        Self {
            font,
            font_width,
            font_height,
            bold,
            scale,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TextFieldProps {
    pub width: f32,
    pub min_y: f32,
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

    pub fn measure_text_dimensions(&self) -> TextFieldProps {
        let width_scale =
            self.font_config.font_width / self.font_config.font_height * self.font_config.scale;
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
            self.font_config.font_width / self.font_config.font_height * self.font_config.scale;

        let mut pen_x = adjusted_x as f32;
        let pen_y = self.position.y as f32 - textfield_props.min_y;

        for ch in self.content.chars() {
            let (metrics, bitmap) = self.font_config.font.rasterize(ch, base_scale);
            let bitmap = if self.font_config.bold {
                // dilate_bitmap(&bitmap, metrics.width, metrics.height)
                dilate_bitmap_hybrid_passes(&bitmap, metrics.width, metrics.height, 1)
            } else {
                bitmap
            };

            if metrics.width == 0 || metrics.height == 0 {
                pen_x += metrics.advance_width;
                continue;
            }

            // Create an alpha-only pixmap from bitmap
            let w = metrics.width as u32;
            let h = metrics.height as u32;

            // for x_offset in &bold_offsets {
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
            let glyph_x = pen_x + metrics.xmin as f32;
            let glyph_y = pen_y - metrics.height as f32 - metrics.ymin as f32;
            let transform = Transform::from_translate(glyph_x, glyph_y).pre_scale(width_scale, 1.0); // Scale width independently
            let pixmap_paint = PixmapPaint::default();

            target.draw_pixmap(0, 0, glyph_pixmap.as_ref(), &pixmap_paint, transform, None);
            // }
            pen_x += metrics.advance_width * width_scale;
        }

        Ok(())
    }
}

fn dilate_bitmap(bitmap: &[u8], width: usize, height: usize) -> Vec<u8> {
    let mut result = vec![0u8; bitmap.len()];

    for y in 0..height {
        for x in 0..width {
            let idx = y * width + x;
            let mut max_alpha = bitmap[idx];

            // Check 8 neighbors + self
            for dy in -1..=1 {
                for dx in -1..=1 {
                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;

                    if nx >= 0 && nx < width as i32 && ny >= 0 && ny < height as i32 {
                        let neighbor_idx = (ny as usize) * width + (nx as usize);
                        max_alpha = max_alpha.max(bitmap[neighbor_idx]);
                    }
                }
            }

            result[idx] = max_alpha;
        }
    }

    result
}

fn dilate_bitmap_hybrid(bitmap: &[u8], width: usize, height: usize) -> Vec<u8> {
    // First pass: Max filter for expansion
    let mut expanded = vec![0u8; bitmap.len()];

    for y in 0..height {
        for x in 0..width {
            let idx = y * width + x;
            let mut max_alpha = bitmap[idx];

            // 4-connected for clean edges
            for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                let nx = x as i32 + dx;
                let ny = y as i32 + dy;

                if nx >= 0 && nx < width as i32 && ny >= 0 && ny < height as i32 {
                    let neighbor_idx = (ny as usize) * width + (nx as usize);
                    max_alpha = max_alpha.max(bitmap[neighbor_idx]);
                }
            }

            expanded[idx] = max_alpha;
        }
    }

    // Second pass: Gentle blur for smooth anti-aliasing
    let mut result = vec![0u8; bitmap.len()];

    for y in 0..height {
        for x in 0..width {
            let idx = y * width + x;
            let mut sum = expanded[idx] as u32 * 4; // Center weight
            let mut count = 4u32;

            for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                let nx = x as i32 + dx;
                let ny = y as i32 + dy;

                if nx >= 0 && nx < width as i32 && ny >= 0 && ny < height as i32 {
                    let neighbor_idx = (ny as usize) * width + (nx as usize);
                    sum += expanded[neighbor_idx] as u32;
                    count += 1;
                }
            }

            result[idx] = (sum / count) as u8;
        }
    }

    result
}

fn dilate_bitmap_hybrid_passes(
    bitmap: &[u8],
    width: usize,
    height: usize,
    passes: usize,
) -> Vec<u8> {
    let mut current = bitmap.to_vec();

    // Multiple expansion passes for consistent boldness
    for _ in 0..passes {
        let mut expanded = vec![0u8; bitmap.len()];

        for y in 0..height {
            for x in 0..width {
                let idx = y * width + x;
                let mut max_alpha = current[idx];

                // 4-connected for clean edges
                for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;

                    if nx >= 0 && nx < width as i32 && ny >= 0 && ny < height as i32 {
                        let neighbor_idx = (ny as usize) * width + (nx as usize);
                        max_alpha = max_alpha.max(current[neighbor_idx]);
                    }
                }

                expanded[idx] = max_alpha;
            }
        }

        current = expanded;
    }

    // Final pass: Gentle blur for smooth anti-aliasing
    let mut result = vec![0u8; bitmap.len()];

    for y in 0..height {
        for x in 0..width {
            let idx = y * width + x;
            let mut sum = current[idx] as u32 * 4; // Center weight
            let mut count = 4u32;

            for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                let nx = x as i32 + dx;
                let ny = y as i32 + dy;

                if nx >= 0 && nx < width as i32 && ny >= 0 && ny < height as i32 {
                    let neighbor_idx = (ny as usize) * width + (nx as usize);
                    sum += current[neighbor_idx] as u32;
                    count += 1;
                }
            }

            result[idx] = (sum / count) as u8;
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use fontdue::Font;

    #[test]
    fn check_font_for_styles() {
        let font_data = std::fs::read("../fonts/AdwaitaSans/AdwaitaSans-Regular.ttf").unwrap();

        // Index 0 is usually regular
        let font_regular = Font::from_bytes(
            font_data.clone(),
            fontdue::FontSettings {
                collection_index: 0,
                ..Default::default()
            },
        )
        .unwrap();

        // Try index 1, 2, 3 etc for other styles
        let font_bold = Font::from_bytes(
            font_data,
            fontdue::FontSettings {
                collection_index: 1,
                ..Default::default()
            },
        )
        .unwrap();
    }
}
