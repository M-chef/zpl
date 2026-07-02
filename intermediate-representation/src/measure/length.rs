use std::ops::{Add, AddAssign, Deref, Div, Mul};

use crate::measure::FontSize;

/// Unit of measures for internal representation
/// and will be used as pixels in renderers. Renderers may
/// scale final output based on `Dots`.
///
/// Parsers should not use `Length` directly but use one of the
/// other unit structs (`Dots`, `Mm`) and use the `to_lenght` to
/// convert it to `Length`.
///
/// # Example
/// ```rust
/// use intermediate_representation::*;
///
/// let dots = Dots::from_unsigned(10);
/// let length = dots.to_length();
/// assert_eq!(length.as_f32(), 10.);
///
/// let mm = Mm::from_raw(10.);
/// let dpi = 300.;
/// let length = mm.to_length(dpi);
/// assert_eq!(length.as_f32(), 10. * 300. / 25.4)
/// ```
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct Length(pub(crate) f32);

impl Length {
    pub fn as_u32(&self) -> u32 {
        self.0.round() as u32
    }

    pub fn as_i32(&self) -> i32 {
        self.0.round() as i32
    }

    pub fn as_f32(&self) -> f32 {
        self.0
    }
}

impl From<FontSize> for Length {
    fn from(value: FontSize) -> Self {
        Self(value.0 as f32)
    }
}

impl Add for Length {
    type Output = Self;

    fn add(mut self, rhs: Self) -> Self::Output {
        self.0 += rhs.0;
        self
    }
}

impl Add<f32> for Length {
    type Output = Self;

    fn add(mut self, rhs: f32) -> Self::Output {
        self.0 += rhs;
        self
    }
}

impl AddAssign for Length {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl AddAssign<f32> for Length {
    fn add_assign(&mut self, rhs: f32) {
        self.0 += rhs
    }
}

impl Mul for Length {
    type Output = Self;

    fn mul(mut self, rhs: Self) -> Self::Output {
        self.0 *= rhs.0;
        self
    }
}

impl Mul<f32> for Length {
    type Output = Self;

    fn mul(mut self, rhs: f32) -> Self::Output {
        self.0 *= rhs;
        self
    }
}

impl Mul<Length> for f32 {
    type Output = Length;

    fn mul(self, mut rhs: Length) -> Self::Output {
        rhs.0 *= self;
        rhs
    }
}

impl Div for Length {
    type Output = Self;

    fn div(mut self, rhs: Self) -> Self::Output {
        self.0 /= rhs.0;
        self
    }
}

impl Div<f32> for Length {
    type Output = Self;

    fn div(mut self, rhs: f32) -> Self::Output {
        self.0 /= rhs;
        self
    }
}

#[cfg(test)]
mod tests {
    use crate::measure::Length;

    #[test]
    fn should_divide_dots() {
        let left = Length(2.);
        let result = left / 2.;
        assert_eq!(result, Length(1.));
    }
}
