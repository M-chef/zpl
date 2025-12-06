mod decode_image;

use zpl_parser::{Justification, ZplFormatCommand};

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
    },
    Image {
        x: i32,
        y: i32,
        bmp: DecodedBitmap,
    },
}

#[derive(Default)]
struct InterpreterState {
    current_x: i32,
    current_y: i32,
    current_font_height: f32,
    current_font_width: f32,
    current_justification: Justification,
}

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
                state.current_justification = *justification;
            }
            ZplFormatCommand::FieldData(text) => {
                let elem = ZplElement::Text {
                    x: state.current_x,
                    y: state.current_y,
                    font_width: state.current_font_width,
                    font_height: state.current_font_height,
                    content: text.clone(),
                    justification: state.current_justification,
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
            ZplFormatCommand::GraficField {
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
                    x: state.current_x,
                    y: state.current_y,
                    bmp,
                };
                elements.push(elem)
            }
            ZplFormatCommand::FieldSeparator => {
                // reset state
                state = InterpreterState::default()
            }
        }
    }

    ZplLabel {
        width,
        height,
        elements,
    }
}

// fn check_label_bounds(elem: ZplElement, width: &mut usize, height: &mut usize) -> _ {
//     width = max(width, elem.)
// }

#[cfg(test)]
mod tests {

    #[test]
    fn interpreter_test() {}
}
