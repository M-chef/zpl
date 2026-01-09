mod barcode;
mod decode_image;

use zpl_parser::{BarcodeType, Color, Justification, TextBlockJustification, ZplFormatCommand};

pub use crate::decode_image::DecodedBitmap;
use crate::{
    barcode::{BarcodeContent, barcode_from_content},
    decode_image::decode_zpl_graphic,
};

#[derive(Default)]
enum Origin {
    #[default]
    Top,
    Bottom,
}

#[derive(Debug, Clone)]
pub enum ZplElement {
    Text {
        x: usize,
        y: usize,
        font_name: char,
        font_width: f32,
        font_height: f32,
        content: String,
        justification: Justification,
        inverted: bool,
        field_block: Option<FieldBlock>,
    },
    Rectangle {
        x: usize,
        y: usize,
        width: usize,
        height: usize,
        thickness: usize,
        color: Color,
        rounding: u8,
        inverted: bool,
    },
    Image {
        x: usize,
        y: usize,
        bmp: DecodedBitmap,
    },
    Barcode {
        x: usize,
        y: usize,
        content: BarcodeContent,
    },
}

struct BarcodeConfig {
    width: u8,
    width_ratio: f32,
    height: usize,
}

struct FontState {
    current_font_height: f32,
    current_font_width: f32,
    current_font_name: char,
}

impl Default for FontState {
    fn default() -> Self {
        Self {
            current_font_height: 10.,
            current_font_width: 10.,
            current_font_name: 'A',
        }
    }
}

#[derive(Debug, Clone)]
pub struct FieldBlock {
    pub width: usize,
    pub lines: usize,
    pub line_spacing: isize,
    pub justification: TextBlockJustification,
    pub hanging_indent: usize,
}

#[derive(Default)]
struct InterpreterState {
    current_x: usize,
    current_y: usize,
    current_origin: Origin,
    font: FontState,
    fieldblock_state: Option<FieldBlock>,
    current_justification: Justification,
    inverted: bool,
    barcode_type: Option<BarcodeType>,
    barcode_config: Option<BarcodeConfig>,
}

impl InterpreterState {
    pub fn current_x(&self) -> usize {
        self.current_x
    }

    pub fn current_y(&self, element_height: usize) -> usize {
        let offset = match self.current_origin {
            Origin::Top => 0,
            Origin::Bottom => element_height,
        };

        self.current_y - offset
    }
}

#[derive(Debug)]
pub struct ZplLabel {
    pub width: usize,
    pub height: usize,
    pub elements: Vec<ZplElement>,
}

pub fn interpret(cmds: &[ZplFormatCommand]) -> ZplLabel {
    let mut state = InterpreterState::default();
    let mut elements = Vec::new();
    let mut width = 0usize;
    let mut height = 0usize;

    for cmd in cmds {
        match cmd {
            ZplFormatCommand::FieldOrigin {
                x,
                y,
                justification,
            } => {
                state.current_x = *x;
                state.current_y = *y;
                state.current_justification = *justification;
            }
            ZplFormatCommand::FieldTypeset {
                x,
                y,
                justification,
            } => {
                state.current_x = *x;
                state.current_y = *y;
                state.current_origin = Origin::Bottom;
                state.current_justification = *justification;
            }
            ZplFormatCommand::FieldData(text) => {
                let content = text.clone();
                let elem = if let Some(barcode_type) = state.barcode_type
                    && let Ok(mut barcode_content) =
                        barcode_from_content(state.barcode_config.as_ref(), barcode_type, &content)
                {
                    barcode_content.set_text_x(state.current_x());
                    let element_height = barcode_content.bitmap.height;
                    barcode_content.set_text_y(state.current_y(element_height));

                    ZplElement::Barcode {
                        x: state.current_x() as usize,
                        y: state.current_y(element_height) as usize,
                        content: barcode_content,
                    }
                } else {
                    ZplElement::Text {
                        x: state.current_x(),
                        y: state.current_y(state.font.current_font_height as usize),
                        font_name: state.font.current_font_name,
                        font_width: state.font.current_font_width,
                        font_height: state.font.current_font_height,
                        content,
                        justification: state.current_justification,
                        inverted: state.inverted,
                        field_block: state.fieldblock_state.clone(),
                    }
                };
                elements.push(elem)
            }
            ZplFormatCommand::LabelLength(h) => height = *h,
            ZplFormatCommand::PrintWidth(w) => width = *w,
            ZplFormatCommand::LabelShift(_) => {}
            ZplFormatCommand::Font {
                name,
                orientation,
                height,
                width,
            } => {
                state.font.current_font_name = *name;
                state.font.current_font_height = *height as f32;
                state.font.current_font_width = *width as f32;
            }
            ZplFormatCommand::ChangeFont {
                name,
                height,
                width,
            } => {
                state.font.current_font_name = *name;
                state.font.current_font_height = *height as f32;
                state.font.current_font_width = *width as f32;
            }
            ZplFormatCommand::GraphicField {
                compression_type,
                data_bytes,
                total_bytes,
                row_bytes,
                data,
            } => {
                let width = row_bytes * 8;
                let height = total_bytes / row_bytes;
                let bmp = match decode_zpl_graphic(
                    data.compression_method,
                    &data.data,
                    width,
                    height,
                    *row_bytes,
                ) {
                    Ok(bmp) => bmp,
                    _ => DecodedBitmap::default(),
                };
                let elem = ZplElement::Image {
                    x: state.current_x() as usize,
                    y: state.current_y(height) as usize,
                    bmp,
                };
                elements.push(elem)
            }
            ZplFormatCommand::GraphicalBox {
                width,
                height,
                thickness,
                color,
                rounding,
            } => {
                let elem = ZplElement::Rectangle {
                    x: state.current_x(),
                    y: state.current_y(*height),
                    width: *width,
                    height: *height,
                    thickness: *thickness,
                    color: *color,
                    rounding: *rounding,
                    inverted: state.inverted,
                };
                elements.push(elem)
            }
            ZplFormatCommand::Inverted => state.inverted = true,
            ZplFormatCommand::BarcodeConfig {
                width,
                width_ratio,
                height,
            } => {
                state.barcode_config = Some(BarcodeConfig {
                    width: *width,
                    width_ratio: *width_ratio,
                    height: *height,
                })
            }
            ZplFormatCommand::Barcode(barcode_type) => state.barcode_type = Some(*barcode_type),
            ZplFormatCommand::FieldHexIndicator { char } => {}
            ZplFormatCommand::CharacterSet { num, mapping } => {}
            ZplFormatCommand::FieldBlock {
                width,
                lines,
                line_spacing,
                justification,
                hanging_indent,
            } => {
                state.fieldblock_state = Some(FieldBlock {
                    width: *width,
                    lines: *lines,
                    line_spacing: *line_spacing,
                    justification: *justification,
                    hanging_indent: *hanging_indent,
                })
            }
            ZplFormatCommand::FieldSeparator => {
                // reset state
                state = InterpreterState {
                    font: FontState {
                        current_font_height: state.font.current_font_height,
                        current_font_width: state.font.current_font_width,
                        current_font_name: state.font.current_font_name,
                    },
                    ..Default::default()
                }
            }
        }
    }

    ZplLabel {
        width,
        height,
        elements,
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn interpreter_test() {}
}
