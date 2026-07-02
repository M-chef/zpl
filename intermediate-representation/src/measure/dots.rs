use std::ops::{Add, Sub};

use crate::{FontSize, measure::Length};

/// Unit of measures for dot based parsers.
///
/// Dot based parsers should use `Dots` and then convert
/// it to `Length`.
///
/// When constructing `Dots` with `from_signed` / `from_unsigned` negative values
/// will be converted to positive integers. Caller can obtain
/// information from convertion by looking at ParseContext.
///
/// # Example
/// ```rust
/// use intermediate_representation::*;
///
/// let dots = Dots::from_unsigned(10);
/// let length = dots.to_length();
/// assert_eq!(length.as_f32(), 10.);
///
/// //constructing from negative value
/// let dots = Dots::from_signed(-50);
/// let length = dots.to_length();
/// assert_eq!(length.as_f32(), 50.);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Dots(u32);

impl Dots {
    /// Get `Dots` from a signed integer
    pub fn from_signed(raw: i32) -> Self {
        let v = raw.unsigned_abs();
        Dots(v)
    }

    /// Get `Dots` from a unsigned integer
    pub fn from_unsigned(raw: u32) -> Self {
        Dots(raw)
    }

    pub fn to_length(self) -> Length {
        Length(self.0 as f32)
    }

    pub fn to_fontsize(self) -> FontSize {
        FontSize(self.0 as f32)
    }
}

impl Add for Dots {
    type Output = Self;

    fn add(mut self, rhs: Self) -> Self::Output {
        self.0 += rhs.0;
        self
    }
}

impl Sub for Dots {
    type Output = Self;

    fn sub(mut self, rhs: Self) -> Self::Output {
        self.0 -= rhs.0;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_dots() {
        let a = Dots(1);
        let b = Dots(1);
        assert_eq!(a + b, Dots(2))
    }

    #[test]
    fn sub_dots() {
        let a = Dots(1);
        let b = Dots(1);
        assert_eq!(a - b, Dots(0))
    }
}
