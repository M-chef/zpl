use std::error::Error;

use rxing::{
    BarcodeFormat, EncodeHintValue, EncodeHints, Writer,
    oned::{Code128Writer, EAN13Writer},
};
use zpl_parser::BarcodeType;

use crate::{BarcodeConfig, DecodedBitmap};

const EAN_WIDTH_CORRECTION: f32 = 5. / 6.;

pub(super) fn bitmap_from_barcode(
    barcode_config: Option<&BarcodeConfig>,
    barcode: BarcodeType,
    contents: &mut String,
) -> Result<DecodedBitmap, Box<dyn Error>> {
    let bitmatrix = match barcode {
        BarcodeType::Code39 => todo!(),
        BarcodeType::Code128 {
            orientation,
            height,
            show_text,
            text_above,
            check_digit,
            mode,
        } => {
            let writer = Code128Writer::default();
            let modules = estimate_code128_modules(contents);
            let width =
                barcode_config.map(|config| config.width).unwrap_or(2) as usize * modules / 2;
            let height = height.unwrap_or(barcode_config.map(|config| config.height).unwrap_or(10));
            writer.encode(
                contents,
                &BarcodeFormat::CODE_128,
                width as i32,
                height as i32,
            )?
        }
        BarcodeType::Pdf417 => todo!(),
        BarcodeType::Ean8 => todo!(),
        BarcodeType::Ean13 {
            orientation,
            height,
            show_text,
            text_above,
        } => {
            check_ean_content(contents)?;

            let writer = EAN13Writer::default();
            let modules = estimate_ean13_modules(&contents);
            let width = barcode_config.map(|config| config.width).unwrap_or(2);
            let width = width as f32 * modules as f32 * EAN_WIDTH_CORRECTION;
            let width = width as i32;
            let height = height.unwrap_or(barcode_config.map(|config| config.height).unwrap_or(10));
            let bitmatrix = writer.encode_with_hints(
                &contents,
                &BarcodeFormat::EAN_13,
                width,
                height as i32,
                &EncodeHints::default().with(EncodeHintValue::Margin("0".into())),
            )?;
            pad_ean_content(contents);
            bitmatrix
        }
        BarcodeType::Qr => todo!(),
        BarcodeType::DataMatrix => todo!(),
    };

    let width = bitmatrix.width() as usize;
    let height = bitmatrix.height() as usize;

    let mut pixels = Vec::with_capacity(width * height);

    for y in 0..height {
        for x in 0..width {
            match bitmatrix.get(x as u32, y as u32) {
                true => pixels.push(1),
                false => pixels.push(0),
            }
        }
    }

    Ok(DecodedBitmap {
        width,
        height,
        pixels,
    })
}

fn check_ean_content(contents: &mut String) -> Result<(), Box<dyn Error + 'static>> {
    let content_len = contents.len();
    if content_len != 13 {
        match contents.len() {
            12 => {}
            c if c < 12 => {
                let remaining = 12 - c;
                let mut filled = (0..remaining).map(|_| "0").collect::<String>();
                filled.push_str(contents);
                *contents = filled;
            }
            c if c > 12 => {
                let (part, _) = contents.split_at(12);
                *contents = part.to_string();
            }
            _ => panic!("should not happen or I did something wrong"),
        };
        let check_digit = ean13_check_digit(contents)?;
        contents.push_str(&check_digit.to_string());
    };
    Ok(())
}

fn pad_ean_content(contents: &mut String) {
    let mut padded_content = String::new();
    for (i, ch) in contents.chars().enumerate() {
        if i == 1 {
            padded_content.push_str("   ");
        } else if i == 6 {
            padded_content.push_str("  ");
        } else {
            padded_content.push_str(" ");
        }
        padded_content.push(ch);
    }
    *contents = padded_content;
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

fn estimate_code128_modules(data: &str) -> usize {
    // Code128 structure:
    // - Start code: 11 modules
    // - Each character: 11 modules
    // - Checksum: 11 modules
    // - Stop pattern: 13 modules
    // - Quiet zone: typically 10 modules on each side

    let char_count = data.len();
    11 + (char_count * 11) + 11 + 13 + 20 // +20 for quiet zones
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
