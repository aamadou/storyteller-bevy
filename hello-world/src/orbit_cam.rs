use std::f32::consts;

use bevy::{
    core_pipeline::prepass::{DepthPrepass, NormalPrepass},
    input::mouse::{MouseButton, MouseMotion},
    prelude::*,
    window::PrimaryWindow,
};

pub struct OrbitCamPlugin;

impl Plugin for OrbitCamPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera)
            .add_systems(Update, pan_orbit_camera);
    }
}

#[derive(Component)]
pub struct OrbitCam {
    pub focus: Vec3,
    pub radius: f32,
    pub upside_down: bool,
}

impl OrbitCam {
    // TODO: Make these configurable
    const ORBIT_BTN: MouseButton = MouseButton::Left;
    const PAN_BTN: MouseButton = MouseButton::Middle;
    const ZOOM_BTN: MouseButton = MouseButton::Right;
}

impl Default for OrbitCam {
    fn default() -> Self {
        Self {
            focus: Vec3::ZERO,
            radius: 10.0,
            upside_down: false,
        }
    }
}

enum ControlMode {
    Orbit,
    Pan,
    Zoom,
}

fn spawn_camera(mut cmd: Commands) {
    let camera_xform = Transform::from_xyz(-2.5, 3.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y);

    cmd.spawn((
        Camera3dBundle {
            transform: camera_xform,
            ..default()
        },
        DepthPrepass,
        NormalPrepass,
        OrbitCam {
            radius: camera_xform.translation.length(),
            ..default()
        },
    ));
}

pub fn pan_orbit_camera(
    window: Query<&Window, With<PrimaryWindow>>,
    mut mouse_motion: EventReader<MouseMotion>,
    mouse_buttons: Res<Input<MouseButton>>,
    keyboard: Res<Input<KeyCode>>,
    mut camera: Query<(&mut OrbitCam, &mut Transform, &Projection)>,
) {
    if !(keyboard.pressed(KeyCode::AltLeft) || keyboard.pressed(KeyCode::AltRight)) {
        return;
    }

    let (mut orbit, mut xform, proj) = camera.single_mut();

    if mouse_buttons.just_pressed(OrbitCam::ORBIT_BTN)
        || mouse_buttons.just_released(OrbitCam::ORBIT_BTN)
    {
        orbit.upside_down = xform.up().y <= 0.0;
    }

    let control_mode = if mouse_buttons.pressed(OrbitCam::ORBIT_BTN) {
        ControlMode::Orbit
    } else if mouse_buttons.pressed(OrbitCam::ZOOM_BTN) {
        ControlMode::Zoom
    } else if mouse_buttons.pressed(OrbitCam::PAN_BTN) {
        ControlMode::Pan
    } else {
        return;
    };

    let window = window.single();
    let mouse_delta = mouse_motion
        .into_iter()
        .fold(Vec2::ZERO, |accum, current| accum + current.delta);

    match control_mode {
        ControlMode::Orbit => {
            do_orbit(window, mouse_delta, &mut xform, &orbit);
        }
        ControlMode::Pan => {
            do_pan(proj, window, mouse_delta, &mut xform, &mut orbit);
        }
        ControlMode::Zoom => {
            let delta = (mouse_delta.y * 1.5) / window.height();
            orbit.radius -= orbit.radius * delta;
            xform.translation = orbit.focus + xform.back() * orbit.radius;
        }
    }
}

#[inline]
fn do_orbit(win: &Window, mouse_delta: Vec2, xform: &mut Transform, orbit: &OrbitCam) {
    // Translate to origin
    xform.translation += xform.forward() * orbit.radius;

    // Calculate rotation
    let delta_x = {
        let delta = mouse_delta.x / win.width() * consts::PI * 2.0;
        match orbit.upside_down {
            true => -delta,
            false => delta,
        }
    };
    let delta_y = mouse_delta.y / win.height() * consts::PI;

    let yaw = Quat::from_rotation_y(-delta_x);
    let pitch = Quat::from_rotation_x(-delta_y);

    // Rotate around global Y and local X
    xform.rotation = yaw * xform.rotation;
    xform.rotation *= pitch;

    // Translate back to the original position
    xform.translation += xform.back() * orbit.radius;
}

#[inline]
fn do_pan(
    proj: &Projection,
    win: &Window,
    mouse_delta: Vec2,
    xform: &mut Transform,
    orbit: &mut OrbitCam,
) {
    // Normalize the pan distance with respect to the window resolution
    let pan = match proj {
        Projection::Perspective(p) => {
            mouse_delta
                * Vec2 {
                    x: (p.fov * p.aspect_ratio) / win.width(),
                    y: p.fov / win.height(),
                }
        }
        _ => mouse_delta,
    };

    // Pan along local XY plane, with the distance scaled by the radius
    let up = xform.up();
    let left = xform.left();
    let delta = (pan.y * up + pan.x * left) * orbit.radius * 0.8;

    // Update camera translation and focus point
    xform.translation += delta;
    orbit.focus += delta;
}
