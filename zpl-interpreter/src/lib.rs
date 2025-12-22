mod decode_image;

use zpl_parser::{Color, Justification, ZplFormatCommand};

pub use crate::decode_image::DecodedBitmap;
use crate::decode_image::decode_zpl_graphic;

pub enum FieldAlignment {
    LeftTop,
    LeftBottom,
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
        x: i32,
        y: i32,
        bmp: DecodedBitmap,
    },
}

#[derive(Default)]
enum Origin {
    #[default]
    Top,
    Bottom,
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
    let mut width = 0;
    let mut height = 0;

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
                let elem = ZplElement::Text {
                    x: state.current_x(),
                    y: state.current_y(state.current_font_height as i32),
                    font_width: state.current_font_width,
                    font_height: state.current_font_height,
                    content: text.clone(),
                    justification: state.current_justification,
                    inverted: state.inverted,
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
                    x: state.current_x(),
                    y: state.current_y(height as i32),
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
