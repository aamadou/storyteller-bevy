use bevy::app::{App, Plugin};
use space_editor::prelude::EditorRegistryExt;
use crate::material::short_material::ShortMaterial;


pub mod short_material;

pub struct MaterialPlugin;
impl Plugin for MaterialPlugin {
  fn build(&self, app: &mut App) {
    app.register_type::<ShortMaterial>()
        .editor_registry::<ShortMaterial>();
  }
}
