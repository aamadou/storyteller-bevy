mod composition;
mod model;
mod view;

use bevy::prelude::*;

use super::{InteractionMode, Selected, TransformSpace, TransformType};

pub(super) struct StudioGizmosPlugin;

impl Plugin for StudioGizmosPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, (model::update_spawns, view::configure))
            .add_systems(Update, model::update)
            .add_systems(PostUpdate, view::draw);
    }
}
