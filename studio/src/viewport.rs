use core::fmt;

use bevy::window::WindowResolution;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Clone, Copy)]
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

impl fmt::Debug for ViewportSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("({:.2}, {:.2})", self.width, self.height))
    }
}

#[cfg(target_arch = "wasm32")]
pub mod wasm {
    use std::{cell::RefCell, sync::OnceLock};

    use bevy::prelude::*;
    use parking_lot::ReentrantMutex;
    use wasm_bindgen::prelude::*;

    use super::ViewportSize;

    static RESIZE_BUF: OnceLock<ReentrantMutex<RefCell<Option<ViewportSize>>>> = OnceLock::new();

    #[wasm_bindgen]
    pub fn resize(viewport_size: ViewportSize) {
        let buffer = RESIZE_BUF.get_or_init(|| ReentrantMutex::new(RefCell::new(None)));
        if let Some(lock) = buffer.try_lock() {
            *lock.borrow_mut() = Some(viewport_size);
        }
    }

    pub struct ViewportPlugin;

    impl Plugin for ViewportPlugin {
        fn build(&self, app: &mut App) {
            app.add_event::<ViewportResizeEvent>()
                .add_systems(PreUpdate, dispatch_viewport_events)
                .add_systems(Update, sync_window_to_viewport);
        }
    }

    #[derive(Event, Clone, Copy)]
    struct ViewportResizeEvent(ViewportSize);

    fn sync_window_to_viewport(
        mut events: EventReader<ViewportResizeEvent>,
        mut window: Query<&mut Window>,
    ) {
        let mut resize = None;
        for event in events.into_iter() {
            resize = Some(event.0);
        }

        if let Some(ViewportSize { width, height }) = resize {
            window.single_mut().resolution.set(width, height);
        }
    }

    fn dispatch_viewport_events(mut events: EventWriter<ViewportResizeEvent>) {
        let buffer = RESIZE_BUF.get_or_init(|| ReentrantMutex::new(RefCell::new(None)));
        if let Some(lock) = buffer.try_lock() {
            if let Some(resize) = lock.borrow_mut().take() {
                events.send(ViewportResizeEvent(resize));
            }
        }
    }
}
