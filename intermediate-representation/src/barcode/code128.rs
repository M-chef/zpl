use std::error::Error;

use rxing::{BarcodeFormat, EncodeHintValue, EncodeHints, Writer, oned::Code128Writer};

use crate::{
    Alignment, DecodedBitmap, Element, Justification, OCR_B, YReference,
    barcode::bitmap_from_bitmatrix,
    hri_ratios,
    measure::{FontSize, Length},
};

pub(crate) fn generate_code_128(
    target_width: i32,
    target_height: i32,
    contents: &str,
) -> Result<DecodedBitmap, Box<dyn Error>> {
    let writer = Code128Writer::default();
    let bit_matrix = writer.encode_with_hints(
        contents,
        &BarcodeFormat::CODE_128,
        target_width,
        target_height,
        &EncodeHints::default().with(EncodeHintValue::Margin("0".into())),
    )?;
    let bitmap = bitmap_from_bitmatrix(bit_matrix)?;

    Ok(bitmap)
}

pub(crate) fn generate_code_128_text(
    x: Length,
    y: Length,
    data: &str,
    bmp: &DecodedBitmap,
) -> Vec<Element> {
    let font_size = bmp.width * hri_ratios::CODE128;
    let y = y + bmp.height + font_size * 1.2;
    let font_size: FontSize = font_size.into();
    vec![Element::Text {
        x,
        y,
        max_width: Some(bmp.width),
        lines: 1,
        font: OCR_B.to_string(),
        font_size,
        content: data.to_string(),
        alignment: Alignment::Left,
        justification: Justification::Center,
        y_reference: YReference::Baseline,
        inverted: false,
    }]
}

pub fn estimate_code128_modules(data: &str) -> usize {
    // Code128 structure:
    // - Start code: 11 modules
    // - Each character: 11 modules
    // - Checksum: 11 modules
    // - Stop pattern: 13 modules
    // - Quiet zone: typically 10 modules on each side

    let char_count = data.len();
    11 + (char_count * 11) + 11 + 13 //+ 20 // +20 for quiet zones
}

#[cfg(test)]
mod tests {}
