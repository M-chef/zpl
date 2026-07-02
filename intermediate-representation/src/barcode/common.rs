use std::error::Error;

use rxing::common::BitMatrix;

use crate::{DecodedBitmap, Length};

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
        width: Length(width as f32),
        height: Length(height as f32),
        pixels,
    })
}
