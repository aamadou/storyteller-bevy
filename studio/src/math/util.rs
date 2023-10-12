use std::f32::consts;

use bevy::prelude::Vec3;

pub trait NearlyEq<Rhs> {
    fn nearly_eq(self, rhs: Rhs) -> bool;
}

impl NearlyEq<f32> for f32 {
    fn nearly_eq(self, rhs: f32) -> bool {
        (self - rhs).abs() < Self::EPSILON
    }
}

impl NearlyEq<Vec3> for Vec3 {
    fn nearly_eq(self, rhs: Vec3) -> bool {
        self.x.nearly_eq(rhs.x) && self.y.nearly_eq(rhs.y) && self.z.nearly_eq(rhs.z)
    }
}

/// Fit a [0, 1] `t` interpolation parameter to a sine curve
///
/// https://www.desmos.com/calculator/k9x4c4cr5y
#[inline]
pub fn curve_sin(t: f32) -> f32 {
    -1. * (f32::cos(t * consts::PI) / 2.) + 0.5
}
