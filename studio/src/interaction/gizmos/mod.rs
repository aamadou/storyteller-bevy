mod composition;
mod model;
mod view;

use bevy::prelude::*;

use super::{InteractionMode, Selected, TransformSpace, TransformType};

pub(super) struct StudioGizmosPlugin;

impl Plugin for StudioGizmosPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(StudioGizmoConfig::default())
            .add_systems(PreUpdate, (model::update_spawns, view::configure))
            .add_systems(Update, model::update)
            .add_systems(PostUpdate, view::draw);
    }
}

#[derive(Resource, Clone, Copy)]
pub struct StudioGizmoConfig {
    /// Measured (roughly) as a fraction of the viewport height
    pub base_size: f32,
    /// Measured in device-independent "pixel" units
    pub line_thickness: f32,
}

impl Default for StudioGizmoConfig {
    fn default() -> Self {
        Self {
            base_size: 0.125,
            line_thickness: 4.,
        }
    }
}

impl StudioGizmoConfig {
    fn base_size_world_units(&self, gizmo_origin: Vec3, camera_origin: Vec3) -> f32 {
        let dist = Vec3::distance(gizmo_origin, camera_origin);
        // The magic numbers here are because I first decided that dist/20
        // looked pretty good, then measured it as ~1/8 of the viewport height,
        // and then factored that measurement in as a configurable parameter.
        self.base_size * 8. * (dist / 20.)
    }
}
