use std::f32::consts;

use bevy::{prelude::*, window::PrimaryWindow};

use crate::math::Lerp;

use super::{
    model::{Arc, Axes, Control, Gizmo, GizmoColor, Ring, Shape},
    InteractionMode, StudioGizmoConfig, TransformSpace, TransformType,
};

pub(super) fn configure(
    q_window: Query<&Window, (Changed<Window>, With<PrimaryWindow>)>,
    mut render_config: ResMut<GizmoConfig>,
    user_config: Res<StudioGizmoConfig>,
) {
    if let Ok(scale_factor) = q_window.get_single().map(|win| win.scale_factor() as f32) {
        render_config.line_width = user_config.line_thickness * scale_factor;
        render_config.line_perspective = false;
        render_config.depth_bias = -0.1;
    }
}

pub(super) fn draw(
    q_gizmo: Query<(&Gizmo, &GlobalTransform)>,
    q_camera_xform: Query<&Transform, With<Camera>>,
    mut gizmos: Gizmos,
    mode: Res<InteractionMode>,
) {
    let cam_xform = q_camera_xform.single();

    let Ok((gizmo, world_xform)) = q_gizmo.get_single() else {
        return;
    };

    let mut cached_xform: Option<Transform> = None;

    for el in gizmo.iter() {
        match (el.shape, el.color, el.control) {
            (
                Shape::Ring(ring),
                GizmoColor::Single(color),
                Control::Rotation(Axes::X | Axes::Y | Axes::Z),
            ) => {
                let (front, back) = bisect_rotator_ring(&ring, cam_xform);
                gizmos.arc(front.center, front.axis, front.start, front.length, color);

                if !mode.transform.intersects(TransformType::TRANSLATE) {
                    gizmos.arc(
                        back.center,
                        back.axis,
                        back.start,
                        back.length,
                        color.with_a(0.2),
                    );
                }
            }
            #[rustfmt::skip] (
                Shape::Ring(ring),
                GizmoColor::Single(color),
                _,
            ) => {
                gizmos.circle(ring.origin, ring.normal, ring.radius, color);
            }
            #[rustfmt::skip] (
                shape @ (Shape::Line(_) | Shape::Cone(_)),
                color,
                Control::Translation(axes),
            ) => {
                let cam_pos = cam_xform.translation;
                let xform = match cached_xform {
                    Some(xform) => xform,
                    None => {
                        let xform = world_xform.compute_transform();
                        cached_xform = Some(xform);
                        xform
                    }
                };

                let basis = match mode.space {
                    TransformSpace::Local => xform,
                    TransformSpace::World => Transform::from_translation(xform.translation),
                };

                let alpha = fade_aligned_control(axes, &basis, cam_pos);
                if alpha.nearly_eq(0.) {
                    continue;
                }

                match (shape, color) {
                    (Shape::Line(line), GizmoColor::Single(color)) => {
                        let (start, end) = line.into();
                        gizmos.line(start, end, color.with_a(alpha));
                    }
                    (Shape::Line(line), GizmoColor::Gradient(start_color, end_color)) => {
                        let (start, end) = line.into();
                        gizmos.line_gradient(
                            start,
                            end,
                            start_color.with_a(alpha),
                            end_color.with_a(alpha),
                        );
                    }
                    (Shape::Cone(cone), GizmoColor::Single(color)) => {
                        gizmos.cone(
                            cone.base,
                            cone.direction,
                            cone.radius,
                            cone.length,
                            color.with_a(alpha),
                        );
                    }
                    (shape, GizmoColor::Gradient(_, _)) => {
                        warn!("Unexpected gradient color specified for {shape:?}");
                    }
                    _ => unreachable!(),
                }
            }
            (shape, color, control) => {
                warn!("Invalid combination of gizmo properties: {shape:?}, {color:?}, {control:?}");
            }
        }
    }
}

/// Slice a non-screen-space rotator ring in half by the camera's XY plane,
/// returning the semicircle closer to the camera as the first tuple element,
/// and the one further away as the second.
#[inline]
fn bisect_rotator_ring(ring: &Ring, cam_xform: &Transform) -> (Arc, Arc) {
    let plane_intersection = Vec3::cross(ring.normal, cam_xform.forward()).normalize();
    let start_a = ring.origin + plane_intersection * ring.radius;
    let start_b = ring.origin - plane_intersection * ring.radius;

    let arc_a = Arc {
        center: ring.origin,
        axis: ring.normal,
        start: start_a,
        length: consts::PI,
    };

    let mut arc_b = arc_a;
    arc_b.start = start_b;

    (arc_a, arc_b)
}

/// Compute an alpha value to fade the translation arrows out when they're
/// pointed directly at or away from the camera.
///
/// This prevents overlapping/obscuring the center screen-space translation
/// ring, or just generally getting in the way when the control is not very
/// useful to grab onto.
#[inline]
fn fade_aligned_control(axes: Axes, xform: &Transform, cam_pos: Vec3) -> f32 {
    let to_camera = (cam_pos - xform.translation).normalize_or_zero();
    let direction = match axes {
        Axes::X | Axes::YZ => xform.local_x(),
        Axes::Y | Axes::XZ => xform.local_y(),
        Axes::Z | Axes::XY => xform.local_z(),
        _ => panic!("Unexpected `axes` value provided to `fade_aligned_control`: {axes:?}"),
    };

    match axes {
        Axes::X | Axes::Y | Axes::Z => (1. - Vec3::dot(to_camera, direction).abs())
            .remap((0.05, 0.15), (0., 1.))
            .clamp(0., 1.),
        Axes::XY | Axes::XZ | Axes::YZ => Vec3::dot(to_camera, direction)
            .abs()
            .remap((0.25, 0.45), (0., 1.))
            .clamp(0., 1.),
        _ => unreachable!(),
    }
}

trait GizmoExtras {
    /// Draw an arc around `center` with angular `length` (in radians),
    /// beginning at `start` and progressing counter-clockwise around `axis`
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
        let perp = normal.any_orthonormal_vector();
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
