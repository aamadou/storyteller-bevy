use core::fmt;

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Clone, Copy, Serialize, Deserialize)]
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
        Self {
            width: 1280.,
            height: 720.,
        }
    }
}

impl fmt::Debug for ViewportSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("({:.2}, {:.2})", self.width, self.height))
    }
}

#[cfg(feature = "wasm")]
pub mod wasm {
    use bevy::{prelude::*, render::camera::Viewport};
    use wasm_bindgen::prelude::*;

    use crate::{
        wasm::{StaticBuffer, WasmMessageBuffer},
        MainCamera,
    };

    use super::ViewportSize;

    static RESIZE_BUF: WasmMessageBuffer<(ViewportSize, f32)> = WasmMessageBuffer::new();

    #[wasm_bindgen]
    pub fn resize(viewport_size: ViewportSize, scale_factor: f32) {
        RESIZE_BUF.write_message((viewport_size, scale_factor));
    }

    pub struct ViewportPlugin;

    impl Plugin for ViewportPlugin {
        fn build(&self, app: &mut App) {
            app.add_event::<ViewportResizeEvent>().add_systems(
                PreUpdate,
                (dispatch_resize_events, sync_window_to_viewport).chain(),
            );
        }
    }

    #[derive(Event, Clone, Copy)]
    struct ViewportResizeEvent {
        pub size: ViewportSize,
        pub scale_factor: f32,
    }

    fn sync_window_to_viewport(
        mut events: EventReader<ViewportResizeEvent>,
        mut window: Query<&mut Window>,
        mut q_camera: Query<&mut Camera, With<MainCamera>>,
    ) {
        if let Some(ViewportResizeEvent { size, scale_factor }) = events.read().last() {
            let ViewportSize { width, height } = size;

            window.single_mut().resolution.set(*width, *height);
            q_camera.single_mut().viewport = Some(Viewport {
                physical_size: UVec2::new(
                    (width * scale_factor) as u32,
                    (height * scale_factor) as u32,
                ),
                ..Default::default()
            });
            info!("Viewport size: ({width:.2}, {height:.2}) [{scale_factor:.2}]");
        }
    }

    fn dispatch_resize_events(mut events: EventWriter<ViewportResizeEvent>) {
        if let Some((size, scale_factor)) = RESIZE_BUF.take_message() {
            events.send(ViewportResizeEvent { size, scale_factor });
            info!(
                "Dispatch Viewport size: ({:.2}, {:.2}) [{scale_factor:.2}]",
                size.width, size.height,
            );
        }
    }
}
