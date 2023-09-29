#![allow(clippy::type_complexity)]

use bevy::prelude::*;
use wasm_bindgen::prelude::*;

mod orbit_cam;
mod test_scene;
mod ui;

use orbit_cam::{OrbitCam, OrbitCamPlugin};
use test_scene::TestScenePlugin;
use ui::UiLayerPlugin;

#[wasm_bindgen]
pub fn init() {
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    App::new()
        .add_plugins((
            DefaultPlugins,
            OrbitCamPlugin,
            TestScenePlugin,
            UiLayerPlugin,
        ))
        .add_systems(Update, draw_world_axes.after(orbit_cam::pan_orbit_camera))
        .run();
}

fn draw_world_axes(mut gizmos: Gizmos, q: Query<&OrbitCam>) {
    let &OrbitCam { focus, radius, .. } = q.single();

    let mut draw_axis = |direction: Vec3, color: Color| {
        let end = focus + direction * radius / 15.;
        gizmos.line(focus, end, color);
    };

    draw_axis(Vec3::X, Color::RED);
    draw_axis(Vec3::Y, Color::GREEN);
    draw_axis(Vec3::Z, Color::BLUE);
}
