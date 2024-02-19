use bevy::prelude::*;

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

pub struct BlockingPlugin;

impl Plugin for BlockingPlugin {
    fn build(&self, _app: &mut App) {}
}

#[cfg_attr(feature = "wasm", wasm_bindgen(js_name = test_blocking))]
pub fn test_blocking() -> bool {
    info!("test_blocking");
    true
}
