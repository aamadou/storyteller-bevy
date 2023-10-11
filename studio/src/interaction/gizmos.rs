use std::f32::consts;

use bevy::{prelude::*, window::PrimaryWindow};

use crate::design_system;

use super::{InteractionMode, InteractionTransform, Selected, TransformSpace};

#[derive(Clone, Copy)]
enum GizmoVariant {
    Full,
    Partial,
}

pub(super) fn draw(
    q_selection: Query<&GlobalTransform, With<Selected>>,
    q_camera_xform: Query<&Transform, With<Camera>>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    mode: Res<InteractionMode>,
    mut gizmos: Gizmos,
    mut config: ResMut<GizmoConfig>,
) {
    if !q_selection.is_empty() {
        // TODO: Use an app state or something so we only have to set this up once
        let scale_factor = q_window.single().scale_factor() as f32;
        config.line_width = 4. * scale_factor;
        config.line_perspective = false;
        config.depth_bias = -1.;

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
            let size = dist / 20.;

            let variant = if mode
                .transform
                .contains(InteractionTransform::TRANSLATE | InteractionTransform::ROTATE)
            {
                GizmoVariant::Partial
            } else {
                GizmoVariant::Full
            };

            if mode.transform.contains(InteractionTransform::TRANSLATE) {
                draw_translate_gizmo(&mut gizmos, variant, &xform, mode.space, size);
            }
            if mode.transform.contains(InteractionTransform::ROTATE) {
                draw_rotate_gizmo(&mut gizmos, variant, &xform, camera_xform, mode.space, size);
            }
        }
    }
}

// FIXME: We might need more control over the Z-ordering / depth bias than the
//        Bevy gizmos give us. :/

#[inline]
fn draw_translate_gizmo(
    gizmos: &mut Gizmos,
    variant: GizmoVariant,
    xform: &Transform,
    space: TransformSpace,
    size: f32,
) {
    let origin = xform.translation;
    let basis = match space {
        TransformSpace::Local => xform,
        TransformSpace::World => &Transform::IDENTITY,
    };

    match variant {
        GizmoVariant::Full => {
            gizmos.sphere(origin, Quat::IDENTITY, size * 0.125, Color::WHITE);

            for (direction, color) in [
                (basis.local_x(), design_system::Color::RED),
                (basis.local_y(), design_system::Color::GREEN),
                (basis.local_z(), design_system::Color::BLUE),
            ] {
                let end = origin + direction * size * 1.4;
                gizmos.line(origin, end, color);

                let cone_base = origin + direction * size;
                let radius = size * 0.1;
                let length = size * 0.4;
                gizmos.cone(cone_base, direction, radius, length, color);
            }
        }
        GizmoVariant::Partial => {
            for (direction, color) in [
                (basis.local_x(), design_system::Color::RED),
                (basis.local_y(), design_system::Color::GREEN),
                (basis.local_z(), design_system::Color::BLUE),
            ] {
                let cone_base = origin + direction * size * 1.4;
                let radius = size * 0.1;
                let length = size * 0.4;
                gizmos.cone(cone_base, direction, radius, length, color);
            }
        }
    }
}

#[inline]
fn draw_rotate_gizmo(
    gizmos: &mut Gizmos,
    variant: GizmoVariant,
    xform: &Transform,
    camera_xform: &Transform,
    space: TransformSpace,
    radius: f32,
) {
    let origin = xform.translation;
    let basis = match space {
        TransformSpace::Local => xform,
        TransformSpace::World => &Transform::IDENTITY,
    };
    let camera_fwd = camera_xform.forward();

    // Draw a screen-space rotator ring
    gizmos.circle(origin, camera_xform.forward(), radius * 1.2, Color::WHITE);

    draw_rotator_axis_rings(
        gizmos,
        RotatorRings::Near,
        origin,
        camera_fwd,
        basis,
        radius,
    );

    if matches!(variant, GizmoVariant::Full) {
        draw_rotator_axis_rings(gizmos, RotatorRings::Far, origin, camera_fwd, basis, radius);
    }
}

enum RotatorRings {
    Near,
    Far,
}

#[inline]
fn draw_rotator_axis_rings(
    gizmos: &mut Gizmos,
    which: RotatorRings,
    origin: Vec3,
    camera_fwd: Vec3,
    basis: &Transform,
    radius: f32,
) {
    for (normal, color) in [
        (basis.local_x(), design_system::Color::RED),
        (basis.local_y(), design_system::Color::GREEN),
        (basis.local_z(), design_system::Color::BLUE),
    ] {
        let (color, perp) = match which {
            RotatorRings::Near => (color, Vec3::cross(normal, camera_fwd)),
            RotatorRings::Far => (color.with_a(0.2), Vec3::cross(camera_fwd, normal)),
        };

        let start = origin + perp.normalize() * radius;

        gizmos.arc(origin, normal, start, consts::PI, color);
    }
}

trait GizmoExtras {
    /// Draw an arc around `center` with angular `length` (in radians),
    /// beginning at `point` and progressing counter-clockwise around `axis`
    /// (if axis were pointed toward the viewer). The radius is derived from the
    /// distance between `center` and `start`.
    fn arc(&mut self, center: Vec3, axis: Vec3, start: Vec3, length: f32, color: Color);

    fn cone(&mut self, base: Vec3, normal: Vec3, radius: f32, length: f32, color: Color);
}

impl<'a> GizmoExtras for Gizmos<'a> {
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

    fn cone(&mut self, base: Vec3, normal: Vec3, radius: f32, length: f32, color: Color) {
        self.circle(base, normal, radius, color);

        let angle_inc = (consts::PI * 2.) / 32.;
        let rotator = Mat3::from_axis_angle(normal, angle_inc);
        let perp = normal.any_orthonormal_pair().0;
        let first = base + perp * radius;
        let end = base + normal * length;

        let mut vector = first - base;
        for _ in 0..32 {
            let start = base + vector;
            self.line(start, end, color);
            vector = rotator * vector;
        }
    }
}
