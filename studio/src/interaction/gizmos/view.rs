use std::f32::consts;

use bevy::{prelude::*, window::PrimaryWindow};

// use crate::{design_system, math::Lerp};

#[rustfmt::skip]
use super::{
    model::{Arc, Axes, Control, Gizmo, GizmoColor, Ring, Shape},
    // InteractionMode, Selected, TransformSpace, TransformType,
};

// #[derive(Clone, Copy)]
// enum GizmoVariant {
//     Full,
//     Partial,
// }

pub(super) fn configure(
    q_window: Query<&Window, With<PrimaryWindow>>,
    mut config: ResMut<GizmoConfig>,
) {
    let scale_factor = q_window.single().scale_factor() as f32;
    config.line_width = 4. * scale_factor;
    config.line_perspective = false;
    config.depth_bias = -1.;
}

pub(super) fn draw(
    q_gizmo: Query<&Gizmo>,
    q_camera_xform: Query<&Transform, With<Camera>>,
    mut gizmos: Gizmos,
) {
    let cam_xform = q_camera_xform.single();

    // TODO (maybe):
    //      1. Map GizmoElements to a new Vec<(Shape, Color)>
    //      2. Bisect the rotator rings and push the back sides into the vec
    //      3. Sort by camera depth
    //      4. Draw
    //
    //      (Starting to feel like a really gross hack -- wish I had a better
    //      grasp of Bevy's render pipeline so we didn't have to lean so hard on
    //      the Gizmos API)

    for gizmo in q_gizmo.iter() {
        for el in gizmo.0.iter() {
            match el.shape {
                Shape::Ring(ring) => {
                    let GizmoColor::Single(color) = el.color else {
                        unreachable!();
                    };

                    if let Control::Rotation(Axes::X | Axes::Y | Axes::Z) = el.control {
                        let (front, _back) = bisect_rotator_ring(&ring, cam_xform);
                        gizmos.arc(front.center, front.axis, front.start, front.length, color);
                        // TODO
                        // gizmos.arc(
                        //     back.center,
                        //     back.axis,
                        //     back.start,
                        //     back.length,
                        //     color.with_a(0.2),
                        // );
                    } else {
                        gizmos.circle(ring.origin, ring.normal, ring.radius, color);
                    }
                }
                Shape::Line(line) => {
                    // TODO: Camera-direction fade
                    let start = line.origin;
                    let end = line.origin + line.direction * line.length;

                    match el.color {
                        GizmoColor::Single(color) => {
                            gizmos.line(start, end, color);
                        }
                        GizmoColor::Gradient(color_a, color_b) => {
                            gizmos.line_gradient(start, end, color_a, color_b);
                        }
                    }
                }
                Shape::Cone(cone) => {
                    // TODO: Camera-direction fade
                    let GizmoColor::Single(color) = el.color else {
                        unreachable!()
                    };
                    gizmos.cone(cone.base, cone.direction, cone.radius, cone.length, color);
                }
            }
        }
    }
}

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

// pub(super) fn draw_old(
//     q_selection: Query<&GlobalTransform, With<Selected>>,
//     q_camera_xform: Query<&Transform, With<Camera>>,
//     q_window: Query<&Window, With<PrimaryWindow>>,
//     mode: Res<InteractionMode>,
//     mut gizmos: Gizmos,
//     mut config: ResMut<GizmoConfig>,
// ) {
//     if !q_selection.is_empty() {
//         // TODO: Use an app state or something so we only have to set this up once
//         let scale_factor = q_window.single().scale_factor() as f32;
//         config.line_width = 4. * scale_factor;
//         config.line_perspective = false;
//         config.depth_bias = -1.;

//         let camera_xform = q_camera_xform.single();

