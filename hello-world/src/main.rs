use bevy::prelude::*;

mod orbit_cam;
mod people;
mod test_scene;

use orbit_cam::{OrbitCam, OrbitCamPlugin};
use test_scene::TestScenePlugin;

fn main() {
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    App::new()
        .add_plugins((
            DefaultPlugins,
            people::Plugin,
            OrbitCamPlugin,
            TestScenePlugin,
        ))
        .add_systems(Startup, hello_world)
        .add_systems(Update, draw_world_axes.after(orbit_cam::pan_orbit_camera))
        .run();
}

fn hello_world() {
    info!("Hello, world!");
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
