use zpl_parser::ZplFormatCommand;

pub enum FieldAlignment {
    LeftTop,
    LeftBottom,
}

#[derive(Debug, Clone)]
pub enum ZplElement {
    Text {
        x: i32,
        y: i32,
        font_size: f32,
        content: String,
    },
    Image {
        x: i32,
        y: i32,
        content: Vec<u8>,
    },
}

#[derive(Default)]
struct InterpreterState {
    current_x: i32,
    current_y: i32,
}

pub fn interpret(cmds: &[ZplFormatCommand]) -> Vec<ZplElement> {
    let mut state = InterpreterState::default();
    let mut elements = Vec::new();

    for cmd in cmds {
        println!("{cmd:?}");
        match cmd {
            ZplFormatCommand::FieldOrigin {
                x,
                y,
                justification,
            } => {
                state.current_x = *x;
                state.current_y = *y;
            }
            ZplFormatCommand::FieldTypeset {
                x,
                y,
                justification,
            } => {
                state.current_x = *x;
                state.current_y = *y;
            }
            ZplFormatCommand::FieldData(text) => {
                const VAR_NAME: f32 = 10.;
                elements.push(ZplElement::Text {
                    x: state.current_x,
                    y: state.current_y,
                    font_size: VAR_NAME,
                    content: text.clone(),
                })
            }
            ZplFormatCommand::LabelLength(_) => {}
            ZplFormatCommand::PrintWidth(_) => {}
            ZplFormatCommand::LabelShift(_) => {}
            ZplFormatCommand::Font {
                name,
                orientation,
                height,
                width,
            } => {}
            ZplFormatCommand::GraficField {
                compression_type,
                data_bytes,
                total_bytes,
                row_bytes,
                data,
            } => {}
            ZplFormatCommand::FieldSeparator => {
                // reset state
                state = InterpreterState::default()
            }
        }
    }

    elements
}

#[cfg(test)]
mod tests {

    #[test]
    fn interpreter_test() {}
}
