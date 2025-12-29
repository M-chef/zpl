#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Code128Mode {
    Normal,
    Ucc,
    Auto,
    Ean,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BarcodeType {
    Code39,
    Code128 {
        orientation: super::Orientation,
        height: Option<usize>,
        show_text: bool,
        text_above: bool,
        check_digit: bool,
        mode: Code128Mode,
    },
    Pdf417,
    Ean8,
    Ean13 {
        orientation: super::Orientation,
        height: Option<usize>,
        show_text: bool,
        text_above: bool,
    },
    Qr,
    DataMatrix,
}

impl BarcodeType {
    pub fn height(&self) -> Option<usize> {
        match self {
            BarcodeType::Code39 => todo!(),
            BarcodeType::Code128 { height, .. } => *height,
            BarcodeType::Pdf417 => todo!(),
            BarcodeType::Ean8 => todo!(),
            BarcodeType::Ean13 { height, .. } => *height,
            BarcodeType::Qr => todo!(),
            BarcodeType::DataMatrix => todo!(),
        }
    }

    pub fn show_content(&self) -> bool {
        match self {
            BarcodeType::Code39 => todo!(),
            BarcodeType::Code128 { show_text, .. } => *show_text,
            BarcodeType::Pdf417 => todo!(),
            BarcodeType::Ean8 => todo!(),
            BarcodeType::Ean13 { show_text, .. } => *show_text,
            BarcodeType::Qr => todo!(),
            BarcodeType::DataMatrix => todo!(),
        }
    }
}
