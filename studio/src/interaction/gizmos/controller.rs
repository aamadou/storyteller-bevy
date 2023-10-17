use core::fmt;
use std::f32::consts;

use bevy::{prelude::*, window::PrimaryWindow};
use bevy_rapier3d::prelude::*;

use crate::{math::NearlyEq, orbit_cam::OrbitCam};

use super::{
    model::{self, Axes, Control, Gizmo, Hovered},
    InteractionMode, TransformSpace, TransformType,
};

#[derive(Component, Deref, DerefMut, Clone, Copy, Debug)]
pub(super) struct GizmoController(pub Control);

pub(super) fn update_spawns(
    mut cmd: Commands,
    q_needs_spawn: Query<(Entity, &Gizmo), Without<GizmoController>>,
    q_needs_despawn: Query<Entity, (With<GizmoController>, Without<Gizmo>)>,
    mode: Res<InteractionMode>,
) {
    for (ent, gizmo) in q_needs_spawn.iter() {
        let mut ent_cmd = cmd.entity(ent);

        ent_cmd.insert((RigidBody::KinematicPositionBased, Sensor));

        if mode.transform.contains(TransformType::ROTATE) {
            ent_cmd.insert(GizmoController(Control::Rotation(Axes::XYZ)));

            let Some(radius) = gizmo.iter().find_map(|el| match el.control {
                Control::Rotation(axes) if !axes.contains(Axes::SCREEN) => match el.shape {
                    model::Shape::Ring(ring) => Some(ring.radius),
                    _ => None,
                },
                _ => None,
            }) else {
                return;
            };

            ent_cmd.insert(Collider::ball(radius));
        }
    }

    for ent in q_needs_despawn.iter() {
        cmd.entity(ent)
            .remove::<(GizmoController, RigidBody, Sensor)>();
    }
}

pub(super) fn update(mut q_collider: Query<(&mut Collider, &Gizmo, &GizmoController)>) {
    for (mut collider, gizmo, ctrl) in q_collider.iter_mut() {
        match **ctrl {
            Control::Rotation(_) => {
                let Some(radius) = gizmo.iter().find_map(|el| match el.control {
                    Control::Rotation(axes) if !axes.contains(Axes::SCREEN) => match el.shape {
                        model::Shape::Ring(ring) => Some(ring.radius),
                        _ => None,
                    },
                    _ => None,
                }) else {
                    continue;
                };

                let Some(mut ball) = collider.as_ball_mut() else {
                    continue;
                };

                ball.set_radius(radius);
            }
            _ => {
                // TODO
            }
        }
    }
}

pub(super) fn hit_test(
    mut cmd: Commands,
    q_gizmo: Query<(Entity, &GlobalTransform, &GizmoController)>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform), With<OrbitCam>>,
    rapier_cx: Res<RapierContext>,
    mode: Res<InteractionMode>,
) {
    let win = q_window.single();
    let (cam, cam_xform) = q_camera.single();

    if let Some(ray) = win
        .cursor_position()
        .and_then(|pos| cam.viewport_to_world(cam_xform, pos))
    {
        if let Some((ent, t)) = rapier_cx.cast_ray(
            ray.origin,
            ray.direction,
            f32::MAX,
            false,
            QueryFilterFlags::EXCLUDE_SOLIDS.into(),
        ) {
            let hit_pos = ray.get_point(t);

            if let Ok((_, world_xform, _gizmo_ctrl)) = q_gizmo.get(ent) {
                let cam_xform = cam_xform.compute_transform();
                let world_xform_mat = world_xform.compute_matrix();
                let world_xform = Transform::from_matrix(world_xform_mat);
                let basis = match mode.space {
                    TransformSpace::Local => world_xform_mat,
                    TransformSpace::World => Mat4::from_translation(world_xform.translation),
                };

                let normal = (hit_pos - world_xform.translation).normalize();
                let to_camera = (cam_xform.translation - world_xform.translation).normalize();

                let perpendicularity = 1. - Vec3::dot(to_camera, normal).abs();
                let threshold = (perpendicularity * (consts::PI / 5.)).max(2_f32.to_radians());

                let local_pos = basis.inverse().transform_point3(hit_pos);
                let coords = SphericalCoords::from(local_pos);

                let mut proximity_results = [
                    (coords.heading.abs(), Axes::X),
                    (coords.pitch.abs(), Axes::Y),
                    ((90_f32.to_radians() - coords.heading.abs()).abs(), Axes::Z),
                ];
                proximity_results
                    .sort_by(|(prox_a, _), (prox_b, _)| prox_a.partial_cmp(prox_b).unwrap());

                let (prox, axis) = proximity_results[0];
                let hit_axes = if prox < threshold { axis } else { Axes::XYZ };

                cmd.entity(ent).insert(Hovered(Control::Rotation(hit_axes)));
            }
        } else {
            for (ent, _, _) in q_gizmo.iter() {
                cmd.entity(ent).remove::<Hovered>();
            }
        }
    }
}

struct SphericalCoords {
    radius: f32,
    heading: f32,
    pitch: f32,
}

impl Default for SphericalCoords {
    fn default() -> Self {
        Self {
            radius: 0.,
            heading: 0.,
            pitch: 0.,
        }
    }
}

impl From<Vec3> for SphericalCoords {
    fn from(value: Vec3) -> Self {
        let Vec3 { x, y, z } = value;

        let radius = value.length();
        if radius.nearly_eq(0.) {
            return Self::default();
        }

        let pitch = f32::asin(-y / radius);
        let heading = if (pitch - 90_f32.to_radians()).abs() < 1e-3_f32 {
            0.
        } else {
            f32::atan2(x, z)
        };

        Self {
            radius,
            heading,
            pitch,
        }
    }
}

impl fmt::Display for SphericalCoords {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let precision = f.precision().unwrap_or(3);
        write!(
            f,
            "(r {:.3$}, h {:.3$}°, p {:.3$}°)",
            self.radius,
            self.heading.to_degrees(),
            self.pitch.to_degrees(),
            precision,
        )
    }
}
