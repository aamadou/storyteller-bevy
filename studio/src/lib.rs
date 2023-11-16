#![allow(clippy::type_complexity, clippy::too_many_arguments)]

use bevy::prelude::*;
use bevy_mod_picking::DefaultPickingPlugins;
use camera_controller::CameraControllerPlugin;
pub use debug::{DebugMode, DebugPlugin};
use input::StudioInputPlugin;
use interaction::InteractionPlugin;
pub use math::InterpolationPlugin;
pub use scene::{ActiveScene, Inspectable, ScenePlugin};
pub use skybox::{SkyboxId, SkyboxPlugin};
use ui::UiLayerPlugin;
use viewport::ViewportSize;
use wasm_bindgen::prelude::*;

mod camera_controller;
mod debug;
pub mod design_system;
mod input;
mod interaction;
pub mod math;
mod scene;
mod skybox;
mod ui;
mod viewport;

#[cfg(feature = "wasm")]
mod wasm;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct MainCamera;

#[wasm_bindgen]
pub fn init(viewport_size: ViewportSize) {
    #[cfg(feature = "wasm")]
    console_error_panic_hook::set_once();

    let mut app = App::new();
    app.register_type::<MainCamera>();

    #[cfg(feature = "wasm")]
    app.add_plugins(viewport::wasm::ViewportPlugin);

    app.add_plugins((
        DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: viewport_size.into(),
                title: "Storyteller Studio".into(),
                canvas: Some("#studio_canvas".into()),
                ..default()
            }),
            ..default()
        }),
        DefaultPickingPlugins,
        DebugPlugin,
        InteractionPlugin,
        StudioInputPlugin,
        InterpolationPlugin,
        CameraControllerPlugin,
        ScenePlugin,
        SkyboxPlugin,
        UiLayerPlugin,
    ))
    .run();
}
