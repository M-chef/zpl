mod code128;
mod ean13;

use std::error::Error;

use rxing::common::BitMatrix;
use zpl_parser::BarcodeType;

use crate::{BarcodeConfig, BarcodeContent, DecodedBitmap};

use code128::generate_code_128;
use ean13::generate_ean13;

pub(super) fn barcode_from_content(
    barcode_config: Option<&BarcodeConfig>,
    barcode_type: BarcodeType,
    contents: &str,
) -> Result<BarcodeContent, Box<dyn Error>> {
    let height = {
        let height = barcode_type.height();
        if height.is_none() {
            barcode_config.map(|state| state.height)
        } else {
            height
        }
    };

    let width = barcode_config.map(|state| state.width);

    let barcode_content = match barcode_type {
        BarcodeType::Code39 => todo!(),
        BarcodeType::Code128 { .. } => generate_code_128(width, contents, height)?,
        BarcodeType::Pdf417 => todo!(),
        BarcodeType::Ean8 => todo!(),
        BarcodeType::Ean13 { .. } => generate_ean13(width, contents, height)?,
        BarcodeType::Qr => todo!(),
        BarcodeType::DataMatrix => todo!(),
    };

    Ok(barcode_content)
}

fn bitmap_from_bitmatrix(bitmatrix: BitMatrix) -> Result<DecodedBitmap, Box<dyn Error>> {
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

// /// relative position of text for barcode
// /// in percentage of font height
// pub fn relative_text_ypos(barcode_type: &BarcodeType) -> f32 {
//     match barcode_type {
//         BarcodeType::Code39 => todo!(),
//         BarcodeType::Code128 { .. } => 0.,
//         BarcodeType::Pdf417 => todo!(),
//         BarcodeType::Ean8 => todo!(),
//         BarcodeType::Ean13 { .. } => -0.5,
//         BarcodeType::Qr => todo!(),
//         BarcodeType::DataMatrix => todo!(),
//     }
// }
