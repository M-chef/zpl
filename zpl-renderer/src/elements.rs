use intermediate_representation::{Alignment, Color, Document, Element, Justification, YReference};

use crate::{LoweringContext, font::FontStore, text::to_glyphs};

#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PositionedGlyph {
    pub glyph_index: u16,
    pub x: f32,
    pub y: f32,
    pub advance_width: f32,
}

#[derive(Debug, Clone)]
pub enum DrawCommand {
    Text {
        font: String,
        font_size: f32,
        glyphs: Vec<PositionedGlyph>,
        bold: bool,
        inverted: bool,
    },
    Rectangle {
        bounds: Rect,
        line_thickness: f32,
        rounding: f32,
        color: Color,
        inverted: bool,
    },
    FilledRect {
        bounds: Rect,
        rounding: f32,
        color: Color,
        inverted: bool,
    },
    BitMap {
        bounds: Rect,
        data: Vec<u8>,
    },
}

impl DrawCommand {
    pub fn is_inverted(&self) -> bool {
        match self {
            DrawCommand::Text { inverted, .. } => *inverted,
            DrawCommand::Rectangle { inverted, .. } => *inverted,
            DrawCommand::FilledRect { inverted, .. } => *inverted,
            DrawCommand::BitMap { .. } => false,
        }
    }

    pub fn max_x(&self, font_store: &FontStore) -> f32 {
        match self {
            DrawCommand::Text { .. } => {
                let bounds = self.screen_bounds(font_store);
                bounds.x + bounds.height
            }
            DrawCommand::Rectangle { bounds, .. } => bounds.x + bounds.width,
            DrawCommand::FilledRect { bounds, .. } => bounds.x + bounds.width,
            DrawCommand::BitMap { bounds, .. } => bounds.x + bounds.width,
        }
    }

    pub fn max_y(&self, font_store: &FontStore) -> f32 {
        match self {
            DrawCommand::Text { .. } => {
                let bounds = self.screen_bounds(font_store);
                bounds.y + bounds.height
            }
            DrawCommand::Rectangle { bounds, .. } => bounds.y + bounds.height,
            DrawCommand::FilledRect { bounds, .. } => bounds.y + bounds.height,
            DrawCommand::BitMap { bounds, .. } => bounds.y + bounds.height,
        }
    }

    pub fn screen_bounds(&self, font_store: &FontStore) -> Rect {
        match self {
            DrawCommand::Text {
                font_size,
                glyphs,
                font,
                ..
            } => {
                let mut min_x = f32::INFINITY;
                let mut min_y = f32::INFINITY;
                let mut max_x = f32::NEG_INFINITY;
                let mut max_y = f32::NEG_INFINITY;

                for glyph in glyphs {
                    let metrics = font_store
                        .get(font)
                        .font
                        .metrics_indexed(glyph.glyph_index, *font_size);
                    // mirrors draw_text exactly
                    let top = glyph.y - metrics.height as f32 - metrics.ymin as f32;
                    let bottom = top + metrics.height as f32;
                    let left = glyph.x;
                    let right = glyph.x + glyph.advance_width;

                    min_x = min_x.min(left);
                    min_y = min_y.min(top);
                    max_x = max_x.max(right);
                    max_y = max_y.max(bottom);
                }

                Rect {
                    x: min_x,
                    y: min_y,
                    width: max_x - min_x,
                    height: max_y - min_y,
                }
            }
            DrawCommand::Rectangle { bounds, .. } => *bounds,
            DrawCommand::FilledRect { bounds, .. } => *bounds,
            DrawCommand::BitMap { bounds, .. } => *bounds,
        }
    }
}

#[derive(Debug)]
pub struct DrawDocument {
    pub width: u32,
    pub height: u32,
    pub elements: Vec<DrawCommand>,
}

