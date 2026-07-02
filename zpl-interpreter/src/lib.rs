mod datetime;
mod decode_image;

use intermediate_representation::{
    ADWAITA_MONO, Alignment, BarcodeBuilder, DecodedBitmap, Document, Dots, Element, Justification,
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
    width: Dots,
    lines: u32,
    line_spacing: isize,
    justification: TextBlockJustification,
    hanging_indent: Dots,
}

struct BarcodeConfig {
    width: u8,
    width_ratio: f32,
    height: Dots,
}

struct FontState {
    current_font_height: Dots,
    current_font_width: Dots,
    current_font_name: char,
}

impl Default for FontState {
    fn default() -> Self {
        Self {
            current_font_height: Dots::from_unsigned(10),
            current_font_width: Dots::from_unsigned(10),
            current_font_name: 'A',
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct SetRealTimeClock {
    month: Option<u8>,
    day: Option<u8>,
    year: Option<u32>,
    hour: Option<u8>,
    minute: Option<u8>,
    second: Option<u8>,
    format: ClockFormat,
}

struct LabelSize {
    total_width: Option<Dots>,
    total_height: Option<Dots>,
    current_width: Dots,
    current_height: Dots,
}

impl Default for LabelSize {
    fn default() -> Self {
        Self {
            total_width: Default::default(),
            total_height: Default::default(),
            current_width: Dots::from_unsigned(0),
            current_height: Dots::from_unsigned(0),
        }
    }
}

struct InterpreterState {
    x_offset: Dots,
    y_offset: Dots,
    current_x: Dots,
    current_y: Dots,
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

impl Default for InterpreterState {
    fn default() -> Self {
        Self {
            x_offset: Dots::from_unsigned(0),
            y_offset: Dots::from_unsigned(0),
            current_x: Dots::from_unsigned(0),
            current_y: Dots::from_unsigned(0),
            current_origin: Default::default(),
            font: Default::default(),
            fieldblock_state: Default::default(),
            current_justification: Default::default(),
            inverted: Default::default(),
            barcode_type: Default::default(),
            barcode_config: Default::default(),
            escape_chars: Default::default(),
            real_time_clock_setup: Default::default(),
            label_size: Default::default(),
        }
    }
}

impl InterpreterState {
    pub fn current_x(&self) -> Dots {
        self.x_offset + self.current_x
    }

    pub fn current_y(&self, element_height: Dots) -> Dots {
        let offset = match self.current_origin {
            Origin::Top => Dots::from_unsigned(0),
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
                state.x_offset = Dots::from_unsigned(*x);
                state.y_offset = Dots::from_unsigned(*y);
            }
            ZplFormatCommand::FieldOrigin {
                x,
                y,
                justification,
            } => {
                state.current_x = Dots::from_unsigned(*x);
                state.current_y = Dots::from_unsigned(*y);
                state.current_justification = *justification;
            }
            ZplFormatCommand::FieldTypeset {
                x,
                y,
                justification,
            } => {
                state.current_x = Dots::from_unsigned(*x);
                state.current_y = Dots::from_unsigned(*y);
                state.current_origin = Origin::Bottom;
                state.current_justification = *justification;
            }
            ZplFormatCommand::FieldData(text) => {
                let mut content = text.clone();
                if let Some(barcode_type) = state.barcode_type {
                    // get barcode height from current setting.
                    // if neither barcode type nor global barcode settings holds height
                    // use default `10` (see zpl spec)
                    let barcode_height = barcode_type
                        .height()
                        .map(|h| Dots::from_unsigned(h))
                        .unwrap_or(
                            state
                                .barcode_config
                                .as_ref()
                                .map(|conf| conf.height)
                                .unwrap_or(Dots::from_unsigned(10)),
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
                            let target_width = modules as u32 * module_width as u32;
                            (Symbology::Code128, target_width)
                        }
                        BarcodeType::Pdf417 => todo!(),
                        BarcodeType::Ean8 => todo!(),
                        BarcodeType::Ean13 { .. } => {
                            let modules = ean13_modules(&text);
                            let target_width = modules as u32 * module_width as u32;
                            (Symbology::Ean13, target_width)
                        }
                        BarcodeType::Qr => todo!(),
                        BarcodeType::DataMatrix => todo!(),
                    };
                    let elem = BarcodeBuilder {
                        x: state.current_x().to_length(),
                        y: state.current_y(barcode_height).to_length(),
                        symbology,
                        data: text.to_string(),
                        show_text: barcode_type.show_text(),
                        width: Dots::from_unsigned(width).to_length(),
                        heigth: barcode_height.to_length(),
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
                        x: state.current_x().to_length(),
                        y: state.current_y(state.font.current_font_height).to_length(),
                        font: font.to_string(),
                        max_width: state
                            .fieldblock_state
                            .as_ref()
                            .map(|block| block.width)
                            .map(|width| width.to_length()),
                        lines: state
                            .fieldblock_state
                            .as_ref()
                            .map(|block| block.lines)
                            .unwrap_or(1),
                        font_size: state.font.current_font_width.to_fontsize(),
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
            ZplFormatCommand::LabelLength(h) => {
                state.label_size.total_height = Some(Dots::from_unsigned(*h))
            }
            ZplFormatCommand::PrintWidth(w) => {
                state.label_size.total_width = Some(Dots::from_unsigned(*w))
            }
            ZplFormatCommand::LabelShift(_) => {}
            ZplFormatCommand::Font {
                name,
                orientation,
                height,
                width,
            } => {
                state.font.current_font_name = *name;
                state.font.current_font_height = Dots::from_unsigned(*height);
                state.font.current_font_width = Dots::from_unsigned(*width);
            }
            ZplFormatCommand::ChangeFont {
                name,
                height,
                width,
            } => {
                state.font.current_font_name = *name;
                state.font.current_font_height = Dots::from_unsigned(*height);
                state.font.current_font_width = Dots::from_unsigned(*width);
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
                    x: state.current_x().to_length(),
                    y: state.current_y(Dots::from_unsigned(height)).to_length(),
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
                    x: state.current_x().to_length(),
                    y: state.current_y(Dots::from_unsigned(*height)).to_length(),
                    width: Dots::from_unsigned(*width).to_length(),
                    height: Dots::from_unsigned(*height).to_length(),
                    thickness: Dots::from_unsigned(*thickness).to_length(),
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
                    height: Dots::from_unsigned(*height),
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
                    width: Dots::from_unsigned(*width),
                    lines: *lines,
                    line_spacing: *line_spacing,
                    justification: *justification,
                    hanging_indent: Dots::from_unsigned(*hanging_indent),
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
        width: state.label_size.total_width.map(|w| w.to_length()),
        height: state.label_size.total_height.map(|h| h.to_length()),
        elements: elements,
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn interpreter_test() {}
}
