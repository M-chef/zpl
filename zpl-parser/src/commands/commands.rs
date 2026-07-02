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
pub enum Alignment {
    #[default]
    Left, // 0
    Right, // 1
    Auto,  // 2
}

impl From<Option<u8>> for Alignment {
    fn from(value: Option<u8>) -> Self {
        match value {
            Some(u) if u == 0 => Alignment::Left,
            Some(u) if u == 1 => Alignment::Right,
            Some(u) if u == 2 => Alignment::Auto,
            Some(_) => Alignment::Left,
            None => Alignment::Left,
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
    Resolution(u32),
}

impl From<&str> for ClockMode {
    fn from(value: &str) -> Self {
        match value {
            "S" => ClockMode::Start,
            "T" => ClockMode::Now,
            _ => match value.parse::<u32>() {
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
    LabelHome {
        x: u32,
        y: u32,
    },
    LabelLength(u32),
    PrintWidth(u32),
    LabelShift(u32),
    BarcodeConfig {
        width: u8,
        width_ratio: f32,
        height: u32,
    },
    Barcode(super::BarcodeType),
    ChangeFont {
        name: char,
        height: u32,
        width: u32,
    },
    Font {
        name: char,
        orientation: Orientation,
        height: u32,
        width: u32,
    },
    FieldOrigin {
        x: u32,
        y: u32,
        justification: Alignment,
    },
    FieldTypeset {
        x: u32,
        y: u32,
        justification: Alignment,
    },
    FieldData(String),
    GraphicField {
        compression_type: CompressionType,
        data_bytes: u32,
        total_bytes: u32,
        row_bytes: u32,
        data: GraficData,
    },
    GraphicalBox {
        width: u32,
        height: u32,
        thickness: u32,
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
        width: u32,
        lines: u32,
        line_spacing: isize,
        justification: TextBlockJustification,
        hanging_indent: u32,
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
        year: Option<u32>,
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
