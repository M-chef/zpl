mod draw;
mod elements;
mod font;
mod text;

use std::error::Error;

use intermediate_representation::ADWAITA_MONO;
use intermediate_representation::{Color, Document};
use tiny_skia::{Color as SkiaColor, Pixmap};

use crate::draw::Drawable;

use crate::elements::lower;
pub use crate::font::*;

pub struct RenderConfig {
    pub dpi: f32,          // e.g. 300.0
    pub scale_factor: f32, // e.g. 1.0 for print, 2.0 for preview
    pub default_font_id: &'static str,
    pub background: Color,
}

impl RenderConfig {
    pub fn default() -> Self {
        Self {
            dpi: 300.,
            scale_factor: 1.,
            default_font_id: ADWAITA_MONO,
            background: Color::White,
        }
    }
}

pub struct LoweringContext {
    pub fonts: FontStore,
    pub config: RenderConfig,
}

pub struct RenderOutput {
    pub png: Vec<u8>,
}

pub fn render(document: &Document, ctx: &LoweringContext) -> Result<RenderOutput, Box<dyn Error>> {
    let document = lower(document, ctx);

    // Create a pixmap
    let width = document.width as u32;
    let height = document.height as u32;
    let mut target = Pixmap::new(width, height).expect("Invalid dimensions");

    // Set background
    let color = match ctx.config.background {
        Color::Black => SkiaColor::BLACK,
        Color::White => SkiaColor::WHITE,
    };
    target.fill(color);

    for el in &document.elements {
        if el.is_inverted() {
            el.draw_inverted(&mut target, &ctx.fonts)?;
        } else {
            el.draw(&mut target, &ctx.fonts)?;
        }
    }

    let png = target.encode_png().expect("encode png");
    Ok(RenderOutput { png })
}
