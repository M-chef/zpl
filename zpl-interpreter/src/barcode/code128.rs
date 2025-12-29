use std::error::Error;

use rxing::{BarcodeFormat, EncodeHintValue, EncodeHints, Writer, oned::Code128Writer};
use zpl_parser::Justification;

use crate::{BarcodeContent, barcode::bitmap_from_bitmatrix};

pub(super) fn generate_code_128(
    width: Option<u8>,
    contents: &str,
    height: Option<usize>,
) -> Result<BarcodeContent, Box<dyn Error>> {
    let writer = Code128Writer::default();
    let modules = estimate_code128_modules(contents);
    let width = width.unwrap_or(2) as usize * modules / 2;
    let height = height.unwrap_or(10);
    let bit_matrix = writer.encode_with_hints(
        contents,
        &BarcodeFormat::CODE_128,
        width as i32,
        height as i32,
        &EncodeHints::default().with(EncodeHintValue::Margin("0".into())),
    )?;
    let bitmap = bitmap_from_bitmatrix(bit_matrix)?;
    let font_width = (bitmap.width / contents.chars().count()) as f32 * 0.8;
    let mut barcode_content = BarcodeContent {
        font_width,
        text_elements: vec![],
        bitmap,
    };
    let text_x = barcode_content.bitmap.width as isize / 2;
    let text_y = { font_width * 0.2 } as isize;
    barcode_content.add_text_element(text_x, text_y, contents.to_string(), Justification::Auto);
    Ok(barcode_content)
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
