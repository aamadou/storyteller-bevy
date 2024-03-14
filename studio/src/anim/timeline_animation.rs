use bevy::{
    ecs::{
        entity::{EntityMapper, MapEntities},
        reflect::ReflectMapEntities,
    },
    prelude::*,
};

use serde::{Deserialize, Serialize};
use space_editor::space_prefab::editor_registry::EditorRegistryExt;

use crate::interaction::Selectable;

pub struct TimelinePlugin;

impl Plugin for TimelinePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<TimedAnimationClipAsset>()
            .editor_registry::<TimedAnimationClipAsset>()
            .register_type::<AnimationTarget>()
            .editor_registry::<AnimationTarget>()
            .register_type::<TransformKeyframe>()
            .editor_registry::<TransformKeyframe>()
            .register_type::<Keyframe>()
            .editor_registry::<Keyframe>()
            .register_type::<KeyframeOrder>()
            .editor_registry::<KeyframeOrder>()
            .add_event::<TimelineInteractionEvent>()
            .add_event::<KeyframeInteractionEvent>();

        app.insert_resource(GlobalSeekTime {
            time: 0.,
            playing: false,
        });

        app.add_systems(PreUpdate, (process_timeline_events, update_tick).chain());

        app.add_systems(Update, process_keyframe_events);

        #[cfg(feature = "wasm")]
        {
            app.add_systems(
                First,
                (
                    wasm::update_global_seek_time,
                    wasm::update_keyframe_interaction,
                ),
            );
            app.add_systems(
                PostUpdate,
                (
                    wasm::notify_seek_time_changes.map(bevy::utils::error),
                    wasm::notify_keyframe_changes.map(bevy::utils::error),
                    wasm::notify_updated_animation_targets.map(bevy::utils::error),
                ),
            );
        }
    }
}

#[derive(Event, Clone, Copy, Debug)]
pub enum TimelineInteractionEvent {
    Play,
    Pause,
    Seek(f32),
}

#[derive(Event, Clone, Copy, Debug)]
pub enum KeyframeInteractionEvent {
    RecordKeyframe { target: Entity },
    MoveKeyframeTime { target: Entity, time: f32 },
}

#[derive(Resource, Clone, Copy, Debug, Default, Reflect, Serialize, Deserialize)]
#[reflect(Resource)]
pub struct GlobalSeekTime {
    pub time: f32,
    pub playing: bool,
}

#[derive(Clone, Debug, Copy, Default, Reflect, PartialEq, Serialize, Deserialize)]
pub enum EaseType {
    #[default]
    Linear,
    CubicHermite,
}

#[derive(Component, Clone, Debug, Reflect, PartialEq)]
#[reflect(Component)]
pub struct TransformKeyframe {
    incoming_easing: EaseType,
    affect_translation: bool,
    affect_rotation: bool,
    affect_scale: bool,
}

impl Default for TransformKeyframe {
    fn default() -> Self {
        Self {
            incoming_easing: Default::default(),
            affect_translation: true,
            affect_rotation: true,
            affect_scale: true,
        }
    }
}

#[derive(Component, Clone, Debug, Reflect, PartialEq)]
#[reflect(Component, MapEntities)]
pub struct Keyframe {
    target: Entity,
    time: f32,
}

#[derive(Component, Clone, Debug, Default, Reflect, PartialEq, Eq, PartialOrd, Ord)]
#[reflect(Component)]
pub struct KeyframeOrder(u32);

impl PartialOrd for Keyframe {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.time.partial_cmp(&other.time)
    }
}

impl MapEntities for Keyframe {
    fn map_entities(&mut self, entity_mapper: &mut EntityMapper) {
        self.target = entity_mapper.get_or_reserve(self.target);
    }
}

impl Default for Keyframe {
    fn default() -> Self {
        Self {
            target: Entity::PLACEHOLDER,
            time: Default::default(),
        }
    }
}

#[derive(Component, Clone, Debug, Reflect, PartialEq)]
#[reflect(Component)]
pub struct TimedAnimationClipAsset {
    clip: Handle<AnimationClip>,
    duration: f32,
    speed: f32,
}