pub fn lower(doc: &Document, ctx: &LoweringContext) -> DrawDocument {
    let mut elements = Vec::new();

    for elem in &doc.elements {
        match elem {
            Element::Text {
                x,
                y,
                max_width,
                lines,
                font,
                font_size,
                content,
                justification,
                alignment,
                y_reference,
                inverted,
            } => {
                let mut bounds =
                    ctx.fonts
                        .measure_text_dimensions(&font, &content, font_size.as_f32());
                bounds.x = match alignment {
                    Alignment::Left => x.as_f32(),
                    Alignment::Right => x.as_f32() - bounds.width,
                };
                if let Some(width) = max_width {
                    bounds.width = width.as_f32()
                }
                let mut line_spacing = None;
                // let mut justification = Justification::Left;

                // First resolve both Y values
                bounds.y = {
                    let y = y.as_f32();
                    let loaded_font = ctx.fonts.get(&font);
                    match y_reference {
                        YReference::Baseline => y,
                        YReference::CapHeight => {
                            let idx = loaded_font.font.lookup_glyph_index('H');
                            let m = loaded_font.font.metrics_indexed(idx, font_size.as_f32());
                            y + m.ymin as f32 + m.height as f32
                        }
                        YReference::Ascent => {
                            let lm = loaded_font
                                .font
                                .horizontal_line_metrics(font_size.as_f32())
                                .unwrap();
                            y + lm.ascent
                        }
                        YReference::Bottom => {
                            let lm = loaded_font
                                .font
                                .horizontal_line_metrics(font_size.as_f32())
                                .unwrap();
                            y + lm.descent
                        }
                    }
                };

                let glyphs = to_glyphs(
                    &font,
                    font_size.as_f32(),
                    bounds,
                    line_spacing,
                    *justification,
                    *y_reference,
                    &content,
                    ctx,
                );
                let com = DrawCommand::Text {
                    font: font.clone(),
                    font_size: font_size.as_f32(),
                    glyphs,
                    bold: false,
                    inverted: *inverted,
                };
                elements.push(com);
            }
            Element::Rectangle {
                x,
                y,
                width,
                height,
                thickness,
                color,
                rounding,
                inverted,
            } => {
                let bound = Rect {
                    x: x.as_f32(),
                    y: y.as_f32(),
                    width: width.as_f32(),
                    height: height.as_f32(),
                };
                let com = DrawCommand::Rectangle {
                    bounds: bound,
                    line_thickness: thickness.as_f32(),
                    rounding: *rounding as f32,
                    color: *color,
                    inverted: *inverted,
                };
                elements.push(com);
            }
            Element::Image { x, y, bmp } => {
                let bound = Rect {
                    x: x.as_f32(),
                    y: y.as_f32(),
                    width: bmp.width.as_f32(),
                    height: bmp.height.as_f32(),
                };
                let com = DrawCommand::BitMap {
                    bounds: bound,
                    data: bmp.pixels.clone(),
                };
                elements.push(com);
            }
        }
    }

    let mut current_width: f32 = 0.;
    let mut current_height: f32 = 0.;

    elements.iter().for_each(|elem| {
        current_width = current_width.max(elem.max_x(&ctx.fonts));
        current_height = current_height.max(elem.max_y(&ctx.fonts));
    });

    DrawDocument {
        width: doc
            .width
            .map(|w| w.as_i32() as u32)
            .unwrap_or(current_width.round() as u32),
        height: doc
            .height
            .map(|h| h.as_i32() as u32)
            .unwrap_or(current_height.round() as u32),
        elements,
    }
}

#[cfg(test)]
mod tests {
    use intermediate_representation::{
        Alignment, Dots, Element, FontSize, Justification, YReference,
    };

    use crate::{ADWAITA_MONO, Document, FontStore, LoweringContext, RenderConfig, lower};

    #[test]
    fn should_align_text_right() {
        let font_size = 10;
        let text = Element::Text {
            x: Dots::from_unsigned(100).to_length(),
            y: Dots::from_unsigned(0).to_length(),
            max_width: None,
            lines: 1,
            font: ADWAITA_MONO.to_string(),
            font_size: FontSize::from(Dots::from_unsigned(font_size).to_length()),
            content: "Test".to_string(),
            justification: Justification::Left,
            alignment: Alignment::Right,
            y_reference: YReference::Baseline,
            inverted: false,
        };

        let doc = Document {
            width: Some(Dots::from_unsigned(100).to_length()),
            height: Some(Dots::from_unsigned(100).to_length()),
            elements: vec![text],
        };

        let ctx = LoweringContext {
            fonts: FontStore::load_defaults(),
            config: RenderConfig {
                dpi: 1.,
                scale_factor: 1.,
                default_font_id: ADWAITA_MONO,
                background: crate::Color::White,
            },
        };
        let doc = lower(&doc, &ctx);
        let lowered_text = &doc.elements[0];
        let bounds = lowered_text.screen_bounds(&ctx.fonts);
        assert!(bounds.x < 100.)
    }
}
