// mod barcode;
mod datetime;
mod decode_image;

use intermediate_representation::{
    ADWAITA_MONO, Alignment, BarcodeBuilder, DecodedBitmap, Document, Element, Justification,
    OSWALD, Symbology, YReference, ean13_modules, estimate_code128_modules,
};
use zpl_parser::{
    Alignment as ZplAlignment, BarcodeType, ClockFormat, TextBlockJustification, ZplFormatCommand,
};

use crate::{datetime::format_timestamp, decode_image::decode_zpl_graphic};

#[derive(Default)]
enum Origin {
    #[default]
    Top,
    Bottom,
}

#[derive(Debug, Clone)]
struct FieldBlock {
    width: usize,
    lines: usize,
    line_spacing: isize,
    justification: TextBlockJustification,
    hanging_indent: usize,
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

#[derive(Debug, Clone, Default)]
pub struct SetRealTimeClock {
    month: Option<u8>,
    day: Option<u8>,
    year: Option<usize>,
    hour: Option<u8>,
    minute: Option<u8>,
    second: Option<u8>,
    format: ClockFormat,
}

#[derive(Default)]
struct LabelSize {
    total_width: Option<usize>,
    total_height: Option<usize>,
    current_width: usize,
    current_height: usize,
}

#[derive(Default)]
struct InterpreterState {
    x_offset: f32,
    y_offset: f32,
    current_x: f32,
    current_y: f32,
    current_origin: Origin,
    font: FontState,
    fieldblock_state: Option<FieldBlock>,
    current_justification: ZplAlignment,
    inverted: bool,
    barcode_type: Option<BarcodeType>,
    barcode_config: Option<BarcodeConfig>,
    escape_chars: Vec<char>,
    real_time_clock_setup: SetRealTimeClock,
    label_size: LabelSize,
}

impl InterpreterState {
    pub fn current_x(&self) -> f32 {
        self.x_offset + self.current_x
    }

