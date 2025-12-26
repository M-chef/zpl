use std::error::Error;

use rxing::{BarcodeFormat, EncodeHintValue, EncodeHints, Writer, oned::Code128Writer};
use zpl_parser::Justification;

use crate::{BarcodeContent, barcode::bitmap_from_bitmatrix};

pub(super) fn generate_code_128(
    // barcode_config: Option<&BarcodeState>,
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
    let font_width = (bitmap.width / contents.chars().count()) as f32;
    Ok(BarcodeContent {
        text_x: 0,
        text_y: 0,
        text_y_shift: -0.5,
        font_width,
        justification: Justification::Auto,
        text: Some(contents.to_string()),
        bitmap,
    })
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
