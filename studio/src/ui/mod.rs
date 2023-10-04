use bevy::prelude::*;

use self::fps::FpsCounterPlugin;

mod fps;

pub struct UiLayerPlugin;

impl Plugin for UiLayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FpsCounterPlugin);
    }
}
