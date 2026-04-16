use std::error::Error;

use rxing::common::BitMatrix;

use crate::DecodedBitmap;

#[derive(Debug, Clone)]
pub enum Symbology {
    Code39,
    Code128,
    Ean13,
    Qr,
    DataMatrix,
}

/// Convert rxing BitMatrix to a Bitmap with pixels
pub(crate) fn bitmap_from_bitmatrix(bitmatrix: BitMatrix) -> Result<DecodedBitmap, Box<dyn Error>> {
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

// /// Computes the largest font size at which `text` fits within `max_width`,
// /// optionally capped by `max_height`.
// ///
// /// This function has no barcode-specific knowledge — the caller is responsible
// /// for deriving `max_width` and `max_height` from the symbology and barcode
// /// geometry.
// pub fn fit_text_to_width(
//     text: &str,
//     font: &str,
//     max_width: f32,
//     max_height: Option<f32>,
//     ctx: &LoweringContext,
// ) -> f32 {
//     let (mut lo, mut hi) = (1.0_f32, max_width);
//     while hi - lo > 0.5 {
//         let mid = (lo + hi) / 2.0;
//         let text_width = ctx.fonts.measure_text_dimensions(font, text, mid).width;
//         if text_width <= max_width {
//             lo = mid;
//         } else {
//             hi = mid;
//         }
//     }
//     match max_height {
//         Some(h) => lo.min(h),
//         None => lo,
//     }
// }
