use std::fmt::Display;

#[derive(Debug, Clone, PartialEq)]
pub enum ZplErrorKind {
    ParseError(zpl_parser::ParseError),
    InterpretError,
    RenderError,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZplError {
    kind: ZplErrorKind,
    message: String,
}

impl From<zpl_parser::ParseError> for ZplError {
    fn from(value: zpl_parser::ParseError) -> Self {
        Self {
            kind: ZplErrorKind::ParseError(value),
            message: String::new(),
        }
    }
}

impl Display for ZplError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            ZplErrorKind::ParseError(ref parse_error) => write!(f, "Parse error: {parse_error}"),
            ZplErrorKind::InterpretError => todo!(),
            ZplErrorKind::RenderError => todo!(),
        }
    }
}

impl std::error::Error for ZplError {}
