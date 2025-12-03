use fontdue::Font;
use tiny_skia::{Color, IntSize, Pixmap, PixmapPaint, Transform};
use zpl_interpreter::ZplElement;

pub struct RenderOutput {
    pub png: Vec<u8>,
}

pub fn render(elements: &[ZplElement], width: u32, height: u32) -> RenderOutput {
    // Create a pixmap
    let mut pixmap = Pixmap::new(width, height).expect("Failed to create pixmap");

    // White background
    pixmap.fill(Color::from_rgba8(255, 255, 255, 255));

    // Load a TTF font from bytes. For demo purposes we use include_bytes!; replace with your chosen font.
    // This example expects a file at "assets/DejaVuSans.ttf" or you can change to any TTF you have.
    let font_data: &'static [u8] = include_bytes!("../../fonts/arial.ttf");
    let font = Font::from_bytes(font_data as &[u8], fontdue::FontSettings::default()).unwrap();

    println!("{}", elements.len());
    for el in elements {
        println!("{el:?}");
        match el {
            ZplElement::Text {
                x,
                y,
                font_size,
                content,
            } => {
                draw_text_rasterized(
                    &mut pixmap,
                    &font,
                    *font_size,
                    *x as i32,
                    *y as i32,
                    content,
                );
            }
            ZplElement::Image { x, y, content } => todo!(),
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
