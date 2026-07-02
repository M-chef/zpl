use std::ops::Mul;

use crate::measure::Length;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct FontSize(pub(crate) f32);

impl FontSize {
    pub fn as_f32(&self) -> f32 {
        self.0
    }
}

impl From<Length> for FontSize {
    fn from(value: Length) -> Self {
        Self(value.0)
    }
}

impl Mul<f32> for FontSize {
    type Output = Self;

    fn mul(mut self, rhs: f32) -> Self::Output {
        self.0 *= rhs;
        self
    }
}
