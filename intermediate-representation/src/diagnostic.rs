pub trait DiagnosticSink {
    fn warn(&mut self, diag: Diagnostic);
}

pub struct Corrected<T> {
    pub value: T,
    pub original: i32,
}

/// Interprets a signed raw value as an unsigned one.
/// Negative input is corrected via sign-flip (abs), not clamped to 0.
pub(crate) fn correct_unsigned(raw: i32) -> (u32, Option<Corrected<u32>>) {
    if raw.is_negative() {
        let corrected = raw.unsigned_abs();
        (
            corrected,
            Some(Corrected {
                value: corrected,
                original: raw,
            }),
        )
    } else {
        (raw as u32, None)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Warning,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    // or, if you track line/col instead of byte offsets:
    // pub line: u32,
    // pub column: u32,
}

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub message: String,
    pub severity: Severity,
    pub span: Option<Span>,
}

impl Diagnostic {
    pub fn correction(corrected: Corrected<u32>, span: Option<Span>) -> Self {
        Diagnostic {
            message: format!(
                "{} is invalid input, used {}",
                corrected.original, corrected.value,
            ),
            severity: Severity::Warning,
            span,
        }
    }
}

#[derive(Default)]
pub struct ParseContext {
    diagnostics: Vec<Diagnostic>,
}

impl DiagnosticSink for ParseContext {
    fn warn(&mut self, diag: Diagnostic) {
        self.diagnostics.push(diag);
    }
}
