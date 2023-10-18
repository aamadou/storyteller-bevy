use bevy::prelude::Vec3;

pub trait NearlyEq<Rhs, Tolerance = Rhs> {
    const DEFAULT_TOLERANCE: Tolerance;

    #[inline]
    fn nearly_eq(self, rhs: Rhs) -> bool
    where
        Self: Sized,
    {
        self.nearly_eq_within(rhs, Self::DEFAULT_TOLERANCE)
    }

    fn nearly_eq_within(self, rhs: Rhs, tolerance: Tolerance) -> bool;
}

impl NearlyEq<f32> for f32 {
    const DEFAULT_TOLERANCE: f32 = f32::EPSILON;

    fn nearly_eq_within(self, rhs: f32, tolerance: f32) -> bool {
        (self - rhs).abs() < tolerance
    }
}

impl NearlyEq<Vec3, f32> for Vec3 {
    const DEFAULT_TOLERANCE: f32 = <f32 as NearlyEq<f32>>::DEFAULT_TOLERANCE;

    fn nearly_eq_within(self, rhs: Vec3, tolerance: f32) -> bool {
        self.x.nearly_eq_within(rhs.x, tolerance)
            && self.y.nearly_eq_within(rhs.y, tolerance)
            && self.z.nearly_eq_within(rhs.z, tolerance)
    }
}
