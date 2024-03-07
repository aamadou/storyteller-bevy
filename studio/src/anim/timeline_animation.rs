use bevy::{
    ecs::{
        entity::{EntityMapper, MapEntities},
        reflect::ReflectMapEntities,
    },
    prelude::*,
};
use space_editor::space_prefab::editor_registry::EditorRegistryExt;

pub struct TimelinePlugin;

impl Plugin for TimelinePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<TimedAnimationClip>()
            .editor_registry::<TimedAnimationClip>()
            .register_type::<AnimationTarget>()
            .editor_registry::<AnimationTarget>();

        app.insert_resource(GlobalSeekTime(0.));
    }
}

#[derive(Resource, Clone, Copy, Debug, Default, Deref, DerefMut, Reflect)]
#[reflect(Resource)]
pub struct GlobalSeekTime(pub f32);

#[derive(Component, Clone, Debug, Reflect, PartialEq)]
#[reflect(Component, MapEntities)]
pub struct TimedAnimationClip {
    clip: Handle<AnimationClip>,
    start_time: f32,
    duration: f32,
    incoming_transition_duration: f32,
    target: Entity,
}

impl PartialOrd for TimedAnimationClip {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.start_time.partial_cmp(&other.start_time) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        match self.duration.partial_cmp(&other.duration) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        match self.clip.partial_cmp(&other.clip) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        match self
            .incoming_transition_duration
            .partial_cmp(&other.incoming_transition_duration)
        {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.target.partial_cmp(&other.target)
    }
}

impl Default for TimedAnimationClip {
    fn default() -> Self {
        Self {
            clip: Default::default(),
            start_time: Default::default(),
            duration: Default::default(),
            incoming_transition_duration: Default::default(),
            target: Entity::PLACEHOLDER,
        }
    }
}

impl MapEntities for TimedAnimationClip {
    fn map_entities(&mut self, entity_mapper: &mut EntityMapper) {
        self.target = entity_mapper.get_or_reserve(self.target);
    }
}

#[derive(Component, Clone, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct AnimationTarget;

#[cfg(feature = "wasm")]
mod wasm {
    use bevy::prelude::*;
    use wasm_bindgen::prelude::*;

    use crate::wasm::{StaticBuffer, WasmMessageBuffer};

    use super::GlobalSeekTime;

    static SEEK_TIME: WasmMessageBuffer<f32> = WasmMessageBuffer::new();

    #[wasm_bindgen(js_name = seekTime)]
    pub fn seek_time(timestamp: f32) {
        SEEK_TIME.write_message(timestamp);
    }

    pub(super) fn update_global_seek_time(mut r_seek_time: ResMut<GlobalSeekTime>) {
        if let Some(timestamp) = SEEK_TIME.take_message() {
            **r_seek_time = timestamp;
        }
    }
}
