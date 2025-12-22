use std::iter;

use fontdue::Font;
use tiny_skia::{
    Color, ColorU8, IntSize, Mask, Paint, PathBuilder, Pixmap, PixmapPaint, PremultipliedColorU8,
    Rect, Stroke, Transform,
};
use zpl_interpreter::{DecodedBitmap, ZplElement, ZplLabel};
use zpl_parser::{Color as ZplColor, Justification};

// tb be closer to zebra font
const ZEBRA_SPACING_CORRECTION: f32 = 0.85;

pub struct RenderOutput {
    pub png: Vec<u8>,
}

pub fn render(label: &ZplLabel) -> RenderOutput {
    // Create a pixmap
    let width = label.width as u32;
    let height = label.height as u32;
    let mut pixmap = Pixmap::new(width, height).expect("Failed to create pixmap");

    // White background
    pixmap.fill(Color::WHITE);

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
                inverted,
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
                    false,
                    *inverted,
                );
            }
            ZplElement::Rectangle {
                x,
                y,
                width,
                height,
                thickness,
                color,
                rounding,
                inverted,
            } => draw_rectangle(
                &mut pixmap,
                *x as f32,
                *y as f32,
                *width as f32,
                *height as f32,
                *thickness as f32,
                *color,
                *rounding,
                *inverted,
            ),
            ZplElement::Image { x, y, bmp } => {
                draw_bitmap(&mut pixmap, bmp, *x, *y);
            }
        }
    }

    let png = pixmap.encode_png().expect("encode png");
    RenderOutput { png }
}

pub struct TextFieldProps {
    width: f32,
    min_y: f32,
}

fn measure_text_width(
    font: &Font,
    font_height: f32,
    font_width: f32,
    text: &str,
) -> TextFieldProps {
    let width_scale = font_width / font_height * ZEBRA_SPACING_CORRECTION;
    let mut total_width = 0.0;
    let mut min_y: f32 = 0.0;

    for ch in text.chars() {
        let (metrics, _) = font.rasterize(ch, font_height);
        total_width += metrics.advance_width * width_scale;
        let top = -(metrics.height as f32 + metrics.ymin as f32);
        min_y = min_y.min(top)
    }

    TextFieldProps {
        width: total_width,
        min_y,
    }
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
    inverted: bool,
) {
    // Calculate text width for justification
    let textfield_props = measure_text_width(font, font_height, font_width, text);

    // Adjust starting x position based on justification
    let adjusted_x = match justification {
        Justification::Left => x as f32,
        Justification::Right => x as f32 - textfield_props.width,
        Justification::Auto => x as f32 - textfield_props.width / 2.0,
    };

    // Calculate scaling factors
    // Use font_height as the base scale for rasterization
    let base_scale = font_height;
    let width_scale = font_width / font_height * ZEBRA_SPACING_CORRECTION;

    let mut pen_x = adjusted_x as f32;
    let pen_y = y as f32 - textfield_props.min_y;

    // For bold, we'll render multiple times with slight offsets
    let bold_offsets = if bold { vec![0.0, 0.4, 0.8] } else { vec![0.0] };

    for ch in text.chars() {
        let (metrics, bitmap) = font.rasterize(ch, base_scale);

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
                    pen_x += metrics.advance_width; // * width_scale;
                    continue;
                }
            };

            // if inverted {
            //     draw_field_with_fr(pixmap, x, y, draw_fn);
            // }

            // Apply transform with width scaling
            let glyph_x = pen_x + metrics.xmin as f32 + x_offset; //* width_scale + x_offset;
            let glyph_y = pen_y - metrics.height as f32 - metrics.ymin as f32;
            // let glyph_y = pen_y + metrics.ascent - metrics.ymin as f32;
            let transform = Transform::from_translate(glyph_x, glyph_y).pre_scale(width_scale, 1.0); // Scale width independently

            if inverted {
                draw_field_with_fr(pixmap, x, y, |pm| {
                    pm.draw_pixmap(
                        // metrics.height.strict_neg() as i32,
                        0,
                        0,
                        glyph_pixmap.as_ref(),
                        &PixmapPaint::default(),
                        transform,
                        None,
                    );
                });
            } else {
                pixmap.draw_pixmap(
                    0,
                    0,
                    glyph_pixmap.as_ref(),
                    &PixmapPaint::default(),
                    transform,
                    None,
                );
            }

            //     if inverted {
            //         draw_field_with_fr(
            //             pixmap,
            //             glyph_x as i32,
            //             glyph_y as i32,
            //             metrics.width as u32,
            //             metrics.height as u32,
            //             |pm| {
            //                 pm.draw_pixmap(
            //                     0,
            //                     0,
            //                     glyph_pixmap.as_ref(),
            //                     &PixmapPaint::default(),
            //                     transform,
            //                     None,
            //                 );
            //             },
            //         );
            //     } else {
            //         pixmap.draw_pixmap(
            //             0,
            //             0,
            //             glyph_pixmap.as_ref(),
            //             &PixmapPaint::default(),
            //             transform,
            //             None,
            //         );
            //     }
        }
        pen_x += metrics.advance_width * width_scale;
    }
}

