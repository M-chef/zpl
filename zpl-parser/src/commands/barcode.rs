use nom::combinator::Opt;

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
            BarcodeType::Code128 {
                orientation,
                height,
                show_text,
                text_above,
                check_digit,
                mode,
            } => *show_text,
            BarcodeType::Pdf417 => todo!(),
            BarcodeType::Ean8 => todo!(),
            BarcodeType::Ean13 {
                orientation,
                height,
                show_text,
                text_above,
            } => *show_text,
            BarcodeType::Qr => todo!(),
            BarcodeType::DataMatrix => todo!(),
        }
    }

    /// relative position of text for barcode
    /// in percentage of font height
    pub fn relative_text_ypos(&self) -> f32 {
        match self {
            BarcodeType::Code39 => todo!(),
            BarcodeType::Code128 {
                orientation,
                height,
                show_text,
                text_above,
                check_digit,
                mode,
            } => 0.,
            BarcodeType::Pdf417 => todo!(),
            BarcodeType::Ean8 => todo!(),
            BarcodeType::Ean13 {
                orientation,
                height,
                show_text,
                text_above,
            } => -0.5,
            BarcodeType::Qr => todo!(),
            BarcodeType::DataMatrix => todo!(),
        }
    }
}
