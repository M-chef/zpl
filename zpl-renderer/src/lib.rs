use fontdue::Font;
use tiny_skia::{Color, IntSize, Mask, Pixmap, PixmapPaint, Transform};
use zpl_interpreter::{DecodedBitmap, ZplElement, ZplLabel};

pub struct RenderOutput {
    pub png: Vec<u8>,
}

pub fn render(label: &ZplLabel) -> RenderOutput {
    // Create a pixmap
    let mut pixmap =
        Pixmap::new(label.width as u32, label.height as u32).expect("Failed to create pixmap");

    // White background
    pixmap.fill(Color::from_rgba8(255, 255, 255, 255));

    // Load a TTF font from bytes. For demo purposes we use include_bytes!; replace with your chosen font.
    // This example expects a file at "assets/DejaVuSans.ttf" or you can change to any TTF you have.
    let font_data: &'static [u8] = include_bytes!("../../fonts/arial.ttf");
    let font = Font::from_bytes(font_data as &[u8], fontdue::FontSettings::default()).unwrap();

    for el in &label.elements {
        match el {
            ZplElement::Text {
                x,
                y,
                font_size,
                content,
            } => {
                draw_text_rasterized(&mut pixmap, &font, *font_size, *x, *y, content);
            }
            ZplElement::Image { x, y, bmp } => {
                draw_bitmap(&mut pixmap, bmp, *x, *y);
            }
        }
    }

    let png = pixmap.encode_png().expect("encode png");
    RenderOutput { png }
}

fn draw_text_rasterized(pixmap: &mut Pixmap, font: &Font, size: f32, x: i32, y: i32, text: &str) {
    // naive horizontal layout: iterate through chars, rasterize each glyph and blit
    let scale = size;
    let mut pen_x = x;
    let pen_y = y;

    for ch in text.chars() {
        let (metrics, bitmap) = font.rasterize(ch, scale);
        if metrics.width == 0 || metrics.height == 0 {
            pen_x += metrics.advance_width.round() as i32;
            continue;
        }

        // Create an alpha-only pixmap from bitmap
        let w = metrics.width as u32;
        let h = metrics.height as u32;

        // tiny-skia expects RGBA pixels; we will construct an RGBA buffer with black color and alpha from bitmap
        let mut buf = vec![0u8; (w * h * 4) as usize];
        for row in 0..h {
            for col in 0..w {
                let idx = (row * w + col) as usize;
                let a = bitmap[row as usize * metrics.width + col as usize];
                let base = idx * 4;
                buf[base + 0] = 0; // R
                buf[base + 1] = 0; // G
                buf[base + 2] = 0; // B
                buf[base + 3] = a; // A
            }
        }

        // Paint this glyph onto the main pixmap
        let glyph_pixmap = Pixmap::from_vec(
            buf,
            IntSize::from_wh(w, h).expect("Failed to create pixmap"),
        )
        .expect("make glyph pixmap");
        let transform = Transform::from_translate(
            pen_x as f32 + metrics.xmin as f32,
            pen_y as f32 - (metrics.height as i32 as f32 + metrics.ymin as f32),
        );
        pixmap.draw_pixmap(
            0,
            0,
            glyph_pixmap.as_ref(),
            &PixmapPaint::default(),
            transform,
            None,
        );

        pen_x += metrics.advance_width.round() as i32;
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
