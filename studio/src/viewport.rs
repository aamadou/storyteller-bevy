use bevy::window::WindowResolution;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct ViewportSize {
    pub width: f32,
    pub height: f32,
}

#[wasm_bindgen]
impl ViewportSize {
    #[wasm_bindgen(constructor)]
    pub fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }
}

impl Default for ViewportSize {
    fn default() -> Self {
        WindowResolution::default().into()
    }
}

impl From<ViewportSize> for WindowResolution {
    fn from(value: ViewportSize) -> Self {
        Self::new(value.width, value.height)
    }
}

impl From<WindowResolution> for ViewportSize {
    fn from(value: WindowResolution) -> Self {
        Self {
            width: value.width(),
            height: value.height(),
        }
    }
}
