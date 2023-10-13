use std::mem;

use bevy::{prelude::Color, utils::default};

use crate::{design_system, interaction::InteractionMode};

use super::{
    model::{Axes, Control, Gizmo, GizmoColor, GizmoElement, Shape},
    TransformType,
};

impl Gizmo {
    pub(super) fn update_composition(&mut self, mode: &InteractionMode) {
        let indices = Indices::collect_from(self);
        self.remove_extras(mode.transform, &indices);
        self.add_missing(mode.transform, &indices);
    }

    fn remove_extras(&mut self, ttype: TransformType, indices: &Indices) {
        let mut to_remove =
            Vec::<i32>::with_capacity(mem::size_of::<Indices>() / mem::size_of::<i32>());

        match ttype {
            TransformType::TRANSLATE_ROTATE => {
                to_remove.extend_from_slice(&[
                    indices.xlate_x,
                    indices.xlate_y,
                    indices.xlate_z,
                    indices.xlate_xy,
                    indices.xlate_xz,
                    indices.xlate_yz,
                ]);
            }
            TransformType::TRANSLATE => {
                to_remove.extend_from_slice(&[
                    indices.rotate_screen,
                    indices.rotate_x,
                    indices.rotate_y,
                    indices.rotate_z,
                ]);
            }
            TransformType::ROTATE => {
                to_remove.extend_from_slice(&[
                    indices.xlate_screen,
                    indices.xlate_x,
                    indices.xlate_x_cone,
                    indices.xlate_y,
                    indices.xlate_y_cone,
                    indices.xlate_z,
                    indices.xlate_z_cone,
                    indices.xlate_xy,
                    indices.xlate_xz,
                    indices.xlate_yz,
                ]);
            }
            _ => {
                self.0.clear();
                return;
            }
        }

        to_remove.sort();

        for idx in to_remove.into_iter().rev().filter_map(|idx| {
            if idx == -1 {
                None
            } else {
                Some(idx as usize)
            }
        }) {
            self.0.swap_remove(idx);
        }
    }

    fn add_missing(&mut self, ttype: TransformType, indices: &Indices) {
        if ttype.contains(TransformType::ROTATE) {
            if indices.rotate_screen == -1 {
                self.0.push(GizmoElement {
                    shape: Shape::Ring(default()),
                    color: Color::WHITE.into(),
                    control: Control::Rotation(Axes::SCREEN_Z),
                });
            }
            if indices.rotate_x == -1 {
                self.0.push(GizmoElement {
                    shape: Shape::Ring(default()),
                    color: design_system::Color::RED.into(),
                    control: Control::Rotation(Axes::X),
                });
            }
            if indices.rotate_y == -1 {
                self.0.push(GizmoElement {
                    shape: Shape::Ring(default()),
                    color: design_system::Color::GREEN.into(),
                    control: Control::Rotation(Axes::Y),
                });
            }
            if indices.rotate_z == -1 {
                self.0.push(GizmoElement {
                    shape: Shape::Ring(default()),
                    color: design_system::Color::BLUE.into(),
                    control: Control::Rotation(Axes::Z),
                });
            }
        }

        if ttype.contains(TransformType::TRANSLATE) {
            if indices.xlate_screen == -1 {
                self.0.push(GizmoElement {
                    shape: Shape::Ring(default()),
                    color: Color::WHITE.into(),
                    control: Control::Translation(Axes::SCREEN_XY),
                });
            }
            if indices.xlate_x_cone == -1 {
                self.0.push(GizmoElement {
                    shape: Shape::Cone(default()),
                    color: design_system::Color::RED.into(),
                    control: Control::Translation(Axes::X),
                });
            }
            if indices.xlate_y_cone == -1 {
                self.0.push(GizmoElement {
                    shape: Shape::Cone(default()),
                    color: design_system::Color::GREEN.into(),
                    control: Control::Translation(Axes::Y),
                });
            }
            if indices.xlate_z_cone == -1 {
                self.0.push(GizmoElement {
                    shape: Shape::Cone(default()),
                    color: design_system::Color::BLUE.into(),
                    control: Control::Translation(Axes::Z),
                });
            }

            if !ttype.contains(TransformType::ROTATE) {
                if indices.xlate_x == -1 {
                    self.0.push(GizmoElement {
                        shape: Shape::Line(default()),
                        color: design_system::Color::RED.into(),
                        control: Control::Translation(Axes::X),
                    });
                }
                if indices.xlate_y == -1 {
                    self.0.push(GizmoElement {
                        shape: Shape::Line(default()),
                        color: design_system::Color::GREEN.into(),
                        control: Control::Translation(Axes::Y),
                    });
                }
                if indices.xlate_z == -1 {
                    self.0.push(GizmoElement {
                        shape: Shape::Line(default()),
                        color: design_system::Color::BLUE.into(),
                        control: Control::Translation(Axes::Z),
                    });
                }
                if indices.xlate_xy == -1 {
                    self.0.push(GizmoElement {
                        shape: Shape::Line(default()),
                        color: GizmoColor::Gradient(
                            design_system::Color::RED,
                            design_system::Color::GREEN,
                        ),
                        control: Control::Translation(Axes::XY),
                    });
                }
                if indices.xlate_xz == -1 {
                    self.0.push(GizmoElement {
                        shape: Shape::Line(default()),
                        color: GizmoColor::Gradient(
                            design_system::Color::RED,
                            design_system::Color::BLUE,
                        ),
                        control: Control::Translation(Axes::XZ),
                    });
                }
                if indices.xlate_yz == -1 {
                    self.0.push(GizmoElement {
                        shape: Shape::Line(default()),
                        color: GizmoColor::Gradient(
                            design_system::Color::GREEN,
                            design_system::Color::BLUE,
                        ),
                        control: Control::Translation(Axes::YZ),
                    });
                }
            }
        }
    }
}