impl PartialOrd for TimedAnimationClipAsset {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.duration.partial_cmp(&other.duration) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        match self.speed.partial_cmp(&other.speed) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        None
    }
}

impl Default for TimedAnimationClipAsset {
    fn default() -> Self {
        Self {
            clip: Default::default(),
            duration: Default::default(),
            speed: 1.0,
        }
    }
}

#[derive(Component, Clone, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct AnimationTarget;

fn process_timeline_events(
    mut e_timeline_event: EventReader<TimelineInteractionEvent>,
    mut r_global_seek_time: ResMut<GlobalSeekTime>,
) {
    for event in e_timeline_event.read() {
        match event {
            TimelineInteractionEvent::Play => r_global_seek_time.playing = true,
            TimelineInteractionEvent::Pause => r_global_seek_time.playing = false,
            TimelineInteractionEvent::Seek(time) => r_global_seek_time.time = *time,
        }
    }
}

fn update_tick(mut r_global_seek_time: ResMut<GlobalSeekTime>, time: Res<Time>) {
    if r_global_seek_time.playing {
        r_global_seek_time.time += time.delta_seconds();
    }
}

fn process_keyframe_events(
    mut commands: Commands,
    mut e_timeline_event: EventReader<KeyframeInteractionEvent>,
    r_global_seek_time: Res<GlobalSeekTime>,
    q_animation_targets: Query<(Entity, &Transform), With<AnimationTarget>>,
    mut q_keyframes: Query<&mut Keyframe>,
) {
    for event in e_timeline_event.read() {
        info!("Processing Keyframe Event {event:?}");
        match event {
            KeyframeInteractionEvent::RecordKeyframe { target } => {
                if let Ok((target, transform)) = q_animation_targets.get(*target) {
                    commands.spawn((
                        SpatialBundle {
                            transform: *transform,
                            ..Default::default()
                        },
                        Selectable,
                        Keyframe {
                            target,
                            time: r_global_seek_time.time,
                        },
                        TransformKeyframe::default(),
                    ));
                }
            }
            KeyframeInteractionEvent::MoveKeyframeTime { target, time } => {
                if let Ok(mut keyframe) = q_keyframes.get_mut(*target) {
                    keyframe.time = *time;
                }
            }
        }
    }
}

#[cfg(feature = "wasm")]
mod wasm {
    use bevy::{prelude::*, utils::hashbrown::HashMap, window::PrimaryWindow};
    use serde::{Deserialize, Serialize};
    use wasm_bindgen::prelude::*;

    use crate::{
        scene::SceneElement,
        wasm::{FromStr, StaticBuffer, WasmMessageBuffer},
    };

    use super::{
        AnimationTarget, EaseType, GlobalSeekTime, Keyframe, KeyframeInteractionEvent,
        TimedAnimationClipAsset, TimelineInteractionEvent, TransformKeyframe,
    };

    static SEEK_TIME: WasmMessageBuffer<TimelineInteractionEvent> = WasmMessageBuffer::new();
    static KEYFRAME_EVENT: WasmMessageBuffer<Vec<KeyframeInteractionEvent>> =
        WasmMessageBuffer::new();

    #[wasm_bindgen(js_name = seekTime)]
    pub fn seek_time(timestamp: f32) {
        SEEK_TIME.write_message(TimelineInteractionEvent::Seek(timestamp));
    }

    #[wasm_bindgen(js_name = play)]
    pub fn play() {
        SEEK_TIME.write_message(TimelineInteractionEvent::Play);
    }

    #[wasm_bindgen(js_name = pause)]
    pub fn pause() {
        SEEK_TIME.write_message(TimelineInteractionEvent::Pause);
    }

    #[wasm_bindgen(js_name = recordKeyframe)]
    pub fn record_keyframe(entities: Vec<String>) {
        KEYFRAME_EVENT.write_message(
            entities
                .iter()
                .filter_map(|entity| Entity::from_str(entity).ok())
                .map(|target| KeyframeInteractionEvent::RecordKeyframe { target })
                .collect(),
        );
    }