fn draw_bitmap(target: &mut Pixmap, bmp: &DecodedBitmap, x: i32, y: i32) {
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

fn draw_rectangle(
    pixmap: &mut Pixmap,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    thickness: f32,
    zpl_color: ZplColor,
    rounding: u8,
    inverted: bool,
) {
    let rect = Rect::from_xywh(x, y, width, height).unwrap();
    let inset = thickness / 2.0;

    // thickness from zpl is not equal to stroke width
    // for thickness value equally to width and height this would lead
    // to a single point rect (i.e. (x: 1, y: 1, widh: 1, height: 1), not be drawn)
    // hence we must correct the inset in such cases
    // TODO: catch this earlier in the interpreter
    let inset = match inset >= width / 2. && inset >= height / 2. {
        true => inset - 0.1,
        false => inset,
    };
    let rect = rect.inset(inset, inset).unwrap();

    let mut pb = PathBuilder::new();
    pb.push_rect(rect);
    let path = pb.finish().unwrap();

    let mut paint = Paint::default();
    match zpl_color {
        ZplColor::Black => paint.set_color_rgba8(0, 0, 0, 255),
        ZplColor::White => paint.set_color_rgba8(255, 255, 255, 255),
    }

    let mut stroke = Stroke::default();
    stroke.width = thickness;

    if inverted {
        draw_field_with_fr(pixmap, x as i32, y as i32, |pm| {
            pm.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
        });
    } else {
        pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
    }
}

fn draw_field_with_fr(target: &mut Pixmap, x: i32, y: i32, draw_fn: impl FnOnce(&mut Pixmap)) {
    // 1. Render field to mask
    let mut mask = Pixmap::new(target.width(), target.height()).unwrap();
    draw_fn(&mut mask);

    // 2. Apply reverse print against destination
    invert_field(target, &mask, x, y);
}

fn invert_field(target: &mut Pixmap, mask: &Pixmap, x: i32, y: i32) {
    let white_premultiplied = Color::WHITE.premultiply().to_color_u8();
    let black_premultiplied = Color::BLACK.premultiply().to_color_u8();

    let h_range = 0..target.height();
    let w_range = 0..target.width();

    for x in w_range {
        for y in h_range.clone() {
            let mask_pixel = mask.pixel(x as u32, y as u32).unwrap();
            let dest_pixel = target.pixel(x as u32, y as u32).unwrap();

            if mask_pixel == black_premultiplied {
                let dest_color = if dest_pixel == white_premultiplied {
                    black_premultiplied
                } else {
                    white_premultiplied
                };

                let idx = target
                    .width()
                    .checked_mul(y as u32)
                    .unwrap()
                    .checked_add(x as u32)
                    .unwrap() as usize;

                if let Some(p) = target.pixels_mut().get_mut(idx) {
                    *p = dest_color
                }
            }
        }
    }
}
//     }
// }
