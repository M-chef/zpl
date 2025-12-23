use nom::{
    IResult,
    error::{Error, ErrorKind},
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CompressionType {
    Ascii,
    Binary,
    Compressed,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CompressionMethod {
    None,
    Zlib,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GraficData {
    pub compression_method: CompressionMethod,
    pub data: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Orientation {
    Normal,     // 0°
    Rotate,     // 90°
    Invert,     // 180°
    BackRotate, // 270°
}

impl Orientation {
    pub fn try_from_str(value: &str) -> IResult<&str, Self> {
        let orientation = match value {
            "N" => Orientation::Normal,
            "R" => Orientation::Rotate,
            "I" => Orientation::Invert,
            "B" => Orientation::BackRotate,
            _ => return Err(nom::Err::Error(Error::new("", ErrorKind::NoneOf))),
        };

        Ok(("", orientation))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Justification {
    #[default]
    Left, // 0
    Right, // 1
    Auto,  // 2
}

impl From<Option<u8>> for Justification {
    fn from(value: Option<u8>) -> Self {
        match value {
            Some(u) if u == 0 => Justification::Left,
            Some(u) if u == 1 => Justification::Right,
            Some(u) if u == 2 => Justification::Auto,
            Some(_) => Justification::Left,
            None => Justification::Left,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Color {
    #[default]
    Black,
    White,
}

impl From<Option<&str>> for Color {
    fn from(value: Option<&str>) -> Self {
        match value {
            Some(c) if c == "B" => Self::Black,
            Some(c) if c == "W" => Self::White,
            Some(_) => Self::Black,
            None => Self::Black,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ZplFormatCommand {
    LabelLength(usize),
    PrintWidth(usize),
    LabelShift(i32),
    BarcodeConfig {
        width: u8,
        width_ratio: f32,
        height: usize,
    },
    Barcode(super::BarcodeType),
    ChangeFont {
        name: char,
        height: usize,
        width: usize,
    },
    Font {
        name: char,
        orientation: Orientation,
        height: usize,
        width: usize,
    },
    FieldOrigin {
        x: i32,
        y: i32,
        justification: Justification,
    },
    FieldTypeset {
        x: i32,
        y: i32,
        justification: Justification,
    },
    FieldData(String),
    GraphicField {
        compression_type: CompressionType,
        data_bytes: usize,
        total_bytes: usize,
        row_bytes: usize,
        data: GraficData,
    },
    GraphicalBox {
        width: usize,
        height: usize,
        thickness: usize,
        color: Color,
        rounding: u8,
    },
    Inverted,
    FieldSeparator,
}

pub enum ZplHostCommand {
    CancelAllCommands,    // ~JA
    CancelCurrentCommand, // ~JC
    PrintHostStatus,      // ~HS
    DownloadGraphics,     // ~DG
}

pub enum ZplCommand {
    Format(ZplFormatCommand),
    Host(ZplHostCommand),
}
