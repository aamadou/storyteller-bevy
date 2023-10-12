mod model;
mod view;

use bevy::prelude::*;

use super::{InteractionMode, InteractionTransform, Selected, TransformSpace};

pub(super) struct StudioGizmosPlugin;

impl Plugin for StudioGizmosPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, view::draw);
    }
}
