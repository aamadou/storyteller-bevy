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

        app.add_systems(PreUpdate,
            (process_timeline_events, update_tick).chain(),
        );

        #[cfg(feature = "wasm")]
        {
            app.add_systems(First, 
                wasm::update_global_seek_time);
            app.add_systems(
                Update,
                (
                    wasm::notify_seek_time_changes.map(bevy::utils::error),
                    wasm::notify_updated_animation_targets.map(bevy::utils::error)
                ),
            );
        }
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
    speed: f32,
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
            speed: 1.0,
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

fn process_timeline_events(mut e_timeline_event: EventReader<TimelineEvent>, mut r_global_seek_time: ResMut<GlobalSeekTime>) {
    for event in e_timeline_event.read() {
        match event {
            TimelineEvent::Play => r_global_seek_time.playing = true,
            TimelineEvent::Pause => r_global_seek_time.playing = false,
            TimelineEvent::Seek(time) => r_global_seek_time.time = *time,
        }
    }
}

fn update_tick(mut r_global_seek_time: ResMut<GlobalSeekTime>, time: Res<Time>) {
    if r_global_seek_time.playing {
        r_global_seek_time.time += time.delta_seconds();
    }
}

#[cfg(feature = "wasm")]
mod wasm {
    use bevy::{prelude::*, utils::hashbrown::HashMap, window::PrimaryWindow};
    use serde::{Deserialize, Serialize};
    use wasm_bindgen::prelude::*;

    use crate::{scene::SceneElement, wasm::{StaticBuffer, WasmMessageBuffer}};

    use super::{AnimationTarget, GlobalSeekTime, TimelineEvent};

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
    const TIMELINE_TYPES: &str = r#"
    export interface SeekTimeChanged extends CustomEvent {
        type: "seek-time-changed";
        detail: {
            time: number,
            playing: bool
        };
    }

    export interface UpdateAnimationTargets extends CustomEvent {
        type: "update-animation-targets",
        details: {
            added: Record<string, { name?: string, target_type: 'camera' | 'transform' | 'light' | 'bone' | 'skeleton' | 'mesh' >,
            removed: string[]
        }
    }
    
    declare global {
        export interface GlobalEventHandlersEventMap {
            "seek-time-changed": CameraKeyframesEvent; 
            "update-animation-targets": UpdateAnimationTargets;
        }
    }
    "#;

    const CAMERA : &str = "camera";
    const TRANSFORM : &str = "transform";
    const LIGHT : &str = "light";
    const BONE : &str = "bone";
    const SKELETON : &str = "skeleton";
    const MESH : &str = "mesh";

    #[derive(Serialize, Deserialize)]
    struct AddedAnimationTargetRecordItem {
        name: Option<String>,
        target_type: String
    }

    

    #[derive(Serialize, Deserialize)]
    struct UpdatedAnimationTargetsEvent {
        added: HashMap<Entity, AddedAnimationTargetRecordItem>,
        removed: Vec<Entity>
    }

    pub(super) fn notify_updated_animation_targets(
        q_window: Query<&Window, With<PrimaryWindow>>,
        q_cameras: Query<(Entity, Option<&Name>), (Added<AnimationTarget>, With<Camera>)>,
        q_scene_elements: Query<(Entity, Option<&Name>, &SceneElement), (Added<AnimationTarget>, Without<Camera>)>,
        q_transforms: Query<(Entity, Option<&Name>), (Added<AnimationTarget>, With<Transform>, Without<Camera>, Without<SceneElement>)>,
        mut removed: bevy::prelude::RemovedComponents<AnimationTarget>
    ) -> Result<(), String> {
        let Ok(window) = q_window.get_single() else {
            return Ok(());
        };

        let added = q_transforms.iter().map(|(entity, name)| {
            (entity, AddedAnimationTargetRecordItem { name: name.map(|v| v.to_string()), target_type: TRANSFORM.to_owned()})
        }).chain(q_cameras.iter().map(|(entity, name)| (entity, AddedAnimationTargetRecordItem { name: name.map(|v| v.to_string()), target_type: CAMERA.to_owned()})))
        .chain(q_scene_elements.iter().map(|(entity, name, element)| (entity, AddedAnimationTargetRecordItem { name: name.map(|v| v.to_string()), target_type: match element {
            SceneElement::Generic => TRANSFORM,
            SceneElement::Mesh => MESH,
            SceneElement::Skeleton => SKELETON,
            SceneElement::Bone => BONE,
            SceneElement::Light => LIGHT,
            SceneElement::Camera => CAMERA,
        }.to_string()}))).collect::<HashMap<_,_>>();

        let removed = removed.read().collect::<Vec<_>>();

        let event = UpdatedAnimationTargetsEvent { added, removed };

        crate::wasm::dispatch_to_js(
            window,
            "update-animation-targets",
            &serde_json::to_value(&event).map_err(|e| format!("{e}"))?,
        )
    }

    pub(super) fn notify_seek_time_changes(
        r_global_seek_time: Res<GlobalSeekTime>,
        q_window: Query<&Window, With<PrimaryWindow>>,
    ) -> Result<(), String> {
        if !r_global_seek_time.is_changed() {
            return Ok(());
        }
        let Ok(window) = q_window.get_single() else {
            return Ok(());
        };
        info!("Current Seek Time: {r_global_seek_time:?}");

        crate::wasm::dispatch_to_js(
            window,
            "seek-time-changed",
            &serde_json::to_value(r_global_seek_time.as_ref()).map_err(|e| format!("{e}"))?,
        )
    }
}
