use std::error::Error;

use rxing::{BarcodeFormat, EncodeHintValue, EncodeHints, Writer, oned::EAN13Writer};

use crate::{
    Alignment, DecodedBitmap, Element, Justification, OCR_B, YReference,
    barcode::bitmap_from_bitmatrix,
    hri_ratios,
    measure::{FontSize, Length},
};

impl DecodedBitmap {
    fn enlarge_modules_for_ean13(&mut self, module_width: usize) {
        for _ in 0..=(self.height.as_f32() / 10.) as usize {
            self.add_module(1, module_width, 1u8);
            self.add_module(1, module_width, 0u8);
            self.add_module(1, module_width, 1u8);

            self.add_module(43, module_width, 0u8);

            self.add_module(1, module_width, 1u8);
            self.add_module(1, module_width, 0u8);
            self.add_module(1, module_width, 1u8);

            self.add_module(43, module_width, 0u8);

            self.add_module(1, module_width, 1u8);
            self.add_module(1, module_width, 0u8);
            self.add_module(1, module_width, 1u8);

            self.height += 1.;
        }
    }

    fn add_module(&mut self, modules: usize, module_width: usize, kind: u8) {
        if kind != 0u8 && kind != 1u8 {
            panic!("no valid bitmap value")
        }
        let first = (0..module_width * modules).map(|_| kind);
        self.pixels.extend(first);
    }
}

pub(crate) fn generate_ean13(
    target_width: i32,
    target_height: i32,
    content: &str,
) -> Result<DecodedBitmap, Box<dyn Error>> {
    let content = check_ean_content(content)?;
    let writer = EAN13Writer::default();

    let bitmatrix = writer.encode_with_hints(
        &content,
        &BarcodeFormat::EAN_13,
        target_width,
        target_height,
        &EncodeHints::default().with(EncodeHintValue::Margin("0".into())),
    )?;

    let module_width = target_width as f32 / ean13_modules(&content) as f32;
    let mut bitmap = bitmap_from_bitmatrix(bitmatrix)?;
    bitmap.enlarge_modules_for_ean13(module_width as usize);

    Ok(bitmap)
}

pub(crate) fn generate_ean13_text(
    x: Length,
    y: Length,
    data: &str,
    bmp: &DecodedBitmap,
) -> Result<Vec<Element>, Box<dyn Error>> {
    let data = check_ean_content(data)?;
    let font_size = hri_ratios::EAN13 * bmp.width;
    let y = y + bmp.height + font_size * 0.7;
    let font_size: FontSize = font_size.into();

    let first = Element::Text {
        x,
        y,
        max_width: Some(bmp.width),
        lines: 1,
        font: OCR_B.to_string(),
        font_size,
        content: data[0..1].to_string(),
        alignment: Alignment::Right,
        justification: Justification::Left,
        y_reference: YReference::Baseline,
        inverted: false,
    };
    let second = Element::Text {
        x,
        y,
        max_width: Some(bmp.width / 2.),
        lines: 1,
        font: OCR_B.to_string(),
        font_size,
        content: data[1..7].to_string(),
        alignment: Alignment::Left,
        justification: Justification::Center,
        y_reference: YReference::Baseline,
        inverted: false,
    };
    let third = Element::Text {
        x: x + bmp.width / 2.,
        y,
        max_width: Some(bmp.width / 2.),
        lines: 1,
        font: OCR_B.to_string(),
        font_size,
        content: data[7..].to_string(),
        alignment: Alignment::Left,
        justification: Justification::Center,
        y_reference: YReference::Baseline,
        inverted: false,
    };
    let texts = vec![first, second, third];

    Ok(texts)
}

pub fn ean13_modules(_data: &str) -> usize {
    // EAN-13 structure:
    // - Left guard: 3 modules
    // - Left digits (6 × 7): 42 modules
    // - Center guard: 5 modules
    // - Right digits (6 × 7): 42 modules
    // - Right guard: 3 modules
    // - Quiet zones: typically 11 modules on each side

    95 // + 22 // 95 for barcode + 22 for quiet zones
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
    use crate::{
        DecodedBitmap, Element, Length,
        barcode::ean13::{check_ean_content, ean13_check_digit},
        generate_ean13, generate_ean13_text,
    };

    #[test]
    fn should_calculate_ean_check_digit() {
        let ean12 = "123456789112";
        let check = ean13_check_digit(ean12).unwrap();
        assert_eq!(check, 5);
    }

    #[test]
    fn should_error_on_less_than_12_digits() {
        let ean11 = "12345678911";
        assert!(ean13_check_digit(ean11).is_err());
    }

    #[test]
    fn should_error_on_more_than_12_digits() {
        let ean13 = "1234567891123";
        assert!(ean13_check_digit(ean13).is_err());
    }

    #[test]
    fn should_return_return_ean_untouched() {
        let ean13 = "1234567891123";
        let output = check_ean_content(ean13).unwrap();
        assert_eq!(ean13, output);
    }

    #[test]
    fn should_return_pad_ean_on_missing_digits() {
        let input = "123456789";
        let output = check_ean_content(input).unwrap();
        assert_eq!("0001234567895", output);
    }

    #[test]
    fn should_return_pad_ean_on_non_digits() {
        let input = "1abc56789";
        let output = check_ean_content(input).unwrap();
        assert_eq!("0001000567890", output);
    }

    #[test]
    fn should_correctly_split_ean_text() {
        let ean13 = "1234567891123";
        let texts =
            generate_ean13_text(Length(1.), Length(2.), ean13, &DecodedBitmap::default()).unwrap();
        let Element::Text { content, .. } = texts[0].clone() else {
            panic!("invalid")
        };
        assert_eq!(content, "1");

        let Element::Text { content, .. } = texts[1].clone() else {
            panic!("invalid")
        };
        assert_eq!(content, "234567");

        let Element::Text { content, .. } = texts[2].clone() else {
            panic!("invalid")
        };
        assert_eq!(content, "891123");
    }

    #[test]
    fn should_add_correct_num_of_row_to_ean() {
        // width must be a multiple of 95
        let bmp = generate_ean13(95, 15, "").unwrap();
        assert_eq!(bmp.height, Length(17.));
        assert_eq!(bmp.width, Length(95.));
        assert_eq!(
            bmp.pixels.len(),
            (bmp.width.as_f32() * bmp.height.as_f32()) as usize
        );
    }
}
