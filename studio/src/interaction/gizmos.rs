use std::f32::consts;

use bevy::{prelude::*, window::PrimaryWindow};

use crate::design_system;

use super::{InteractionMode, InteractionTransform, Selected, TransformSpace};

pub(super) fn draw(
    q_selection: Query<&GlobalTransform, With<Selected>>,
    q_camera_xform: Query<&Transform, With<Camera>>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    interaction_mode: Res<InteractionMode>,
    mut gizmos: Gizmos,
    mut config: ResMut<GizmoConfig>,
) {
    if !q_selection.is_empty() {
        // TODO: Use an app state or something so we only have to set this up once
        let scale_factor = q_window.single().scale_factor() as f32;
        config.line_width = 4. * scale_factor;
        config.line_perspective = false;
        config.depth_bias = -1.;

        let InteractionMode { transform, space } = *interaction_mode;
        let camera_xform = q_camera_xform.single();

        // TODO: We only want to render one gizmo -- this is just for testing
        for world_xform in q_selection.iter() {
            let xform = world_xform.compute_transform();
            // Note: Scaling the gizmo size by this isn't perfect due to
            // distortion from the perspective projection. It bugged me at
            // first, but even Blender exhibits this type of gizmo distortion if
            // you dial the camera's focal length way down, so I'm gonna call it
            // good enough.
            let dist = Vec3::distance(camera_xform.translation, xform.translation);

            if transform.contains(InteractionTransform::TRANSLATE) {
                draw_translate_gizmo(&xform, &mut gizmos, space, dist / 12.5);
            }
            if transform.contains(InteractionTransform::ROTATE) {
                draw_rotate_gizmo(&xform, camera_xform, &mut gizmos, space, dist / 20.);
            }
        }
    }
}

#[inline]
fn draw_translate_gizmo(xform: &Transform, gizmos: &mut Gizmos, space: TransformSpace, size: f32) {
    let origin = xform.translation;
    let basis = match space {
        TransformSpace::Local => xform,
        TransformSpace::World => &Transform::IDENTITY,
    };

    let mut draw_axis = |direction: Vec3, color: Color| {
        let end = origin + direction * size;
        gizmos.line(origin, end, color);
    };

    draw_axis(basis.local_x(), design_system::Color::RED);
    draw_axis(basis.local_y(), design_system::Color::GREEN);
    draw_axis(basis.local_z(), design_system::Color::BLUE);
}

#[inline]
fn draw_rotate_gizmo(
    xform: &Transform,
    camera_xform: &Transform,
    gizmos: &mut Gizmos,
    space: TransformSpace,
    radius: f32,
) {
    let origin = xform.translation;
    let basis = match space {
        TransformSpace::Local => xform,
        TransformSpace::World => &Transform::IDENTITY,
    };

    // Draw a screen-space rotator ring
    gizmos.circle(origin, camera_xform.forward(), radius * 1.2, Color::WHITE);

    // Draw a semicircle for each of the X, Y and Z axes
    for (normal, color) in [
        (basis.local_x(), design_system::Color::RED),
        (basis.local_y(), design_system::Color::GREEN),
        (basis.local_z(), design_system::Color::BLUE),
    ] {
        let perp = Vec3::cross(normal, camera_xform.forward()).normalize();
        let start = origin + perp * radius;

        gizmos.arc(origin, normal, start, consts::PI, color);
    }
}

trait Arc {
    /// Draw an arc around `center` with angular `length` (in radians),
    /// beginning at `point` and progressing counter-clockwise around `axis`
    /// (if axis were pointed toward the viewer). The radius is derived from the
    /// distance between `center` and `start`.
    fn arc(&mut self, center: Vec3, axis: Vec3, start: Vec3, length: f32, color: Color);
}

impl<'a> Arc for Gizmos<'a> {
    fn arc(&mut self, center: Vec3, normal: Vec3, start: Vec3, length: f32, color: Color) {
        let two_pi = consts::PI * 2.;
        let segments = f32::round(32. * (length / two_pi));
        let angle_inc = length / segments;
        let rotator = Mat3::from_axis_angle(normal, angle_inc);

        let mut vector = start - center;
        for _ in 0..(segments as usize) {
            let next = rotator * vector;
            self.line(center + vector, center + next, color);
            vector = next;
        }
    }
}
