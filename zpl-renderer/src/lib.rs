mod bitmap;
mod shapes;
mod text;

use std::{collections::HashMap, error::Error};

use fontdue::Font;
use tiny_skia::{Color, Pixmap};
use zpl_interpreter::{ZplElement, ZplLabel};
use zpl_parser::{Color as ZplColor, Justification};

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

    // Load a TTF font from bytes.
    let mut font_data = HashMap::new();
    let adwaita: &'static [u8] = include_bytes!("../../fonts/AdwaitaSans/AdwaitaSans-Bold.ttf");
    let font = Font::from_bytes(adwaita as &[u8], fontdue::FontSettings::default()).unwrap();
    let scale = 0.85;
    font_data.insert('0', (font, scale));

    let ocrb: &'static [u8] = include_bytes!("../../fonts/AdwaitaMono/AdwaitaMono-Regular.ttf");
    let font = Font::from_bytes(ocrb as &[u8], fontdue::FontSettings::default()).unwrap();
    let scale = 1.0;
    font_data.insert('A', (font, scale));

    let ocrb: &'static [u8] = include_bytes!("../../fonts/OCRB/OCR-B.ttf");
    let font = Font::from_bytes(ocrb as &[u8], fontdue::FontSettings::default()).unwrap();
    let scale = 1.0;
    font_data.insert(';', (font, scale));

    for el in &label.elements {
        match el {
            ZplElement::Text {
                x,
                y,
                font_name,
                font_width,
                font_height,
                content,
                justification,
                inverted,
            } => {
                let position = Position::new(*x as usize, *y as usize);
                let (font, scale) = font_data.get(font_name).unwrap().clone();
                let font_config = FontConfig::new(font, *font_width, *font_height, scale, false);
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
            ZplElement::Barcode { x, y, content } => {
                let position = Position::new(*x, *y);
                let bitmap = &content.bitmap;
                let pixels = bitmap.pixels.clone();
                let bitmap =
                    BitMap::new(position, bitmap.width as u32, bitmap.height as u32, pixels);
                bitmap.draw(&mut pixmap).unwrap();

                for text_element in content.text_elements() {
                    let font_width = content.font_width;
                    let font_height = font_width;
                    let (font, scale) = font_data.get(&';').unwrap().clone();
                    let font_config = FontConfig::new(font, font_width, font_height, scale, false);
                    let position =
                        Position::new(text_element.text_x as usize, text_element.text_y as usize);
                    let text = Text::new(
                        text_element.text.clone(),
                        font_config,
                        position,
                        text_element.justification,
                    );

                    let rect_width = text.measure_text_dimensions().width;
                    let rect_height = font_height * 1.2;
                    let line_thickness = rect_height.min(rect_width) - 0.1;
                    let dim = RectDim::new(rect_width, rect_height, line_thickness, 0);
                    let rect_pos = {
                        let mut y = position.y as f32;
                        y = y - rect_height / 7.;
                        let x = match text_element.justification {
                            Justification::Left => position.x,
                            Justification::Right => todo!(),
                            Justification::Auto => position.x - rect_width as usize / 2,
                        };
                        Position::new(x, y as usize)
                    };
                    let rect = Rectangle::new(rect_pos, dim, ZplColor::White);

                    rect.draw(&mut pixmap).unwrap();
                    text.draw(&mut pixmap).unwrap();
                }
            }
        }
    }

    let png = pixmap.encode_png().expect("encode png");
    RenderOutput { png }
}
