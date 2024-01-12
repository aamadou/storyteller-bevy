use bevy::prelude::*;
use bevy_mod_picking::prelude::*;

use crate::interaction::TransformSpace;

#[derive(Component, Reflect)]
pub(super) struct TransformGizmo {
    pub(super) target : Entity,
    pub(super) transform_space: TransformSpace,
}

#[derive(Event)]
pub(super) struct RebuildGizmo;

#[derive(Bundle)]
pub(super) struct TransformGizmoBundle {
    name: Name,
    marker: TransformGizmo,
    spatial: SpatialBundle,
}

#[derive(Component, Deref)]
pub(super) struct InitialTransform(pub GlobalTransform);

impl TransformGizmoBundle {
    pub fn new(
        target : Entity,
        target_global_transform : &GlobalTransform,
        space: TransformSpace,
        normalizing_scale: f32,
    ) -> Self {
        let name = Name::new("TransformGizmo");
        let marker = TransformGizmo {
            target,
            transform_space: space
        };

        match space {
            TransformSpace::Local => {
                
                let mut transform = target_global_transform.compute_transform();
                transform.scale = Vec3::splat(normalizing_scale);

                Self {
                    name,
                    marker,
                    spatial: SpatialBundle {
                        global_transform : GlobalTransform::from(transform),
                        transform,
                        ..default()
                    },
                }
            }
            TransformSpace::World => {
                
                let transform = Transform::from_translation(target_global_transform.translation())
                    .with_scale(Vec3::splat(normalizing_scale));

                Self {
                    name,
                    marker,
                    spatial: SpatialBundle {
                        global_transform : GlobalTransform::from(transform),
                        transform,
                        ..default()
                    },
                }
            }
        }
    }
}

pub(super) fn on_gizmo_element_hover(
    ev: Listener<Pointer<Over>>,
    mut ra_mats: ResMut<Assets<StandardMaterial>>,
    q_target: Query<&Handle<StandardMaterial>>,
) {
    let mat = ra_mats.get_mut(q_target.get(ev.target).unwrap()).unwrap();
    mat.base_color *= 2.;
}

pub(super) fn on_gizmo_element_unhover(
    ev: Listener<Pointer<Out>>,
    mut ra_mats: ResMut<Assets<StandardMaterial>>,
    q_target: Query<&Handle<StandardMaterial>>,
) {
    let mat = ra_mats.get_mut(q_target.get(ev.target).unwrap()).unwrap();
    mat.base_color *= 0.5;
}
