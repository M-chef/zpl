use nom::error::{Error, ErrorKind};

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

#[derive(Debug, Clone, PartialEq)]
pub enum Orientation {
    Normal,     // 0°
    Rotate,     // 90°
    Invert,     // 180°
    BackRotate, // 270°
}

#[derive(Debug, Clone, PartialEq)]
pub enum ZplFormatCommand {
    LabelLength(usize),
    PrintWidth(usize),
    LabelShift(i32),
    Font {
        name: char,
        orientation: Orientation,
        height: usize,
        width: usize,
    },
    FieldOrigin {
        x: i32,
        y: i32,
        justification: u8,
    },
    FieldTypeset {
        x: i32,
        y: i32,
        justification: u8,
    },
    FieldData(String),
    GraficField {
        compression_type: CompressionType,
        data_bytes: usize,
        total_bytes: usize,
        row_bytes: usize,
        data: GraficData,
    },
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
