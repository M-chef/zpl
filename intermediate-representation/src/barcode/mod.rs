mod code128;
mod common;
mod ean13;

/// Width ratios for HRI text relative to barcode width, per symbology.
/// These represent how much of the barcode width the HRI text should occupy.
pub mod hri_ratios {
    pub const CODE128: f32 = 0.06;
    pub const CODE39: f32 = 0.6;
    pub const EAN13: f32 = 0.07; // EAN13 HRI spans almost full width
    pub const DEFAULT: f32 = 0.5;
}

pub use code128::*;
pub use common::*;
pub use ean13::*;
