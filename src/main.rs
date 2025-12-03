use std::collections::HashMap;

// Cargo.toml dependencies needed:
// [dependencies]
// image = "0.24"
// imageproc = "0.23"
// rusttype = "0.9"

use image::{ImageBuffer, Rgb, RgbImage};
use imageproc::drawing::{draw_filled_rect_mut, draw_text_mut};
use rusttype::{Font, Scale};

#[derive(Debug, Clone)]
pub struct Label {
    width: u32,
    height: u32,
    dpmm: u32, // dots per millimeter
}

impl Label {
    pub fn new(width_mm: u32, height_mm: u32, dpmm: u32) -> Self {
        Label {
            width: width_mm * dpmm,
            height: height_mm * dpmm,
            dpmm,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ZplCommand {
    StartFormat,                         // ^XA
    EndFormat,                           // ^XZ
    FieldOrigin(i32, i32),               // ^FO
    FieldTypeset(i32, i32, Option<u32>), // ^FT (x, y, optional justification)
    FieldData(String),                   // ^FD
    FieldSeparator,                      // ^FS
    FieldHex(Option<char>),              // ^FH (hex indicator character, default _)
    Font(char, char, u32, u32),          // ^A (font, orientation, height, width)
    GraphicBox(u32, u32, u32, u32, u32), // ^GB (width, height, thickness, color, rounding)
    GraphicCircle(u32, u32, u32),        // ^GC (diameter, thickness, color)
    Barcode128(String),                  // ^BC (Code 128)
    LabelHome(i32, i32),                 // ^LH
    PrintWidth(u32),                     // ^PW (label width in dots)
    LabelLength(u32),                    // ^LL (label length in dots)
    PrintQuantity(u32),                  // ^PQ
    Unknown(String),
}

#[derive(Debug)]
pub struct ZplParser {
    commands: Vec<ZplCommand>,
}

impl ZplParser {
    pub fn new() -> Self {
        ZplParser {
            commands: Vec::new(),
        }
    }

    pub fn parse(&mut self, zpl: &str) -> Result<(), String> {
        let zpl = zpl.trim();
        let mut i = 0;
        let chars: Vec<char> = zpl.chars().collect();

        while i < chars.len() {
            if chars[i] == '^' || chars[i] == '~' {
                let (cmd, next_i) = self.parse_command(&chars, i)?;
                self.commands.push(cmd);
                i = next_i;
            } else {
                i += 1;
            }
        }

        Ok(())
    }

    fn parse_command(&self, chars: &[char], start: usize) -> Result<(ZplCommand, usize), String> {
        let mut i = start + 1;
        if i >= chars.len() {
            return Ok((ZplCommand::Unknown(String::new()), i));
        }

        // Get command code (usually 2 characters)
        let cmd_code = if i + 1 < chars.len() {
            format!("{}{}", chars[i], chars[i + 1])
        } else {
            chars[i].to_string()
        };

        i += 2;

        match cmd_code.as_str() {
            "XA" => Ok((ZplCommand::StartFormat, i)),
            "XZ" => Ok((ZplCommand::EndFormat, i)),
            "FS" => Ok((ZplCommand::FieldSeparator, i)),
            "FO" => {
                let (x, y, next_i) = self.parse_two_numbers(chars, i)?;
                Ok((ZplCommand::FieldOrigin(x, y), next_i))
            }
            "FT" => {
                let params = self.parse_multiple_numbers(chars, i, 3)?;
                let x = params.0.get(0).copied().unwrap_or(0);
                let y = params.0.get(1).copied().unwrap_or(0);
                let justification = params.0.get(2).map(|&v| v as u32);
                Ok((ZplCommand::FieldTypeset(x, y, justification), params.1))
            }
            "FD" => {
                let (text, next_i) = self.parse_until(chars, i, &['^', '~'])?;
                Ok((ZplCommand::FieldData(text), next_i))
            }
            "FH" => {
                let indicator = if i < chars.len() && !chars[i].is_whitespace() {
                    let c = chars[i];
                    i += 1;
                    Some(c)
                } else {
                    Some('_') // Default hex indicator
                };
                Ok((ZplCommand::FieldHex(indicator), i))
            }
            "PW" => {
                let (width, next_i) = self.parse_number(chars, i)?;
                Ok((ZplCommand::PrintWidth(width as u32), next_i))
            }
            "LL" => {
                let (length, next_i) = self.parse_number(chars, i)?;
                Ok((ZplCommand::LabelLength(length as u32), next_i))
            }
            "A" => {
                if i < chars.len() {
                    let font = chars[i];
                    let orientation = if i + 1 < chars.len() {
                        chars[i + 1]
                    } else {
                        'N'
                    };
                    i += 2;
                    let (height, width, next_i) = self.parse_two_numbers(chars, i)?;
                    Ok((
                        ZplCommand::Font(font, orientation, height as u32, width as u32),
                        next_i,
                    ))
                } else {
                    Ok((ZplCommand::Font('0', 'N', 30, 30), i))
                }
            }
            "GB" => {
                let params = self.parse_multiple_numbers(chars, i, 5)?;
                let width = params.0.get(0).copied().unwrap_or(100) as u32;
                let height = params.0.get(1).copied().unwrap_or(100) as u32;
                let thickness = params.0.get(2).copied().unwrap_or(1) as u32;
                let color = params.0.get(3).copied().unwrap_or(0) as u32;
                let rounding = params.0.get(4).copied().unwrap_or(0) as u32;
                Ok((
                    ZplCommand::GraphicBox(width, height, thickness, color, rounding),
                    params.1,
                ))
            }
            "GC" => {
                let params = self.parse_multiple_numbers(chars, i, 3)?;
                let diameter = params.0.get(0).copied().unwrap_or(100) as u32;
                let thickness = params.0.get(1).copied().unwrap_or(1) as u32;
                let color = params.0.get(2).copied().unwrap_or(0) as u32;
                Ok((
                    ZplCommand::GraphicCircle(diameter, thickness, color),
                    params.1,
                ))
            }
            "BC" => {
                let (data, next_i) = self.parse_until(chars, i, &['^', '~'])?;
                Ok((ZplCommand::Barcode128(data), next_i))
            }
            "LH" => {
                let (x, y, next_i) = self.parse_two_numbers(chars, i)?;
                Ok((ZplCommand::LabelHome(x, y), next_i))
            }
            "PQ" => {
                let (qty, next_i) = self.parse_number(chars, i)?;
                Ok((ZplCommand::PrintQuantity(qty as u32), next_i))
            }
            _ => {
                let (text, next_i) = self.parse_until(chars, i, &['^', '~'])?;
                Ok((ZplCommand::Unknown(format!("{}{}", cmd_code, text)), next_i))
            }
        }
    }

    fn parse_number(&self, chars: &[char], start: usize) -> Result<(i32, usize), String> {
        let mut i = start;
        let mut num_str = String::new();
        let mut negative = false;

        if i < chars.len() && chars[i] == '-' {
            negative = true;
            i += 1;
        }

        while i < chars.len() && chars[i].is_ascii_digit() {
            num_str.push(chars[i]);
            i += 1;
        }

        let num = num_str.parse::<i32>().unwrap_or(0);
        Ok((if negative { -num } else { num }, i))
    }

    fn parse_two_numbers(&self, chars: &[char], start: usize) -> Result<(i32, i32, usize), String> {
        let (num1, i) = self.parse_number(chars, start)?;
        let mut i = i;
        if i < chars.len() && chars[i] == ',' {
            i += 1;
        }
        let (num2, i) = self.parse_number(chars, i)?;
        Ok((num1, num2, i))
    }

    fn parse_multiple_numbers(
        &self,
        chars: &[char],
        start: usize,
        max_count: usize,
    ) -> Result<(Vec<i32>, usize), String> {
        let mut numbers = Vec::new();
        let mut i = start;

        for _ in 0..max_count {
            if i >= chars.len() || !chars[i].is_ascii_digit() && chars[i] != '-' {
                break;
            }
            let (num, next_i) = self.parse_number(chars, i)?;
            numbers.push(num);
            i = next_i;
            if i < chars.len() && chars[i] == ',' {
                i += 1;
            }
        }

        Ok((numbers, i))
    }

    fn parse_until(
        &self,
        chars: &[char],
        start: usize,
        delimiters: &[char],
    ) -> Result<(String, usize), String> {
        let mut i = start;
        let mut result = String::new();

        while i < chars.len() && !delimiters.contains(&chars[i]) {
            result.push(chars[i]);
            i += 1;
        }

        Ok((result, i))
    }

    pub fn render(&self, label: &Label) -> Result<RgbImage, String> {
        let mut img: RgbImage =
            ImageBuffer::from_pixel(label.width, label.height, Rgb([255u8, 255u8, 255u8]));

        let font_data = include_bytes!("../fonts/arial.ttf");
        let font = Font::try_from_bytes(font_data as &[u8]).ok_or("Failed to load font")?;

        let mut current_x = 0i32;
        let mut current_y = 0i32;
        let mut current_font_height = 30u32;
        let mut pending_text = String::new();
        let mut hex_indicator: Option<char> = None;
        let mut label_width = label.width;
        let mut label_height = label.height;

        for cmd in &self.commands {
            match cmd {
                ZplCommand::PrintWidth(width) => {
                    label_width = *width;
                }
                ZplCommand::LabelLength(length) => {
                    label_height = *length;
                }
                ZplCommand::FieldOrigin(x, y) => {
                    current_x = *x;
                    current_y = *y;
                }
                ZplCommand::FieldTypeset(x, y, _justification) => {
                    current_x = *x;
                    current_y = *y;
                    // Note: justification (0=left, 1=right, 2=auto) can be implemented
                }
                ZplCommand::FieldHex(indicator) => {
                    hex_indicator = *indicator;
                }
                ZplCommand::Font(_, _, height, _width) => {
                    current_font_height = *height;
                }
                ZplCommand::FieldData(text) => {
                    // Process hex encoding if ^FH was set
                    if let Some(indicator) = hex_indicator {
                        pending_text = self.decode_hex_string(text, indicator);
                    } else {
                        pending_text = text.clone();
                    }
                }
                ZplCommand::FieldSeparator => {
                    if !pending_text.is_empty() {
                        let scale = Scale::uniform(current_font_height as f32);
                        draw_text_mut(
                            &mut img,
                            Rgb([0u8, 0u8, 0u8]),
                            current_x as i32,
                            current_y as i32,
                            scale,
                            &font,
                            &pending_text,
                        );
                        pending_text.clear();
                    }
                    // Reset hex indicator after field separator (behavior may vary)
                    hex_indicator = None;
                }
                ZplCommand::GraphicBox(width, height, thickness, _color, _rounding) => {
                    let rect =
                        imageproc::rect::Rect::at(current_x, current_y).of_size(*width, *height);
                    draw_filled_rect_mut(&mut img, rect, Rgb([0u8, 0u8, 0u8]));

                    if *thickness > 0 && *thickness < (*width).min(*height) / 2 {
                        let inner_rect = imageproc::rect::Rect::at(
                            current_x + *thickness as i32,
                            current_y + *thickness as i32,
                        )
                        .of_size(width - 2 * thickness, height - 2 * thickness);
                        draw_filled_rect_mut(&mut img, inner_rect, Rgb([255u8, 255u8, 255u8]));
                    }
                }
                _ => {}
            }
        }

        Ok(img)
    }

    fn decode_hex_string(&self, text: &str, indicator: char) -> String {
        let mut result = String::new();
        let mut chars = text.chars().peekable();

        while let Some(c) = chars.next() {
            if c == indicator {
                // Read next two hex digits
                let hex1 = chars.next();
                let hex2 = chars.next();

                if let (Some(h1), Some(h2)) = (hex1, hex2) {
                    let hex_str = format!("{}{}", h1, h2);
                    if let Ok(byte) = u8::from_str_radix(&hex_str, 16) {
                        result.push(byte as char);
                        continue;
                    }
                }
                // If hex parsing fails, keep the indicator
                result.push(c);
                if let Some(h1) = hex1 {
                    result.push(h1);
                }
                if let Some(h2) = hex2 {
                    result.push(h2);
                }
            } else {
                result.push(c);
            }
        }

        result
    }

    pub fn commands(&self) -> &[ZplCommand] {
        &self.commands
    }
}

// Example usage:
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let zpl = r#"
        ^XA
        ^FO50,50^A0N,50,50^FDHello World!^FS
        ^FO50,150^A0N,30,30^FDZebra Label^FS
        ^FO50,250^GB400,100,5^FS
        ^XZ
    "#;

    let zpl = std::fs::read_to_string("zpl/zpl_real_live.txt").unwrap();

    let mut parser = ZplParser::new();
    parser.parse(&zpl)?;

    println!("Parsed commands:");
    for cmd in parser.commands() {
        println!("{:?}", cmd);
    }

    let label = Label::new(100, 150, 8); // 100x150mm at 8dpmm (203 DPI)
    let img = parser.render(&label)?;
    img.save("output.png")?;

    println!("Image saved to output.png");

    Ok(())
}