    pub fn current_y(&self, element_height: f32) -> f32 {
        let offset = match self.current_origin {
            Origin::Top => 0.,
            Origin::Bottom => element_height,
        };

        self.y_offset + self.current_y - offset
    }
}

pub fn interpret(cmds: &[ZplFormatCommand]) -> Document {
    let mut state = InterpreterState::default();
    let mut elements = Vec::new();

    for cmd in cmds {
        match cmd {
            ZplFormatCommand::LabelHome { x, y } => {
                state.x_offset = *x as f32;
                state.y_offset = *y as f32;
            }
            ZplFormatCommand::FieldOrigin {
                x,
                y,
                justification,
            } => {
                state.current_x = *x as f32;
                state.current_y = *y as f32;
                state.current_justification = *justification;
            }
            ZplFormatCommand::FieldTypeset {
                x,
                y,
                justification,
            } => {
                state.current_x = *x as f32;
                state.current_y = *y as f32;
                state.current_origin = Origin::Bottom;
                state.current_justification = *justification;
            }
            ZplFormatCommand::FieldData(text) => {
                let mut content = text.clone();
                if let Some(barcode_type) = state.barcode_type {
                    // get barcode height from current setting.
                    // if neither barcode type nor global barcode settings holds height
                    // use default `10` (see zpl spec)
                    let barcode_height = barcode_type.height().unwrap_or(
                        state
                            .barcode_config
                            .as_ref()
                            .map(|conf| conf.height)
                            .unwrap_or(10),
                    );
                    // same for barcode width with default `2`
                    let module_width = state
                        .barcode_config
                        .as_ref()
                        .map(|conf| conf.width)
                        .unwrap_or(2);

                    let (symbology, width) = match barcode_type {
                        BarcodeType::Code39 => todo!(),
                        BarcodeType::Code128 { .. } => {
                            let modules = estimate_code128_modules(&text);
                            let target_width = modules as usize * module_width as usize;
                            (Symbology::Code128, target_width)
                        }
                        BarcodeType::Pdf417 => todo!(),
                        BarcodeType::Ean8 => todo!(),
                        BarcodeType::Ean13 { .. } => {
                            let modules = ean13_modules(&text);
                            let target_width = modules as usize * module_width as usize;
                            (Symbology::Ean13, target_width)
                        }
                        BarcodeType::Qr => todo!(),
                        BarcodeType::DataMatrix => todo!(),
                    };
                    let elem = BarcodeBuilder {
                        x: state.current_x(),
                        y: state.current_y(barcode_height as f32),
                        symbology,
                        data: text.to_string(),
                        show_text: barcode_type.show_text(),
                        width,
                        heigth: barcode_height,
                    };
                    let barcode_elements = elem.build();
                    elements.extend(barcode_elements);
                } else {
                    let escape_chars = &state.escape_chars;
                    if !escape_chars.is_empty() {
                        content =
                            format_timestamp(&content, escape_chars, &state.real_time_clock_setup);
                    }

                    let font = match state.font.current_font_name {
                        '0' => OSWALD,
                        _ => ADWAITA_MONO,
                    };
                    let elem = Element::Text {
                        x: state.current_x(),
                        y: state.current_y(state.font.current_font_height),
                        font: font.to_string(),
                        max_width: state
                            .fieldblock_state
                            .as_ref()
                            .map(|block| block.width as f32),
                        lines: state
                            .fieldblock_state
                            .as_ref()
                            .map(|block| block.lines)
                            .unwrap_or(1),
                        font_size: state.font.current_font_width,
                        content,
                        alignment: match state.current_justification {
                            ZplAlignment::Right => Alignment::Right,
                            _ => Alignment::Left,
                        },
                        justification: state
                            .fieldblock_state
                            .as_ref()
                            .map(|block| match block.justification {
                                TextBlockJustification::Left => Justification::Left,
                                TextBlockJustification::Center => Justification::Center,
                                TextBlockJustification::Right => Justification::Right,
                                TextBlockJustification::Justified => Justification::Justified,
                            })
                            .unwrap_or_default(),
                        y_reference: YReference::CapHeight,
                        inverted: state.inverted,
                    };
                    elements.push(elem);
                };
            }
            ZplFormatCommand::LabelLength(h) => state.label_size.total_height = Some(*h),
            ZplFormatCommand::PrintWidth(w) => state.label_size.total_width = Some(*w),
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
                let elem = Element::Image {
                    x: state.current_x(),
                    y: state.current_y(height as f32),
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
                let elem = Element::Rectangle {
                    x: state.current_x(),
                    y: state.current_y(*height as f32),
                    width: *width as f32,
                    height: *height as f32,
                    thickness: *thickness as f32,
                    color: match color {
                        zpl_parser::Color::Black => intermediate_representation::Color::Black,
                        zpl_parser::Color::White => intermediate_representation::Color::White,
                    },
                    rounding: *rounding,
                    inverted: state.inverted,
                };
                elements.push(elem);
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
            ZplFormatCommand::RealTimeClockMode { mode, language } => {}
            ZplFormatCommand::RealTimeClockEscapeChar {
                first,
                second,
                third,
            } => {
                state.escape_chars.push(*first);
                if let Some(second) = second {
                    state.escape_chars.push(*second);
                }
                if let Some(third) = third {
                    state.escape_chars.push(*third);
                }
            }
            ZplFormatCommand::SetRealTimeClock {
                month,
                day,
                year,
                hour,
                minute,
                second,
                format,
            } => {
                state.real_time_clock_setup = SetRealTimeClock {
                    month: *month,
                    day: *day,
                    year: *year,
                    hour: *hour,
                    minute: *minute,
                    second: *second,
                    format: *format,
                }
            }
            ZplFormatCommand::FieldSeparator => {
                // reset state
                state = InterpreterState {
                    font: FontState {
                        current_font_height: state.font.current_font_height,
                        current_font_width: state.font.current_font_width,
                        current_font_name: state.font.current_font_name,
                    },
                    label_size: state.label_size,
                    x_offset: state.x_offset,
                    y_offset: state.y_offset,
                    ..Default::default()
                }
            }
        }
    }

    Document {
        width: state.label_size.total_width,
        height: state.label_size.total_height,
        elements: elements,
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn interpreter_test() {}
}
