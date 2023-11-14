#![allow(clippy::type_complexity, clippy::too_many_arguments)]

use bevy::prelude::*;
use wasm_bindgen::prelude::*;

#[cfg(not(feature = "headless"))]
mod camera_controller;
#[cfg(not(feature = "headless"))]
mod debug;
#[cfg(not(feature = "headless"))]
mod design_system;
#[cfg(not(feature = "headless"))]
mod input;
#[cfg(not(feature = "headless"))]
mod interaction;
mod math;
mod scene;
mod skybox;
#[cfg(not(feature = "headless"))]
mod ui;
mod viewport;

#[cfg(not(feature = "headless"))]
use crate::{
    camera_controller::CameraControllerPlugin, debug::DebugPlugin, input::StudioInputPlugin,
    interaction::InteractionPlugin, ui::UiLayerPlugin,
};
use crate::{
    math::InterpolationPlugin, scene::ScenePlugin, skybox::SkyboxPlugin, viewport::ViewportSize,
};

#[cfg(feature = "wasm")]
mod wasm;

#[derive(Component)]
pub(crate) struct MainCamera;

#[wasm_bindgen]
pub fn init(viewport_size: ViewportSize) {
    #[cfg(feature = "wasm")]
    console_error_panic_hook::set_once();

    let mut app = App::new();

    #[cfg(feature = "wasm")]
    app.add_plugins(viewport::wasm::ViewportPlugin);

    #[cfg(not(feature = "headless"))]
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

    #[cfg(feature = "headless")]
    app.add_plugins((
        bevy::log::LogPlugin::default(),
        bevy::core::TaskPoolPlugin::default(),
        bevy::core::TypeRegistrationPlugin,
        bevy::time::TimePlugin,
        bevy::transform::TransformPlugin,
        bevy::hierarchy::HierarchyPlugin,
        bevy::diagnostic::DiagnosticsPlugin,
        InterpolationPlugin,
        ScenePlugin,
        SkyboxPlugin,
    ))
    .run();
}
