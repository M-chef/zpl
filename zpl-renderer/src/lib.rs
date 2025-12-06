use fontdue::Font;
use tiny_skia::{Color, IntSize, Mask, Pixmap, PixmapPaint, Transform};
use zpl_interpreter::{DecodedBitmap, ZplElement, ZplLabel};
use zpl_parser::Justification;

// tb be closer to zebra font
const ZEBRA_SPACING_CORRECTION: f32 = 0.9;

pub struct RenderOutput {
    pub png: Vec<u8>,
}

pub fn render(label: &ZplLabel) -> RenderOutput {
    // Create a pixmap
    let width = label.width as u32;
    let height = label.height as u32;
    let mut pixmap = Pixmap::new(width, height).expect("Failed to create pixmap");

    // White background
    pixmap.fill(Color::from_rgba8(255, 255, 255, 255));

    // Load a TTF font from bytes. For demo purposes we use include_bytes!; replace with your chosen font.
    // This example expects a file at "assets/DejaVuSans.ttf" or you can change to any TTF you have.
    let font_data: &'static [u8] = include_bytes!("../../fonts/AdwaitaSans-Regular.ttf");
    let font = Font::from_bytes(font_data as &[u8], fontdue::FontSettings::default()).unwrap();

    for el in &label.elements {
        match el {
            ZplElement::Text {
                x,
                y,
                font_width,
                font_height,
                content,
                justification,
            } => {
                draw_text_rasterized(
                    &mut pixmap,
                    &font,
                    *font_height,
                    *font_width,
                    *x,
                    *y,
                    content,
                    *justification,
                    true,
                );
            }
            ZplElement::Image { x, y, bmp } => {
                draw_bitmap(&mut pixmap, bmp, *x, *y);
            }
        }
    }

    let png = pixmap.encode_png().expect("encode png");
    RenderOutput { png }
}

fn measure_text_width(font: &Font, font_height: f32, font_width: f32, text: &str) -> f32 {
    let width_scale = dbg!(font_width / font_height * ZEBRA_SPACING_CORRECTION);
    let mut total_width = 0.0;

    for ch in text.chars() {
        let (metrics, _) = font.rasterize(ch, font_height);
        total_width += metrics.advance_width * width_scale;
    }

    total_width
}

fn draw_text_rasterized(
    pixmap: &mut Pixmap,
    font: &Font,
    font_height: f32,
    font_width: f32,
    x: i32,
    y: i32,
    text: &str,
    justification: Justification,
    bold: bool,
) {
    // Calculate text width for justification
    let text_width = measure_text_width(font, font_height, font_width, text);

    // Adjust starting x position based on justification
    let adjusted_x = match justification {
        Justification::Left => x as f32,
        Justification::Right => x as f32 - text_width,
        Justification::Auto => x as f32 - text_width / 2.0,
    };

    // Calculate scaling factors
    // Use font_height as the base scale for rasterization
    let base_scale = font_height;
    let width_scale = dbg!(font_width / font_height * ZEBRA_SPACING_CORRECTION);

    let mut pen_x = adjusted_x as f32;
    let pen_y = y as f32;

    // For bold, we'll render multiple times with slight offsets
    let bold_offsets = if bold { vec![0.0, 0.4, 0.8] } else { vec![0.0] };

    for ch in text.chars() {
        let (metrics, bitmap) = font.rasterize(ch, base_scale);

        if metrics.width == 0 || metrics.height == 0 {
            pen_x += metrics.advance_width * width_scale;
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
                    pen_x += metrics.advance_width * width_scale;
                    continue;
                }
            };

            // Apply transform with width scaling
            let glyph_x = pen_x + metrics.xmin as f32 * width_scale + x_offset;
            let glyph_y = pen_y - (metrics.height as f32 + metrics.ymin as f32);
            let transform = Transform::from_translate(glyph_x, glyph_y).pre_scale(width_scale, 1.0); // Scale width independently

            pixmap.draw_pixmap(
                0,
                0,
                glyph_pixmap.as_ref(),
                &PixmapPaint::default(),
                transform,
                None,
            );
        }
        pen_x += metrics.advance_width * width_scale;
    }
}

pub fn draw_bitmap(target: &mut Pixmap, bmp: &DecodedBitmap, x: i32, y: i32) {
    let width = bmp.width as u32;
    let height = bmp.height as u32;

    // Create a pixmap with the bitmap content
    let mut bitmap_pixmap = Pixmap::new(width, height).unwrap();

    // Fill with black (or whatever color you want for the "1" pixels)
    bitmap_pixmap.fill(Color::BLACK);

    // Create and apply mask (0 = transparent, 255 = opaque)
    let mut mask = Mask::new(width, height).unwrap();
    for (i, &pixel) in bmp.pixels.iter().enumerate() {
        // ZPL: 0 = white (transparent), 1 = black (opaque)
        mask.data_mut()[i] = if pixel == 1 { 255 } else { 0 };
    }
    bitmap_pixmap.apply_mask(&mask);

    // Draw the bitmap onto the target at position (x, y)
    target.draw_pixmap(
        x,
        y,
        bitmap_pixmap.as_ref(),
        &PixmapPaint::default(),
        Transform::identity(),
        None,
    );
}
