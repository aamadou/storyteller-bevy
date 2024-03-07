use bevy::{
    ecs::{
        entity::{EntityMapper, MapEntities},
        reflect::ReflectMapEntities,
    },
    prelude::*,
};
use serde::{Deserialize, Serialize};
use space_editor::space_prefab::editor_registry::EditorRegistryExt;

pub struct TimelinePlugin;

impl Plugin for TimelinePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<TimedAnimationClip>()
            .editor_registry::<TimedAnimationClip>()
            .register_type::<AnimationTarget>()
            .editor_registry::<AnimationTarget>()
            .add_event::<TimelineEvent>();

        app.insert_resource(GlobalSeekTime {
            time: 0.,
            playing: false,
        });

        #[cfg(feature = "wasm")]
        app.add_systems(
            Update,
            (
                wasm::update_global_seek_time,
                wasm::notify_seek_time_changes.map(bevy::utils::error),
            ),
        );
    }
}

#[derive(Event, Clone, Copy, Debug)]
pub enum TimelineEvent {
    Play,
    Pause,
    Seek(f32),
}

#[derive(Resource, Clone, Copy, Debug, Default, Reflect, Serialize, Deserialize)]
#[reflect(Resource)]
pub struct GlobalSeekTime {
    pub time: f32,
    pub playing: bool,
}

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
    use bevy::{prelude::*, window::PrimaryWindow};
    use wasm_bindgen::prelude::*;

    use crate::wasm::{StaticBuffer, WasmMessageBuffer};

    use super::{GlobalSeekTime, TimelineEvent};

    static SEEK_TIME: WasmMessageBuffer<TimelineEvent> = WasmMessageBuffer::new();

    #[wasm_bindgen(js_name = seekTime)]
    pub fn seek_time(timestamp: f32) {
        SEEK_TIME.write_message(TimelineEvent::Seek(timestamp));
    }

    #[wasm_bindgen(js_name = play)]
    pub fn play() {
        SEEK_TIME.write_message(TimelineEvent::Play);
    }

    #[wasm_bindgen(js_name = pause)]
    pub fn pause() {
        SEEK_TIME.write_message(TimelineEvent::Pause);
    }

    pub(super) fn update_global_seek_time(mut e_timeline_event: EventWriter<TimelineEvent>) {
        if let Some(timestamp) = SEEK_TIME.take_message() {
            e_timeline_event.send(timestamp);
        }
    }

    #[wasm_bindgen(typescript_custom_section)]
    const SEEK_TIME_CHANGED_EVENT: &str = r#"
    export interface SeekTimeChanged extends CustomEvent {
        type: "seek-time-changed";
        detail: number;
    }
    
    declare global {
        export interface GlobalEventHandlersEventMap {
            "seek-time-changed": CameraKeyframesEvent; 
        }
    }
    "#;

    pub(super) fn notify_seek_time_changes(
        r_global_seek_time: Res<GlobalSeekTime>,
        q_window: Query<&Window, With<PrimaryWindow>>,
    ) -> Result<(), String> {
        let Ok(window) = q_window.get_single() else {
            return Ok(());
        };

        crate::wasm::dispatch_to_js(
            window,
            "seek-time-changed",
            &serde_json::to_value(r_global_seek_time.as_ref()).map_err(|e| format!("{e}"))?,
        )
    }
}