struct Indices {
    xlate_screen: i32,
    xlate_x: i32,
    xlate_x_cone: i32,
    xlate_y: i32,
    xlate_y_cone: i32,
    xlate_z: i32,
    xlate_z_cone: i32,
    xlate_xy: i32,
    xlate_xz: i32,
    xlate_yz: i32,
    rotate_screen: i32,
    rotate_x: i32,
    rotate_y: i32,
    rotate_z: i32,
}

impl Default for Indices {
    fn default() -> Self {
        Self {
            xlate_screen: -1,
            xlate_x: -1,
            xlate_x_cone: -1,
            xlate_y: -1,
            xlate_y_cone: -1,
            xlate_z: -1,
            xlate_z_cone: -1,
            xlate_xy: -1,
            xlate_xz: -1,
            xlate_yz: -1,
            rotate_screen: -1,
            rotate_x: -1,
            rotate_y: -1,
            rotate_z: -1,
        }
    }
}

impl Indices {
    fn collect_from(gizmo: &Gizmo) -> Self {
        let mut result = Indices::default();

        for (idx, GizmoElement { control, shape, .. }) in gizmo.0.iter().enumerate() {
            match control {
                Control::Translation(axes) => match *axes {
                    Axes::SCREEN_XY => result.xlate_screen = idx as i32,
                    Axes::XY => result.xlate_xy = idx as i32,
                    Axes::XZ => result.xlate_xz = idx as i32,
                    Axes::YZ => result.xlate_yz = idx as i32,
                    axis => match shape {
                        Shape::Line { .. } => match axis {
                            Axes::X => result.xlate_x = idx as i32,
                            Axes::Y => result.xlate_y = idx as i32,
                            Axes::Z => result.xlate_z = idx as i32,
                            _ => unreachable!(),
                        },
                        Shape::Cone { .. } => match axis {
                            Axes::X => result.xlate_x_cone = idx as i32,
                            Axes::Y => result.xlate_y_cone = idx as i32,
                            Axes::Z => result.xlate_z_cone = idx as i32,
                            _ => unreachable!(),
                        },
                        _ => unreachable!(),
                    },
                },
                Control::Rotation(axes) => match *axes {
                    Axes::SCREEN_Z => result.rotate_screen = idx as i32,
                    Axes::X => result.rotate_x = idx as i32,
                    Axes::Y => result.rotate_y = idx as i32,
                    Axes::Z => result.rotate_z = idx as i32,
                    _ => unreachable!(),
                },
            }
        }

        result
    }
}
