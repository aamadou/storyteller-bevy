#![allow(clippy::type_complexity)]

use bevy::prelude::*;
use input::StudioInputPlugin;
use interaction::InteractionPlugin;
use orbit_cam::OrbitCamPlugin;
use skybox::SkyboxPlugin;
use test_scene::TestScenePlugin;
use ui::UiLayerPlugin;
use viewport::ViewportSize;
use wasm_bindgen::prelude::*;

mod design_system;
mod input;
mod interaction;
mod math;
mod orbit_cam;
mod skybox;
mod test_scene;
mod ui;
mod viewport;

#[cfg(target_arch = "wasm32")]
mod wasm;

#[wasm_bindgen]
pub fn init(viewport_size: ViewportSize) {
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    let mut app = App::new();

    #[cfg(target_arch = "wasm32")]
    app.add_plugins(viewport::wasm::ViewportPlugin);

    app.add_plugins((
        DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: viewport_size.into(),
                title: "Storyteller Studio".into(),
                ..default()
            }),
            ..default()
        }),
        StudioInputPlugin,
        InteractionPlugin,
        OrbitCamPlugin,
        TestScenePlugin,
        SkyboxPlugin,
        UiLayerPlugin,
    ))
    .run();
}
