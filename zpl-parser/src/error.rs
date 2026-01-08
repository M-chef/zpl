use std::fmt::{self, Display};

#[derive(Debug, Clone, PartialEq)]
pub enum ParseErrorKind {
    InvalidSyntax,
    IncompleteInput,
    MissingCommand,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParseError {
    pub kind: ParseErrorKind,
    pub message: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            ParseErrorKind::InvalidSyntax => write!(f, "Invalid syntax: {}...", self.message),
            ParseErrorKind::IncompleteInput => write!(f, "Incomplete input: {}", self.message),
            ParseErrorKind::MissingCommand => {
                write!(f, "Missing command: {}", self.message)
            }
        }
    }
}

impl std::error::Error for ParseError {}

impl<T: Display> From<nom::Err<nom::error::Error<T>>> for ParseError {
    fn from(value: nom::Err<nom::error::Error<T>>) -> Self {
        match value {
            nom::Err::Incomplete(needed) => {
                let message = match needed {
                    nom::Needed::Unknown => "Unknown".to_string(),
                    nom::Needed::Size(non_zero) => non_zero.to_string(),
                };
                ParseError {
                    kind: ParseErrorKind::IncompleteInput,
                    message,
                }
            }
            nom::Err::Error(err) | nom::Err::Failure(err) => {
                let input = err.input.to_string();
                let message = input.chars().take(10).collect();
                ParseError {
                    kind: ParseErrorKind::InvalidSyntax,
                    message,
                }
            }
        }
    }
}
