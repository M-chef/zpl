use std::collections::HashMap;

use fontdue::{
    Font, Metrics,
    layout::{CoordinateSystem, Layout, TextStyle},
};
use intermediate_representation::{ADWAITA_MONO, OCR_B, OSWALD};

use crate::elements::Rect;

pub struct FontStore {
    fonts: HashMap<&'static str, LoadedFont>,
}

pub struct LoadedFont {
    pub font: fontdue::Font, // the parsed font data
    pub scale: f32,          // per-font scale correction (your 0.85 for Oswald)
}

impl FontStore {
    pub fn load_defaults() -> Self {
        let mut fonts = HashMap::new();

        let adwaita: &'static [u8] = include_bytes!("../../fonts/Oswald/Oswald-Medium.ttf");
        let font = Font::from_bytes(adwaita as &[u8], fontdue::FontSettings::default()).unwrap();
        fonts.insert(OSWALD, LoadedFont { font, scale: 0.85 });

        let ocrb: &'static [u8] = include_bytes!("../../fonts/AdwaitaMono/AdwaitaMono-Regular.ttf");
        let font = Font::from_bytes(ocrb as &[u8], fontdue::FontSettings::default()).unwrap();
        fonts.insert(ADWAITA_MONO, LoadedFont { font, scale: 1. });

        let ocrb: &'static [u8] = include_bytes!("../../fonts/OCRB/OCR-B.ttf");
        let font = Font::from_bytes(ocrb as &[u8], fontdue::FontSettings::default()).unwrap();
        fonts.insert(OCR_B, LoadedFont { font, scale: 1. });

        Self { fonts }
    }

    pub fn get(&self, id: &str) -> &LoadedFont {
        self.fonts.get(id).expect(&format!("Font not found {id}"))
    }

    pub fn measure_glyph(&self, font_id: &str, character: char, px: f32) -> Metrics {
        let loaded = self.get(font_id);
        loaded.font.metrics(character, px)
    }

    pub fn measure_text_dimensions(&self, font_id: &str, text: &str, px: f32) -> Rect {
        let loaded = self.get(font_id);

        let mut layout = Layout::new(CoordinateSystem::PositiveYDown);
        layout.append(&[&loaded.font], &TextStyle::new(text, px, 0));

        let width = match layout.glyphs().last() {
            Some(g) => {
                let metrics = loaded.font.metrics_indexed(g.key.glyph_index, px);
                g.x + metrics.advance_width
            }
            None => 0.0,
        };

        let height = loaded
            .font
            .horizontal_line_metrics(px)
            .map(|m| m.ascent - m.descent) // descent is negative in fontdue
            .unwrap_or(px); // fall back to px as a sane default

        Rect {
            x: 0.,
            y: 0.,
            width,
            height,
        }
    }
}
