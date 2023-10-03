use std::f32::consts;

use bevy::{
    input::mouse::{MouseButton, MouseMotion},
    prelude::*,
    window::{CursorGrabMode, PrimaryWindow},
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
            camera: Camera {
                hdr: true,
                ..default()
            },
            ..default()
        },
        OrbitCam {
            radius: camera_xform.translation.length(),
            ..default()
        },
    ));
}

pub fn pan_orbit_camera(
    mut window: Query<&mut Window, With<PrimaryWindow>>,
    mut mouse_motion: EventReader<MouseMotion>,
    mouse_buttons: Res<Input<MouseButton>>,
    keyboard: Res<Input<KeyCode>>,
    mut camera: Query<(&mut OrbitCam, &mut Transform, &Projection)>,
) {
    if (keyboard.just_released(KeyCode::AltLeft) && !keyboard.pressed(KeyCode::AltRight))
        || (keyboard.just_released(KeyCode::AltRight) && !keyboard.pressed(KeyCode::AltLeft))
        || mouse_buttons.just_released(OrbitCam::ORBIT_BTN)
        || mouse_buttons.just_released(OrbitCam::PAN_BTN)
        || mouse_buttons.just_released(OrbitCam::ZOOM_BTN)
    {
        let mut window = window.single_mut();
        window.cursor.visible = true;
        window.cursor.grab_mode = CursorGrabMode::None;
    }

    if !(keyboard.pressed(KeyCode::AltLeft)
        || keyboard.pressed(KeyCode::AltRight)
        || mouse_buttons.pressed(OrbitCam::PAN_BTN))
    {
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

    let mouse_delta = mouse_motion
        .into_iter()
        .fold(Vec2::ZERO, |accum, current| accum + current.delta);

    // TODO: This is nice because it allows movement input beyond the limits of
    //       the viewport, but it's a little awkward that the cursor doesn't
    //       move while confined and hidden. We have a `set_cursor_position`
    //       API, so maybe do the Blender thing and let the cursor wrap around
    //       at the viewport edges?
    let mut window = window.single_mut();
    window.cursor.visible = false;
    window.cursor.grab_mode = CursorGrabMode::Confined;

    let window = window.as_ref();

    match control_mode {
        ControlMode::Orbit => {
            do_orbit(window, mouse_delta, &mut xform, &orbit);
        }
        ControlMode::Pan => {
            do_pan(proj, window, mouse_delta, &mut xform, &mut orbit);
        }
        ControlMode::Zoom => {
            let delta = (mouse_delta.y * 1.5) / (window.physical_height() as f32);
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
        let delta = mouse_delta.x / (win.physical_width() as f32) * consts::PI * 2.0;
        match orbit.upside_down {
            true => -delta,
            false => delta,
        }
    };
    let delta_y = mouse_delta.y / (win.physical_height() as f32) * consts::PI;

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
                    x: (p.fov * p.aspect_ratio) / (win.physical_width() as f32),
                    y: p.fov / (win.physical_height() as f32),
                }
        }
        _ => mouse_delta,
    };

    // Pan along local XY plane, with the distance scaled by the radius
    let up = xform.up();
    let left = xform.left();
    let delta = (pan.y * up + pan.x * left) * orbit.radius;

    // Update camera translation and focus point
    xform.translation += delta;
    orbit.focus += delta;
}
