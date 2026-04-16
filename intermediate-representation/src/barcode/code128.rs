use std::error::Error;

use rxing::{BarcodeFormat, EncodeHintValue, EncodeHints, Writer, oned::Code128Writer};

use crate::{DecodedBitmap, barcode::bitmap_from_bitmatrix};

pub(crate) fn generate_code_128(
    target_width: usize,
    target_height: usize,
    contents: &str,
) -> Result<DecodedBitmap, Box<dyn Error>> {
    let writer = Code128Writer::default();
    let bit_matrix = writer.encode_with_hints(
        contents,
        &BarcodeFormat::CODE_128,
        target_width as i32,
        target_height as i32,
        &EncodeHints::default().with(EncodeHintValue::Margin("0".into())),
    )?;
    let bitmap = bitmap_from_bitmatrix(bit_matrix)?;

    Ok(bitmap)
}

// pub(crate) fn generate_code_128_text(
//     text: &str,
//     mut bounds: Rect,
//     ctx: &LoweringContext,
// ) -> Result<Vec<DrawCommand>, Box<dyn Error>> {
//     let font = OCR_B;

//     let hri_width = bounds.width * hri_ratios::CODE128;
//     let font_size = fit_text_to_width(text, font, hri_width, None, ctx);

//     let text_bounds = ctx.fonts.measure_text_dimensions(OCR_B, &text, font_size);
//     let height = text_bounds.height * 0.8;

//     bounds.y += height;

//     let glyphs = to_glyphs(
//         font,
//         font_size,
//         bounds,
//         None,
//         Justification::Center,
//         YReference::Ascent,
//         text,
//         ctx,
//     );
//     Ok(vec![DrawCommand::Text {
//         font: font.to_string(),
//         font_size,
//         glyphs,
//         bold: false,
//         inverted: false,
//     }])
// }

pub fn estimate_code128_modules(data: &str) -> usize {
    // Code128 structure:
    // - Start code: 11 modules
    // - Each character: 11 modules
    // - Checksum: 11 modules
    // - Stop pattern: 13 modules
    // - Quiet zone: typically 10 modules on each side

    let char_count = data.len();
    11 + (char_count * 11) + 11 + 13 // + 20 // +20 for quiet zones
}
