use std::collections::HashMap;

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
pub enum TextBlockJustification {
    #[default]
    Left,
    Center,
    Right,
    Justified,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ClockMode {
    #[default]
    Start,
    Now,
    Resolution(usize),
}

impl From<&str> for ClockMode {
    fn from(value: &str) -> Self {
        match value {
            "S" => ClockMode::Start,
            "T" => ClockMode::Now,
            _ => match value.parse::<usize>() {
                Ok(n) => ClockMode::Resolution(n),
                Err(_) => ClockMode::Start,
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ClockLanguage {
    #[default]
    English,
    Spanish,
    French,
    German,
    Italian,
    Norwegian,
    Portuguese,
    Swedish,
    Danish,
    Spanish2,
    Dutch,
    Finnish,
    Japanese,
    Korean,
    SimplifiedChinese,
    TraditionalChinese,
    Russian,
    Polish,
    Czech,
    Romanian,
}

impl From<Option<u8>> for ClockLanguage {
    fn from(value: Option<u8>) -> Self {
        value
            .map(|l| match l {
                1 => ClockLanguage::English,
                2 => ClockLanguage::Spanish,
                3 => ClockLanguage::French,
                4 => ClockLanguage::German,
                5 => ClockLanguage::Italian,
                6 => ClockLanguage::Norwegian,
                7 => ClockLanguage::Portuguese,
                8 => ClockLanguage::Swedish,
                9 => ClockLanguage::Danish,
                10 => ClockLanguage::Spanish2,
                11 => ClockLanguage::Dutch,
                12 => ClockLanguage::Finnish,
                13 => ClockLanguage::Japanese,
                14 => ClockLanguage::Korean,
                15 => ClockLanguage::SimplifiedChinese,
                16 => ClockLanguage::TraditionalChinese,
                17 => ClockLanguage::Russian,
                18 => ClockLanguage::Polish,
                19 => ClockLanguage::Czech,
                20 => ClockLanguage::Romanian,
                _ => ClockLanguage::English,
            })
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ClockFormat {
    AM,
    PM,
    #[default]
    Military,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ZplFormatCommand {
    LabelLength(usize),
    PrintWidth(usize),
    LabelShift(usize),
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
        x: usize,
        y: usize,
        justification: Justification,
    },
    FieldTypeset {
        x: usize,
        y: usize,
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
    FieldHexIndicator {
        char: char,
    },
    CharacterSet {
        num: u8,
        mapping: HashMap<u8, u8>,
    },
    FieldBlock {
        width: usize,
        lines: usize,
        line_spacing: isize,
        justification: TextBlockJustification,
        hanging_indent: usize,
    },
    RealTimeClockMode {
        mode: ClockMode,
        language: ClockLanguage,
    },
    RealTimeClockEscapeChar {
        first: char,
        second: Option<char>,
        third: Option<char>,
    },
    FieldSeparator,
    SetRealTimeClock {
        month: Option<u8>,
        day: Option<u8>,
        year: Option<usize>,
        hour: Option<u8>,
        minute: Option<u8>,
        second: Option<u8>,
        format: ClockFormat,
    },
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
