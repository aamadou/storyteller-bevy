use bevy::prelude::*;
use bitflags::bitflags;

use super::{InteractionMode, Selected, StudioGizmoConfig, TransformSpace, TransformType};

pub(super) fn update_spawns(
    mut cmd: Commands,
    q_needs_spawn: Query<Entity, (With<Selected>, Without<Gizmo>)>,
    q_needs_despawn: Query<Entity, (Without<Selected>, With<Gizmo>)>,
) {
    // TODO: Is there a way to take a component from one entity and move it onto
    //       another entity without recreating the component from scratch? Would
    //       save us some unnecessary reallocations since Gizmo holds a vector
    for ent in q_needs_despawn.iter() {
        cmd.entity(ent).remove::<Gizmo>();
    }

    for ent in q_needs_spawn.iter() {
        cmd.entity(ent).insert(Gizmo::default());
    }
}

pub(super) fn update(
    mut q_gizmo: Query<(&mut Gizmo, &GlobalTransform)>,
    q_camera_xform: Query<&Transform, With<Camera>>,
    mode: Res<InteractionMode>,
    config: Res<StudioGizmoConfig>,
) {
    let Ok((mut gizmo, world_xform)) = q_gizmo.get_single_mut() else {
        return;
    };
    let Ok(cam_xform) = q_camera_xform.get_single() else {
        return;
    };

    gizmo.update_composition(&mode);

    let xform = world_xform.compute_transform();
    let basis = match mode.space {
        TransformSpace::Local => xform,
        TransformSpace::World => Transform::IDENTITY,
    };

    for el in gizmo.0.iter_mut() {
        el.update(mode.transform, &xform, &basis, cam_xform, &config);
    }
}

#[derive(Component, Clone, Default, Debug)]
pub(super) struct Gizmo(pub Vec<GizmoElement>);

#[derive(Clone, Copy, Debug)]
pub(super) struct GizmoElement {
    pub shape: Shape,
    pub color: GizmoColor,
    pub control: Control,
}

impl GizmoElement {
    #[inline]
    fn update(
        &mut self,
        ttype: TransformType,
        xform: &Transform,
        basis: &Transform,
        cam_xform: &Transform,
        config: &StudioGizmoConfig,
    ) {
        let base_size = config.base_size_world_units(xform.translation, cam_xform.translation);

        match self.control {
            Control::Translation(axes) => match axes {
                Axes::SCREEN_XY => {
                    self.update_xlation_ring(xform, cam_xform, base_size);
                }
                Axes::X | Axes::Y | Axes::Z => match self.shape {
                    Shape::Line(_) => {
                        self.update_xlation_line(axes, xform, basis, base_size);
                    }
                    Shape::Cone(_) => {
                        self.update_xlation_cone(ttype, axes, xform, basis, base_size);
                    }
                    _ => unreachable!(),
                },
                Axes::XY | Axes::XZ | Axes::YZ => {
                    self.update_xlation_crossbar(axes, xform, basis, base_size);
                }
                _ => unreachable!(),
            },
            Control::Rotation(axes) => match axes {
                Axes::SCREEN_Z => {
                    self.update_rotation_ring_screenspace(xform, cam_xform, base_size);
                }
                Axes::X | Axes::Y | Axes::Z => {
                    self.update_rotation_ring(axes, xform, basis, base_size);
                }
                _ => unreachable!(),
            },
        }
    }

    #[inline]
    fn update_xlation_ring(&mut self, xform: &Transform, cam_xform: &Transform, base_size: f32) {
        let Shape::Ring(ring) = &mut self.shape else {
            panic!("Wrong shape for Screen XY Translation: {:?}", self.shape)
        };

        ring.origin = xform.translation;
        ring.normal = cam_xform.forward();
        ring.radius = base_size * 0.2;
    }

    #[inline]
    fn update_xlation_line(
        &mut self,
        axis: Axes,
        xform: &Transform,
        basis: &Transform,
        base_size: f32,
    ) {
        let Shape::Line(line) = &mut self.shape else {
            unreachable!()
        };

        let direction = match axis {
            Axes::X => basis.local_x(),
            Axes::Y => basis.local_y(),
            Axes::Z => basis.local_z(),
            _ => unreachable!(),
        };

        line.origin = xform.translation + direction * base_size * 0.25;
        line.direction = direction;
        line.length = base_size * 1.35;
    }