    #[wasm_bindgen(js_name = moveKeyframeTime)]
    pub fn move_keyframe_time(entity: String, time: f32) {
        let Ok(target) = Entity::from_str(&entity) else {
            return;
        };
        KEYFRAME_EVENT.write_message(vec![KeyframeInteractionEvent::MoveKeyframeTime {
            target,
            time,
        }]);
    }

    pub(super) fn update_global_seek_time(
        mut e_timeline_event: EventWriter<TimelineInteractionEvent>,
    ) {
        if let Some(timestamp) = SEEK_TIME.take_message() {
            e_timeline_event.send(timestamp);
        }
    }

    pub(super) fn update_keyframe_interaction(
        mut e_keyframe_event: EventWriter<KeyframeInteractionEvent>,
    ) {
        if let Some(keyframes) = KEYFRAME_EVENT.take_message() {
            for keyframe in keyframes.into_iter() {
                e_keyframe_event.send(keyframe);
            }
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
            added: Record<string, { name?: string, target_type: 'camera' | 'transform' | 'light' | 'bone' | 'skeleton' | 'mesh' } >,
            removed: string[]
        }
    }

    export interface TransformKeyframeInfo {
        target: string,
        time: number,
        entity: string,
        incoming_easing: "linear" | "cubic-hermite",
        affect_translation: bool,
        affect_rotation: bool,
        affect_scale: bool
    }

    export interface TimedAnimationClipInfo {
        target: string,
        time: number,
        entity: string,
        type: 'animation-clip',
        clip: string,
        duration: number,
        speed: number
    }

    export type Keyframe = TransformKeyframeInfo | TimedAnimationClipInfo;