//         // TODO: We only want to render one gizmo -- this is just for testing
//         for world_xform in q_selection.iter() {
//             let xform = world_xform.compute_transform();
//             // Note: Scaling the gizmo size like this isn't perfect due to
//             // distortion from the perspective projection. It bugged me at
//             // first, but even Blender exhibits this type of gizmo distortion if
//             // you dial the camera's focal length way down, so I'm gonna call it
//             // good enough.
//             let dist = Vec3::distance(camera_xform.translation, xform.translation);
//             let size = dist / 20.;

//             let variant = if mode
//                 .transform
//                 .contains(TransformType::TRANSLATE | TransformType::ROTATE)
//             {
//                 GizmoVariant::Partial
//             } else {
//                 GizmoVariant::Full
//             };

//             if mode.transform.contains(TransformType::TRANSLATE) {
//                 draw_translate_gizmo(&mut gizmos, variant, &xform, camera_xform, mode.space, size);
//             }
//             if mode.transform.contains(TransformType::ROTATE) {
//                 draw_rotate_gizmo(&mut gizmos, variant, &xform, camera_xform, mode.space, size);
//             }
//         }
//     }
// }

// // FIXME: We might need more control over the Z-ordering / depth bias than the
// //        Bevy gizmos give us. As best as I can tell, the elements are drawn in
// //        reverse invocation order (the methods called last are rendered first,
// //        like the commands are being popped from a LIFO stack), but it's not
// //        100% consistent and there seems to be lots of flickering / Z-fighting.
// //        Might not be a big deal, so I'm inclined to roll with it for now and
// //        see how users respond.

// #[inline]
// fn draw_translate_gizmo(
//     gizmos: &mut Gizmos,
//     variant: GizmoVariant,
//     xform: &Transform,
//     camera_xform: &Transform,
//     space: TransformSpace,
//     size: f32,
// ) {
//     let origin = xform.translation;
//     let basis = match space {
//         TransformSpace::Local => xform,
//         TransformSpace::World => &Transform::IDENTITY,
//     };

//     for (direction, mut color) in [
//         (basis.local_x(), design_system::Color::RED),
//         (basis.local_y(), design_system::Color::GREEN),
//         (basis.local_z(), design_system::Color::BLUE),
//     ] {
//         // Fade the arrow out when it's pointed directly at the camera.
//         // (Prevents overlapping/obscuring the center screen-space ring, or just
//         // generally getting in the way when it's not very useful to grab onto.)
//         let alpha = (1. - Vec3::dot(camera_xform.forward(), direction).abs())
//             .remap((0., 0.1), (0., 1.))
//             .clamp(0., 1.);

//         color = color.with_a(alpha);

//         // In "Partial" mode, we only draw the center ring and the arrow tips to
//         // leave room for the rotator rings.
//         if matches!(variant, GizmoVariant::Full) {
//             // Draw a line from just outside the center ring to the tip of the arrow
//             let start = origin + direction * size * 0.25;
//             let end = origin + direction * size * 1.6;
//             gizmos.line(start, end, color);

//             // Draw a line from the midpoint of this arrow to the midpoint of
//             // the next one. This "crossbar" can be used to translate the object
//             // on the plane defined by the corresponding axes.
//             let midpoint_self = origin + ((end - origin) / 2.);
//             let (ortho, mut color_ortho) = get_ortho(basis, direction);

//             // Fade the crossbar when its plane is edge-on with the camera, just
//             // like we did for the arrow pointing toward/away from the camera.
//             let ortho_alpha = (1. - Vec3::dot(camera_xform.forward(), ortho).abs())
//                 .remap((0., 0.1), (0., 1.))
//                 .clamp(0., 1.);

//             color_ortho = color_ortho.with_a(f32::min(ortho_alpha, alpha));

//             let ortho_end = origin + ortho * size * 1.6;
//             let midpoint_ortho = origin + ((ortho_end - origin) / 2.);
//             gizmos.line_gradient(midpoint_self, midpoint_ortho, color, color_ortho);
//         }

