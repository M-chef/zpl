use crate::measure::Length;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Mm(f32);

impl Mm {
    pub fn from_raw(raw: f32) -> Self {
        Self(raw)
    }

    pub fn to_length(self, dpi: f32) -> Length {
        Length(self.0 * dpi / 25.4)
    }
}