    export interface UpdateKeyframes extends CustomEvent {
        type: "update-keyframes",
        details: {
            added: Record<string, Keyframe)>,
            removed: string[]
        }
    }
    
    declare global {
        export interface GlobalEventHandlersEventMap {
            "seek-time-changed": CameraKeyframesEvent; 
            "update-animation-targets": UpdateAnimationTargets;
            "update-keyframes": UpdateKeyframes;
        }
    }
    "#;

    const CAMERA: &str = "camera";
    const TRANSFORM: &str = "transform";
    const LIGHT: &str = "light";
    const BONE: &str = "bone";
    const SKELETON: &str = "skeleton";
    const MESH: &str = "mesh";

    #[derive(Serialize, Deserialize)]
    struct AddedAnimationTargetRecordItem {
        name: Option<String>,
        target_type: String,
    }

    #[derive(Serialize, Deserialize)]
    struct UpdatedAnimationTargetsEvent {
        added: HashMap<String, AddedAnimationTargetRecordItem>,
        removed: Vec<String>,
    }

    #[derive(Serialize, Deserialize)]
    struct TimedAnimationClipInfo {
        entity: String,
        target: String,
        time: f32,
        clip: String,
        duration: f32,
        speed: f32,
    }

    #[derive(Serialize, Deserialize)]
    struct TransformKeframeInfo {
        entity: String,
        target: String,
        time: f32,
        incoming_easing: EaseType,
        affect_translation: bool,
        affect_rotation: bool,
        affect_scale: bool,
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct UpdateKeyframes {
        added: HashMap<String, serde_json::Value>,
        removed: Vec<String>,
    }

    pub(super) fn notify_updated_animation_targets(
        q_window: Query<&Window, With<PrimaryWindow>>,
        q_cameras: Query<(Entity, Option<&Name>), (Added<AnimationTarget>, With<Camera>)>,
        q_scene_elements: Query<
            (Entity, Option<&Name>, &SceneElement),
            (Added<AnimationTarget>, Without<Camera>),
        >,
        q_transforms: Query<
            (Entity, Option<&Name>),
            (
                Added<AnimationTarget>,
                With<Transform>,
                Without<Camera>,
                Without<SceneElement>,
            ),
        >,
        mut removed: bevy::prelude::RemovedComponents<AnimationTarget>,
    ) -> Result<(), String> {
        let Ok(window) = q_window.get_single() else {
            return Ok(());
        };

        let added = q_transforms
            .iter()
            .map(|(entity, name)| {
                (
                    format!("{entity:?}"),
                    AddedAnimationTargetRecordItem {
                        name: name.map(|v| v.to_string()),
                        target_type: TRANSFORM.to_owned(),
                    },
                )
            })
            .chain(q_cameras.iter().map(|(entity, name)| {
                (
                    format!("{entity:?}"),
                    AddedAnimationTargetRecordItem {
                        name: name.map(|v| v.to_string()),
                        target_type: CAMERA.to_owned(),
                    },
                )
            }))
            .chain(q_scene_elements.iter().map(|(entity, name, element)| {
                (
                    format!("{entity:?}"),
                    AddedAnimationTargetRecordItem {
                        name: name.map(|v| v.to_string()),
                        target_type: match element {
                            SceneElement::Generic => TRANSFORM,
                            SceneElement::Mesh => MESH,
                            SceneElement::Skeleton => SKELETON,
                            SceneElement::Bone => BONE,
                            SceneElement::Light => LIGHT,
                            SceneElement::Camera => CAMERA,
                        }
                        .to_string(),
                    },
                )
            }))
            .collect::<HashMap<_, _>>();

        let removed = removed.read().map(|v| format!("{v:?}")).collect::<Vec<_>>();

        if added.is_empty() && removed.is_empty() {
            return Ok(());
        }

        let event = UpdatedAnimationTargetsEvent { added, removed };

        crate::wasm::dispatch_to_js(
            window,
            "update-animation-targets",
            &serde_json::to_value(&event).map_err(|e| format!("{e}"))?,
        )
    }

    pub(super) fn notify_keyframe_changes(
        q_window: Query<&Window, With<PrimaryWindow>>,
        q_transform_keyframes: Query<
            (Entity, &Keyframe, &TransformKeyframe),
            Or<(Changed<Keyframe>, Changed<TransformKeyframe>)>,
        >,
        q_clips: Query<
            (Entity, &Keyframe, &TimedAnimationClipAsset),
            Or<(Changed<Keyframe>, Changed<TimedAnimationClipAsset>)>,
        >,
        mut removed: bevy::prelude::RemovedComponents<Keyframe>,
    ) -> Result<(), String> {
        let Ok(window) = q_window.get_single() else {
            return Ok(());
        };

        let added = q_transform_keyframes
            .iter()
            .map(|(entity, keyframe, transform_keyframe)| {
                match serde_json::to_value(TransformKeframeInfo {
                    entity: format!("{entity:?}"),
                    target: format!("{:?}", keyframe.target),
                    time: keyframe.time,
                    incoming_easing: transform_keyframe.incoming_easing,
                    affect_translation: transform_keyframe.affect_translation,
                    affect_rotation: transform_keyframe.affect_rotation,
                    affect_scale: transform_keyframe.affect_scale,
                })
                .map_err(|e| format!("{e}"))
                {
                    Ok(value) => Ok((format!("{entity:?}"), value)),
                    Err(e) => Err(e),
                }
            })
            .chain(q_clips.iter().map(|(entity, keyframe, clip)| {
                match serde_json::to_value(TimedAnimationClipInfo {
                    entity: format!("{entity:?}"),
                    target: format!("{:?}", keyframe.target),
                    time: keyframe.time,
                    clip: clip.clip.id().to_string(),
                    duration: clip.duration,
                    speed: clip.speed,
                })
                .map_err(|e| format!("{e}"))
                {
                    Ok(value) => Ok((format!("{entity:?}"), value)),
                    Err(e) => Err(e),
                }
            }))
            .collect::<Result<HashMap<_, _>, String>>()?;

        let removed = removed.read().map(|v| format!("{v:?}")).collect::<Vec<_>>();

        if added.is_empty() && removed.is_empty() {
            return Ok(());
        }

        let event = UpdateKeyframes { added, removed };

        info!("Updating keyframes: {event:?}");

        crate::wasm::dispatch_to_js(
            window,
            "update-keyframes",
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
