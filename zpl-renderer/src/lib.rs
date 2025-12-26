mod bitmap;
mod shapes;
mod text;

use std::error::Error;

use fontdue::Font;
use tiny_skia::{Color, Mask, Pixmap, PixmapPaint, Transform};
use zpl_interpreter::{DecodedBitmap, ZplElement, ZplLabel};
use zpl_parser::Justification;

use crate::{
    bitmap::BitMap,
    shapes::{RectDim, Rectangle},
    text::{FontConfig, Text},
};

#[derive(Debug, Clone, Copy)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

pub(crate) trait Drawable {
    fn draw(&self, target: &mut Pixmap) -> Result<(), Box<dyn Error>>;
    fn draw_inverted(&self, target: &mut Pixmap) {
        // 1. Render field to mask
        let mut mask = Pixmap::new(target.width(), target.height()).unwrap();
        self.draw(&mut mask).unwrap();

        // 2. Apply reverse print against destination
        self.invert_field(target, &mask);
    }

    fn invert_field(&self, target: &mut Pixmap, mask: &Pixmap) {
        let white_premultiplied = Color::WHITE.premultiply().to_color_u8();
        let black_premultiplied = Color::BLACK.premultiply().to_color_u8();

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
                let position = Position::new(*x as usize, *y as usize);
                let font_config = FontConfig::new(font.clone(), *font_width, *font_height, false);
                let text = Text::new(content.clone(), font_config, position, *justification);
                if *inverted {
                    text.draw_inverted(&mut pixmap);
                } else {
                    text.draw(&mut pixmap).unwrap();
                }
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
            } => {
                let position = Position::new(*x as usize, *y as usize);
                let dim = RectDim::new(*width as f32, *height as f32, *thickness as f32, *rounding);
                let rectangle = Rectangle::new(position, dim, *color);
                if *inverted {
                    rectangle.draw_inverted(&mut pixmap);
                } else {
                    rectangle.draw(&mut pixmap).unwrap();
                }
            }
            ZplElement::Image { x, y, bmp } => {
                let position = Position::new(*x, *y);
                let pixels = bmp.pixels.clone();
                let bitmap = BitMap::new(position, bmp.width as u32, bmp.height as u32, pixels);
                bitmap.draw(&mut pixmap).unwrap();
            }
            ZplElement::Barcode {
                x,
                y,
                content,
                bitmap,
            } => {
                let position = Position::new(*x, *y);
                let pixels = bitmap.pixels.clone();
                let bitmap =
                    BitMap::new(position, bitmap.width as u32, bitmap.height as u32, pixels);
                bitmap.draw(&mut pixmap).unwrap();

                if let Some(content) = content {
                    let font_width = content.font_width;
                    let font_height = font_width;
                    let font_config = FontConfig::new(font.clone(), font_width, font_height, false);
                    let position = Position::new(content.x as usize, content.y as usize);
                    let text = Text::new(
                        content.text.clone(),
                        dbg!(font_config),
                        position,
                        Justification::Left,
                    );
                    text.draw(&mut pixmap).unwrap();
                }
            }
        }
    }

    let png = pixmap.encode_png().expect("encode png");
    RenderOutput { png }
}
