mod barcode;
mod decode_image;

use zpl_parser::{BarcodeType, Color, Justification, ZplFormatCommand};

pub use crate::decode_image::DecodedBitmap;
use crate::{barcode::bitmap_from_barcode, decode_image::decode_zpl_graphic};

pub enum FieldAlignment {
    LeftTop,
    LeftBottom,
}

#[derive(Debug, Clone)]
pub struct BarcodeContent {
    pub x: i32,
    pub y: i32,
    pub relative_y: f32,
    pub font_width: f32,
    pub text: String,
}

#[derive(Debug, Clone)]
pub enum ZplElement {
    Text {
        x: i32,
        y: i32,
        font_width: f32,
        font_height: f32,
        content: String,
        justification: Justification,
        inverted: bool,
    },
    Rectangle {
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        thickness: i32,
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
        content: Option<BarcodeContent>,
        bitmap: DecodedBitmap,
    },
}

#[derive(Default)]
enum Origin {
    #[default]
    Top,
    Bottom,
}

// #[derive(Clone, Copy)]
// struct BarcodeState {
//     r#type: BarcodeType,
//     height: i32,
// }

struct BarcodeConfig {
    width: u8,
    width_ratio: f32,
    height: usize,
}

#[derive(Default)]
struct InterpreterState {
    current_x: i32,
    current_y: i32,
    current_origin: Origin,
    current_font_height: f32,
    current_font_width: f32,
    current_justification: Justification,
    inverted: bool,
    barcode_type: Option<BarcodeType>,
    barcode_config: Option<BarcodeConfig>,
}

impl InterpreterState {
    pub fn current_x(&self) -> i32 {
        self.current_x
    }

    pub fn current_y(&self, height: i32) -> i32 {
        let offset = match self.current_origin {
            Origin::Top => 0,
            Origin::Bottom => height,
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
    let mut state = InterpreterState {
        current_font_height: 10.,
        current_font_width: 10.,
        ..Default::default()
    };
    let mut elements = Vec::new();
    let mut width = 0usize;
    let mut height = 0usize;

    for cmd in dbg!(cmds) {
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
                let mut content = text.clone();
                let elem = if let Some(barcode) = state.barcode_type
                    && let Ok(bitmap) =
                        bitmap_from_barcode(state.barcode_config.as_ref(), barcode, &mut content)
                {
                    let height = barcode.height().unwrap_or(
                        state
                            .barcode_config
                            .as_ref()
                            .map(|config| config.height)
                            .unwrap_or(10),
                    );
                    let relative_y = barcode.relative_text_ypos();
                    let font_width = (bitmap.width / content.chars().count()) as f32;
                    let content = barcode.show_content().then_some(BarcodeContent {
                        x: state.current_x(),
                        y: state.current_y(height as i32) + height as i32,
                        text: content,
                        font_width,
                        relative_y,
                    });
                    ZplElement::Barcode {
                        x: state.current_x() as usize,
                        y: state.current_y(height as i32) as usize,
                        content,
                        bitmap,
                    }
                } else {
                    ZplElement::Text {
                        x: state.current_x(),
                        y: state.current_y(state.current_font_height as i32),
                        font_width: state.current_font_width,
                        font_height: state.current_font_height,
                        content,
                        justification: state.current_justification,
                        inverted: state.inverted,
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
                state.current_font_height = *height as f32;
                state.current_font_width = *width as f32;
            }
            ZplFormatCommand::ChangeFont {
                name,
                height,
                width,
            } => {
                state.current_font_height = *height as f32;
                state.current_font_width = *width as f32;
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
                    y: state.current_y(height as i32) as usize,
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
                    y: state.current_y(*height as i32),
                    width: *width as i32,
                    height: *height as i32,
                    thickness: *thickness as i32,
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
            ZplFormatCommand::FieldSeparator => {
                // reset state
                state = InterpreterState {
                    current_font_height: state.current_font_height,
                    current_font_width: state.current_font_width,
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
