//! Helpers for interpolating between values.

pub use ecs::*;
pub use traits::*;

mod ecs;
mod traits;

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub enum Easing {
    Linear,
    Sine,
    SineOut,
    SineIn,
}

impl Default for Easing {
    fn default() -> Self {
        Self::Linear
    }
}
