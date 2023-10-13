use std::{f32::consts, ops};

use super::NearlyEq;

pub trait Lerp {
    fn lerp(self, a: Self, b: Self) -> Self;
    fn inv_lerp(self, a: Self, b: Self) -> Self;
    fn remap(self, from: (Self, Self), to: (Self, Self)) -> Self
    where
        Self: Sized;
}

#[rustfmt::skip] trait One { fn one() -> Self; }
#[rustfmt::skip] impl One for f32 { fn one() -> Self { 1. } }
#[rustfmt::skip] impl One for f64 { fn one() -> Self { 1_f64 } }

#[rustfmt::skip] trait Zero { fn zero() -> Self; }
#[rustfmt::skip] impl Zero for f32 { fn zero() -> Self { 0. } }
#[rustfmt::skip] impl Zero for f64 { fn zero() -> Self { 0_f64 } }

// lol it would have been faster and less code to just copy/paste. At least I tried.
impl<T> Lerp for T
where
    T: ops::Add<Output = T>
        + ops::Sub<Output = T>
        + ops::Mul<Output = T>
        + ops::Div<Output = T>
        + NearlyEq<T>
        + One
        + Zero
        + Sized
        + Copy,
{
    fn lerp(self, a: Self, b: Self) -> Self {
        (<Self as One>::one() - self) * a + b * self
    }

    fn inv_lerp(self, a: Self, b: Self) -> Self {
        let dividend = b - a;
        if !dividend.nearly_eq(<Self as Zero>::zero()) {
            (self - a) / dividend
        } else {
            b
        }
    }

    fn remap(self, from: (Self, Self), to: (Self, Self)) -> Self
    where
        Self: Sized,
    {
        let t = self.inv_lerp(from.0, from.1);
        t.lerp(to.0, to.1)
    }
}

/// Fit a [0, 1] `t` interpolation parameter to a sine curve
///
/// https://www.desmos.com/calculator/k9x4c4cr5y
#[inline]
pub fn curve_sin(t: f32) -> f32 {
    -1. * (f32::cos(t * consts::PI) / 2.) + 0.5
}
