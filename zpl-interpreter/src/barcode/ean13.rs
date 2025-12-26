use std::error::Error;

use rxing::{BarcodeFormat, EncodeHintValue, EncodeHints, Writer, oned::EAN13Writer};
use zpl_parser::Justification;

use crate::{BarcodeContent, barcode::bitmap_from_bitmatrix};

const EAN_WIDTH_CORRECTION: f32 = 5. / 6.;

pub(super) fn generate_ean13(
    width: Option<u8>,
    content: &str,
    height: Option<usize>,
) -> Result<BarcodeContent, Box<dyn Error>> {
    let content = check_ean_content(content)?;
    let writer = EAN13Writer::default();
    let modules = estimate_ean13_modules(&content);
    let width = width.unwrap_or(2);
    let width = width as f32 * modules as f32 * EAN_WIDTH_CORRECTION;
    let width = width as i32;
    let height = height.unwrap_or(10);
    let bitmatrix = writer.encode_with_hints(
        &content,
        &BarcodeFormat::EAN_13,
        width,
        height as i32,
        &EncodeHints::default().with(EncodeHintValue::Margin("0".into())),
    )?;
    let bitmap = bitmap_from_bitmatrix(bitmatrix)?;
    let font_width = (bitmap.width / content.chars().count()) as f32;
    // let text = pad_ean_content(&content);
    let text = content;

    Ok(BarcodeContent {
        text_x: 0,
        text_y: 0,
        text_y_shift: 0.,
        font_width,
        justification: Justification::Auto,
        text: Some(text),
        bitmap,
    })
}

fn check_ean_content(input: &str) -> Result<String, Box<dyn Error + 'static>> {
    let content_len = input.len();
    let mut content = String::new();
    for ch in input.chars() {
        let ch = if ch.is_ascii_digit() { ch } else { '0' };
        content.push(ch);
    }

    if content_len != 13 {
        match content.len() {
            12 => {}
            c if c < 12 => {
                let remaining = 12 - c;
                let mut filled = (0..remaining).map(|_| "0").collect::<String>();
                filled.push_str(&content);
                content = filled.into();
            }
            c if c > 12 => {
                let (part, _) = content.split_at(12);
                content = part.to_owned().into();
            }
            _ => panic!("should not happen or I did something wrong"),
        };
        let check_digit = ean13_check_digit(&content)?;
        content.push_str(&check_digit.to_string());
    };
    Ok(content)
}

fn pad_ean_content(contents: &str) -> String {
    let mut padded_content = String::new();
    for (i, ch) in contents.chars().enumerate() {
        if i == 1 {
            padded_content.push_str("          ");
        } else if i == 6 {
            padded_content.push_str("  ");
        } else {
            padded_content.push_str(" ");
        }
        padded_content.push(ch);
    }
    padded_content
}

fn estimate_ean13_modules(_data: &str) -> usize {
    // EAN-13 structure:
    // - Left guard: 3 modules
    // - Left digits (6 × 7): 42 modules
    // - Center guard: 5 modules
    // - Right digits (6 × 7): 42 modules
    // - Right guard: 3 modules
    // - Quiet zones: typically 11 modules on each side

    95 + 22 // 95 for barcode + 22 for quiet zones
}

fn ean13_check_digit(ean12: &str) -> Result<u8, &'static str> {
    if ean12.len() != 12 || !ean12.chars().all(|c| c.is_ascii_digit()) {
        return Err("EAN-12 muss genau 12 numerische Ziffern enthalten");
    }

    let sum: u32 = ean12
        .chars()
        .rev() // von rechts nach links
        .enumerate()
        .map(|(i, c)| {
            let digit = c.to_digit(10).unwrap();
            if i % 2 == 0 { digit * 3 } else { digit } // rechts startet mit *3
        })
        .sum();

    let check = (10 - (sum % 10)) % 10;
    Ok(check as u8)
}

#[cfg(test)]
mod tests {
    use crate::barcode::ean13::ean13_check_digit;

    #[test]
    fn ean13_check_digit_test() {
        let ean12 = "000012345678";
        let check = ean13_check_digit(ean12).unwrap();
        assert_eq!(check, 4);
    }
}
