use base64::{Engine, engine::general_purpose};
use zpl_parser::{CompressionMethod, CompressionType, ZplFormatCommand};

#[derive(Debug, Clone, Default)]
pub struct DecodedBitmap {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<u8>, // 0 = white, 1 = black
}

pub fn decode_zpl_graphic(
    // compression: CompressionType,
    compression_method: CompressionMethod,
    raw_data: &str,
    width: usize,
    height: usize,
    bytes_per_row: usize,
) -> Result<DecodedBitmap, String> {
    // Step 1: Decode ASCII hex into bytes if needed
    let binary_data = match compression_method {
        CompressionMethod::None => todo!(),
        CompressionMethod::Zlib => {
            let decompressed = decode_base64(raw_data)?;
            decompress_zlib(&decompressed)
        }
    }?;

    // Step 2: Expand packed bits into 1 byte per pixel (0/1)
    let pixels = expand_monochrome_bitmap(&binary_data, width, height, bytes_per_row)?;

    Ok(DecodedBitmap {
        width,
        height,
        pixels,
    })
}

pub fn decode_ascii_hex(s: &str) -> Result<Vec<u8>, String> {
    let mut out = Vec::new();
    let cleaned = s.trim().replace(char::is_whitespace, "");

    if cleaned.len() % 2 != 0 {
        return Err("invalid ASCII hex length".into());
    }

    for chunk in cleaned.as_bytes().chunks(2) {
        let hex = std::str::from_utf8(chunk).unwrap();
        let byte = u8::from_str_radix(hex, 16).map_err(|_| format!("invalid hex byte: {}", hex))?;
        out.push(byte);
    }
    Ok(out)
}

fn decode_base64(s: &str) -> Result<Vec<u8>, String> {
    let cleaned = s.trim().replace(char::is_whitespace, "");
    general_purpose::STANDARD
        .decode(&cleaned)
        .map_err(|e| format!("invalid base64: {}", e))
}

fn decompress_zlib(data: &[u8]) -> Result<Vec<u8>, String> {
    use flate2::read::ZlibDecoder;
    use std::io::Read;

    let mut decoder = ZlibDecoder::new(data);
    let mut out = Vec::new();
    decoder
        .read_to_end(&mut out)
        .map_err(|e| format!("zlib decompress error: {}", e))?;
    Ok(out)
}

fn expand_monochrome_bitmap(
    packed: &[u8],
    width: usize,
    height: usize,
    bytes_per_row: usize,
) -> Result<Vec<u8>, String> {
    let expected = (bytes_per_row * height) as usize;
    if packed.len() < expected {
        return Err(format!(
            "bitmap too small: expected {} bytes, got {}",
            expected,
            packed.len()
        ));
    }

    let mut pixels = Vec::with_capacity((width * height) as usize);

    for row in 0..height {
        let row_start = (row * bytes_per_row) as usize;

        for byte in &packed[row_start..row_start + bytes_per_row as usize] {
            for bit in 0..8 {
                let shift = 7 - bit;
                let value = (byte >> shift) & 1;

                if pixels.len() < (width * height) as usize {
                    pixels.push(value); // push 0 or 1
                }
            }
        }
    }
    Ok(pixels)
}