    #[inline]
    fn update_xlation_cone(
        &mut self,
        ttype: TransformType,
        axis: Axes,
        xform: &Transform,
        basis: &Transform,
        base_size: f32,
    ) {
        let Shape::Cone(cone) = &mut self.shape else {
            unreachable!()
        };

        let offset_factor = if ttype.contains(TransformType::ROTATE) {
            1.4
        } else {
            1.2
        };

        let direction = match axis {
            Axes::X => basis.local_x(),
            Axes::Y => basis.local_y(),
            Axes::Z => basis.local_z(),
            _ => unreachable!(),
        };

        cone.base = xform.translation + direction * base_size * offset_factor;
        cone.direction = direction;
        cone.radius = base_size * 0.1;
        cone.length = base_size * 0.4;
    }

    #[inline]
    fn update_xlation_crossbar(
        &mut self,
        plane: Axes,
        xform: &Transform,
        basis: &Transform,
        base_size: f32,
    ) {
        let Shape::Line(line) = &mut self.shape else {
            unreachable!()
        };

        let (a, b) = match plane {
            Axes::XY => (basis.local_x(), basis.local_y()),
            Axes::XZ => (basis.local_x(), basis.local_z()),
            Axes::YZ => (basis.local_y(), basis.local_z()),
            _ => unreachable!(),
        };

        let start = xform.translation + a * base_size * 0.925;
        let end = xform.translation + b * base_size * 0.925;
        *line = Line::from((start, end));
    }

    #[inline]
    fn update_rotation_ring_screenspace(
        &mut self,
        xform: &Transform,
        cam_xform: &Transform,
        base_size: f32,
    ) {
        let Shape::Ring(ring) = &mut self.shape else {
            unreachable!()
        };

        ring.origin = xform.translation;
        ring.normal = cam_xform.forward();
        ring.radius = base_size * 1.2;
    }

    #[inline]
    fn update_rotation_ring(
        &mut self,
        axis: Axes,
        xform: &Transform,
        basis: &Transform,
        base_size: f32,
    ) {
        let Shape::Ring(ring) = &mut self.shape else {
            unreachable!()
        };

        let normal = match axis {
            Axes::X => basis.local_x(),
            Axes::Y => basis.local_y(),
            Axes::Z => basis.local_z(),
            _ => unreachable!(),
        };

        ring.origin = xform.translation;
        ring.normal = normal;
        ring.radius = base_size;
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) enum GizmoColor {
    Single(Color),
    Gradient(Color, Color),
}

impl From<Color> for GizmoColor {
    fn from(value: Color) -> Self {
        Self::Single(value)
    }
}

impl From<(Color, Color)> for GizmoColor {
    fn from(value: (Color, Color)) -> Self {
        Self::Gradient(value.0, value.1)
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) enum Shape {
    Ring(Ring),
    Line(Line),
    Cone(Cone),
}

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct Ring {
    pub origin: Vec3,
    pub normal: Vec3,
    pub radius: f32,
}

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct Arc {
    pub center: Vec3,
    pub axis: Vec3,
    pub start: Vec3,
    pub length: f32,
}

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct Line {
    pub origin: Vec3,
    pub direction: Vec3,
    pub length: f32,
}

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct Cone {
    pub base: Vec3,
    pub direction: Vec3,
    pub radius: f32,
    pub length: f32,
}

#[derive(Clone, Copy, Debug)]
pub(super) enum Control {
    Translation(Axes),
    Rotation(Axes),
}

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub (super) struct Axes: u8 {
        const X         = 0b0001;
        const Y         = 0b0010;
        const Z         = 0b0100;
        const SCREEN    = 0b1000;
        const XY        = 0b0011;
        const XZ        = 0b0101;
        const YZ        = 0b0110;
        const XYZ       = 0b0111;
        const SCREEN_XY = 0b1011;
        const SCREEN_Z  = 0b1100;
    }
}

impl From<(Vec3, Vec3)> for Line {
    #[inline]
    fn from(value: (Vec3, Vec3)) -> Self {
        let (start, end) = value;
        let vector = end - start;
        let length = vector.length();
        let direction = vector / length;

        Self {
            origin: start,
            direction,
            length,
        }
    }
}

impl From<Line> for (Vec3, Vec3) {
    #[inline]
    fn from(value: Line) -> Self {
        let start = value.origin;
        let end = start + value.direction * value.length;
        (start, end)
    }
}
