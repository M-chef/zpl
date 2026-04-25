use std::error::Error;

use rxing::{BarcodeFormat, EncodeHintValue, EncodeHints, Writer, oned::EAN13Writer};

use crate::{
    Alignment, DecodedBitmap, Element, Justification, OCR_B, YReference,
    barcode::bitmap_from_bitmatrix, hri_ratios,
};

pub(crate) fn generate_ean13(
    target_width: usize,
    target_height: usize,
    content: &str,
) -> Result<DecodedBitmap, Box<dyn Error>> {
    let content = check_ean_content(content)?;
    let writer = EAN13Writer::default();

    let bitmatrix = writer.encode_with_hints(
        &content,
        &BarcodeFormat::EAN_13,
        target_width as i32,
        target_height as i32,
        &EncodeHints::default().with(EncodeHintValue::Margin("0".into())),
    )?;

    let module_width = target_width as f32 / ean13_modules(&content) as f32;
    // let rows = bitmatrix.getRowSize();
    // let last_row = bitmatrix.getRow(rows as u32);

    let mut bitmap = bitmap_from_bitmatrix(bitmatrix)?;
    bitmap.enlarge_modules_for_ean13(module_width as usize);

    Ok(bitmap)
}

pub(crate) fn generate_ean13_text(
    x: f32,
    y: f32,
    data: &str,
    bmp: &DecodedBitmap,
) -> Result<Vec<Element>, Box<dyn Error>> {
    let data = check_ean_content(data)?;
    let font_size = hri_ratios::EAN13 * bmp.width as f32;

    let y = y + bmp.height as f32 + font_size * 0.6;
    let first = Element::Text {
        x,
        y,
        max_width: Some(bmp.width as f32),
        lines: 1,
        font: OCR_B.to_string(),
        font_size,
        content: data[0..1].to_string(),
        // content: "test".to_string(),
        alignment: Alignment::Right,
        justification: Justification::Left,
        y_reference: YReference::Baseline,
        inverted: false,
    };
    let second = Element::Text {
        // x: x + bmp.width as f32 / 4.,
        x,
        y,
        max_width: Some(bmp.width as f32 / 2.),
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
        x: x + bmp.width as f32 / 2.,
        y,
        max_width: Some(bmp.width as f32 / 2.),
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

// pub(crate) fn generate_ean13_text(
//     text: &str,
//     bounds: Rect,
//     ctx: &LoweringContext,
// ) -> Result<Vec<DrawCommand>, Box<dyn Error>> {
//     let var_name = OCR_B;
//     let font = var_name;

//     let text = check_ean_content(text)?;

//     let hri_width = bounds.width * hri_ratios::EAN13;
//     let font_size = fit_text_to_width(&text, font, hri_width, None, ctx);

//     let elements = create_text_elements(&text, bounds, ctx, font, font_size)?;
//     let mut boxes = {
//         let second = elements.iter().nth(1).unwrap();
//         let second_box = DrawCommand::FilledRect {
//             bounds: second.screen_bounds(&ctx.fonts),
//             rounding: 0.,
//             color: crate::Color::White,
//             inverted: false,
//         };

//         vec![second_box]
//     };
//     boxes.extend(elements);
//     Ok(boxes)
// }

// fn create_text_elements(
//     text: &str,
//     bounds: Rect,
//     ctx: &LoweringContext,
//     font: &str,
//     font_size: f32,
// ) -> Result<Vec<DrawCommand>, Box<dyn Error>> {
//     let y = bounds.y + 0.5 * font_size;

//     let first = &text[0..1];
//     let first_bounds = {
//         let mut first_bounds = ctx.fonts.measure_text_dimensions(OCR_B, first, font_size);
//         first_bounds.x = bounds.x - first_bounds.width;
//         first_bounds.y = y;
//         first_bounds
//     };
//     let first_text = create_text(first_bounds, ctx, font, font_size, first);

//     let second = &text[1..7];
//     let second_bounds = {
//         let mut second_bounds = ctx.fonts.measure_text_dimensions(OCR_B, second, font_size);
//         second_bounds.x = bounds.x + first_bounds.width;
//         second_bounds.y = y;
//         second_bounds
//     };
//     let second_text = create_text(second_bounds, ctx, font, font_size, second);

//     let third = &text[7..];
//     let third_bounds = {
//         let mut third_bounds = ctx.fonts.measure_text_dimensions(OCR_B, second, font_size);
//         third_bounds.x = bounds.x + bounds.width / 2. + font_size;
//         third_bounds.y = y;
//         third_bounds
//     };
//     let third_text = create_text(third_bounds, ctx, font, font_size, third);

//     Ok(vec![first_text, second_text, third_text])
// }

// fn create_text(
//     bounds: Rect,
//     ctx: &LoweringContext,
//     font: &str,
//     font_size: f32,
//     first: &str,
// ) -> DrawCommand {
//     let glyphs = to_glyphs(
//         font,
//         font_size,
//         bounds,
//         None,
//         Justification::Center,
//         YReference::Ascent,
//         first,
//         ctx,
//     );
//     DrawCommand::Text {
//         font: font.to_string(),
//         font_size,
//         glyphs,
//         bold: false,
//         inverted: false,
//     }
// }

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