//         // Draw the arrow tip
//         let factor = match variant {
//             GizmoVariant::Full => 1.2,
//             GizmoVariant::Partial => 1.4,
//         };
//         let cone_base = origin + direction * size * factor;

//         let radius = size * 0.1;
//         let length = size * 0.4;
//         gizmos.cone(cone_base, direction, radius, length, color);
//     }

//     // Draw the center ring. This can be dragged around to translate the object
//     // in screen-space (i.e. on the camera's XY plane).
//     gizmos.circle(origin, camera_xform.forward(), size * 0.2, Color::WHITE);
// }

// // Dumb helper function for grabbing the normal vector/color for the translation
// // gizmo's next arrow. We use this to determine where and how to render the
// // planar translation "crossbars".
// #[inline]
// fn get_ortho(xform: &Transform, compare: Vec3) -> (Vec3, Color) {
//     let x = xform.local_x();
//     let y = xform.local_y();
//     let z = xform.local_z();

//     let diff_x = (compare - x).length_squared();
//     let diff_y = (compare - y).length_squared();
//     let diff_z = (compare - z).length_squared();

//     if diff_x < diff_y && diff_x < diff_z {
//         (y, design_system::Color::GREEN)
//     } else if diff_y < diff_x && diff_y < diff_z {
//         (z, design_system::Color::BLUE)
//     } else {
//         (x, design_system::Color::RED)
//     }
// }

// #[inline]
// fn draw_rotate_gizmo(
//     gizmos: &mut Gizmos,
//     variant: GizmoVariant,
//     xform: &Transform,
//     camera_xform: &Transform,
//     space: TransformSpace,
//     radius: f32,
// ) {
//     let origin = xform.translation;
//     let basis = match space {
//         TransformSpace::Local => xform,
//         TransformSpace::World => &Transform::IDENTITY,
//     };
//     let camera_fwd = camera_xform.forward();

//     // Draw a ring around the whole gizmo for rotating the object in screen
//     // space (i.e. around the camera's forward vector).
//     gizmos.circle(origin, camera_fwd, radius * 1.2, Color::WHITE);

//     // Draw semicircles for the main basis vector axes, bisected by the camera's
//     // XY plane. The `RotatorRings` argument determines which side of the plane
//     // to draw on.
//     draw_rotator_axis_rings(
//         gizmos,
//         RotatorRings::Near,
//         origin,
//         camera_fwd,
//         basis,
//         radius,
//     );

//     // We skip drawing the far-side of the rings in "Partial" mode to cut down
//     // on visual noise and leave less room for mis-clicks.
//     if matches!(variant, GizmoVariant::Full) {
//         draw_rotator_axis_rings(gizmos, RotatorRings::Far, origin, camera_fwd, basis, radius);
//     }
// }

// enum RotatorRings {
//     Near,
//     Far,
// }

// #[inline]
// fn draw_rotator_axis_rings(
//     gizmos: &mut Gizmos,
//     which: RotatorRings,
//     origin: Vec3,
//     camera_fwd: Vec3,
//     basis: &Transform,
//     radius: f32,
// ) {
//     for (normal, color) in [
//         (basis.local_x(), design_system::Color::RED),
//         (basis.local_y(), design_system::Color::GREEN),
//         (basis.local_z(), design_system::Color::BLUE),
//     ] {
//         // The line of intersection between two planes will be mutually
//         // orthogonal to the normal vectors of those planes, so we can find the
//         // points of intersection between the ring and the camera plane by
//         // taking the cross product of the camera direction and the rotation
//         // axis, and scaling it by the ring's radius. The order of the operands
//         // determines which of the intersection points we get.
//         let (color, perp) = match which {
//             RotatorRings::Near => (color, Vec3::cross(normal, camera_fwd)),
//             RotatorRings::Far => (color.with_a(0.2), Vec3::cross(camera_fwd, normal)),
//         };

//         let start = origin + perp.normalize() * radius;

//         gizmos.arc(origin, normal, start, consts::PI, color);
//     }
// }

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
