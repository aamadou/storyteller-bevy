use bevy::prelude::*;

use self::fps::FpsCounterPlugin;

mod fps;
mod inspector;

pub struct UiLayerPlugin;

impl Plugin for UiLayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FpsCounterPlugin);
        app.add_plugins(inspector::InspectorPlugin);
    }
}
