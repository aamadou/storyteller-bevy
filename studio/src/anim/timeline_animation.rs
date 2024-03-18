use std::time::Duration;

use bevy::{
    ecs::{
        entity::{EntityMapper, MapEntities},
        reflect::ReflectMapEntities,
    },
    prelude::*,
    utils::HashSet,
};

use bevy_tweening::{
    lens::{TransformPositionLens, TransformRotationLens, TransformScaleLens},
    Animator, EaseMethod, Sequence, Targetable, Tracks, Tween, TweenCompleted,
};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use space_editor::space_prefab::editor_registry::EditorRegistryExt;

use crate::interaction::{MarkedForDeletion, Selectable};

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
            delta: None,
        })
        .add_event::<TweenCompleted>();

        app.add_systems(PreUpdate, (process_timeline_events, update_tick).chain());

        app.add_systems(
            Update,
            (
                process_keyframe_events,
                timeline_bound_component_animator_system::<Transform>,
            ),
        );
        

        app.add_systems(PostUpdate, generate_transform_animation);

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

const MIN_KEYFRAME_DISTANCE: f32 = 0.01;

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
    AddEntityTrack { target: Entity },
}

#[derive(Resource, Clone, Copy, Debug, Default, Reflect, Serialize, Deserialize)]
#[reflect(Resource)]
pub struct GlobalSeekTime {
    pub time: f32,
    pub playing: bool,
    pub delta: Option<f32>,
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
            TimelineInteractionEvent::Pause => {
                r_global_seek_time.playing = false;
            }
            TimelineInteractionEvent::Seek(time) => {
                r_global_seek_time.time = time.max(0.);
            }
        }

        r_global_seek_time.delta = None;
    }
}

fn update_tick(mut r_global_seek_time: ResMut<GlobalSeekTime>, time: Res<Time>) {
    if r_global_seek_time.playing {
        let delta = time.delta_seconds();
        r_global_seek_time.time += delta;
        r_global_seek_time.delta = Some(delta);
    }
}

fn process_keyframe_events(
    mut commands: Commands,
    mut e_timeline_event: EventReader<KeyframeInteractionEvent>,
    r_global_seek_time: Res<GlobalSeekTime>,
    q_animation_targets: Query<(Entity, &Transform), With<AnimationTarget>>,
    q_potential_targets: Query<Entity, (With<Transform>, Without<AnimationTarget>)>,
    mut q_keyframes: Query<(Entity, &mut Keyframe)>,
) {
    for event in e_timeline_event.read() {
        info!("Processing Keyframe Event {event:?}");
        match event {
            KeyframeInteractionEvent::RecordKeyframe { target } => {
                if let Ok((target, transform)) = q_animation_targets.get(*target) {
                    let mut existing = None;
                    for (entity, keyframe) in q_keyframes.iter_mut() {
                        if keyframe.target == target
                            && (keyframe.time - r_global_seek_time.time).abs()
                                < MIN_KEYFRAME_DISTANCE
                        {
                            existing = Some(entity);
                            break;
                        }
                    }
                    if let Some(existing) = existing {
                        commands.entity(existing).insert((
                            *transform,
                            TransformKeyframe::default(),
                            Keyframe {
                                target,
                                time: r_global_seek_time.time,
                            },
                        ));
                    } else {
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
            }
            KeyframeInteractionEvent::MoveKeyframeTime { target, time } => {
                if let Ok((_, mut keyframe)) = q_keyframes.get_mut(*target) {
                    keyframe.time = *time;
                }
            }
            KeyframeInteractionEvent::AddEntityTrack { target } => {
                if let Ok(target) = q_potential_targets.get(*target) {
                    info!("Adding Animation Target to Entity");
                    commands.entity(target).insert(AnimationTarget);
                } else {
                    warn!("Can't add animation target to entity");
                }
            }
        }
    }
}

fn generate_transform_animation(
    mut commands: Commands,
    changed: Query<&Keyframe, (With<TransformKeyframe>, Changed<Keyframe>)>,
    keyframes: Query<(&Keyframe, &Transform, &TransformKeyframe, Entity), Without<MarkedForDeletion>>,
    removed: Query<&Keyframe, With<MarkedForDeletion>>,
) {
    let changed_targets = changed
        .iter()
        .map(|keyframe| keyframe.target)
        .chain(removed.iter().map(|keyframe| {
            info!("Processing a removed keyframe {keyframe:?}");
            keyframe.target
        }))
        .collect::<HashSet<_>>();
    let keyframes = keyframes
        .iter()
        .filter(|(keyframe, _, _, _)| changed_targets.contains(&keyframe.target))
        .group_by(|(keyframe, _, _, _)| keyframe.target);

    for (target, keyframes) in &keyframes {
        let keyframes = keyframes.sorted_by(|a, b| a.0.time.total_cmp(&b.0.time));

        let (_, _, _, translation, rotation, scale) = keyframes.fold(
            (
                Option::<(f32, Vec3)>::None,
                Option::<(f32, Quat)>::None,
                Option::<(f32, Vec3)>::None,
                Vec::<Tween<Transform>>::new(),
                Vec::<Tween<Transform>>::new(),
                Vec::<Tween<Transform>>::new(),
            ),
            |(
                last_translation,
                last_rotation,
                last_scale,
                mut translation_list,
                mut rotation_list,
                mut scale_list,
            ),
             (keyframe, transform, transform_keyframe, _)| {
                let time = keyframe.time;
                let translation = transform.translation;
                let rotation = transform.rotation;
                let scale = transform.scale;

                info!("Processing Keyframe: {time} previous: {last_rotation:?}");

                if transform_keyframe.affect_translation {
                    let (last_time, start) = match last_translation {
                        Some(v) => v,
                        None => (0., translation),
                    };
                    info!("Should apply translation");
                    if (time - last_time).abs() > 0.00001 {
                        translation_list.push(Tween::new(
                            EaseMethod::Linear,
                            Duration::from_secs_f32((time - last_time).max(0.00001)),
                            TransformPositionLens {
                                start,
                                end: translation,
                            },
                        ));
                    }
                }

                if transform_keyframe.affect_rotation {
                    let (last_time, start) = match last_rotation {
                        Some(v) => v,
                        None => (0., rotation),
                    };

                    info!("Should apply rotation");

                    if (time - last_time).abs() > 0.00001 {
                        rotation_list.push(Tween::new(
                            EaseMethod::Linear,
                            Duration::from_secs_f32((time - last_time).max(0.00001)),
                            TransformRotationLens {
                                start,
                                end: rotation,
                            },
                        ));
                    }
                }
                if transform_keyframe.affect_scale {
                    let (last_time, start) = match last_scale {
                        Some(v) => v,
                        None => (0., scale),
                    };
                    info!("Should apply scale");
                    if (time - last_time).abs() > 0.00001 {
                        scale_list.push(Tween::new(
                            EaseMethod::Linear,
                            Duration::from_secs_f32((time - last_time).max(0.00001)),
                            TransformScaleLens { start, end: scale },
                        ));
                    }
                }

                let next_translation = if transform_keyframe.affect_translation {
                    Some((time, translation))
                } else {
                    last_translation
                };
                let next_rotation = if transform_keyframe.affect_rotation {
                    Some((time, rotation))
                } else {
                    last_rotation
                };
                let next_scale = if transform_keyframe.affect_scale {
                    Some((time, scale))
                } else {
                    last_scale
                };

                (
                    next_translation,
                    next_rotation,
                    next_scale,
                    translation_list,
                    rotation_list,
                    scale_list,
                )
            },
        );

        let mut tracks = Vec::with_capacity(3);

        if !translation.is_empty() {
            tracks.push(Sequence::new(translation));
        } else {
            info!("No translation");
        }
        if !rotation.is_empty() {
            tracks.push(Sequence::new(rotation));
        } else {
            info!("No rotation");
        }
        if !scale.is_empty() {
            tracks.push(Sequence::new(scale));
        } else {
            info!("No scale");
        }

        if tracks.is_empty() {
            commands.entity(target).remove::<Animator<Transform>>();
        } else {
            let tracks = Tracks::new(tracks);

            commands.entity(target).insert(Animator::new(tracks));
        }
    }
}

struct ComponentTarget<'a, T: Component>(&'a mut T);

impl<'a, T: Component> ComponentTarget<'a, T> {
    fn new(target: Mut<'a, T>) -> Self {
        Self(target.into_inner())
    }
}

impl<'a, T: Component> Targetable<T> for ComponentTarget<'a, T> {
    fn target_mut(&mut self) -> &mut T {
        self.0
    }
}

pub fn timeline_bound_component_animator_system<T: Component>(
    time: Res<GlobalSeekTime>,
    mut query: Query<(Entity, &mut T, &mut Animator<T>)>,
    events: ResMut<Events<TweenCompleted>>,
) {
    if time.is_changed() && time.time >= 0.0 {
        info!("Animating with time: {time:?}");
        let mut events: Mut<Events<TweenCompleted>> = events.into();
        for (entity, target, mut animator) in query.iter_mut() {
            if animator.tweenable().duration() <= Duration::from_secs_f32(0.0001) {
                continue;
            }
            let mut target = ComponentTarget::new(target);
            if let Some(delta) = time.delta {
                info!("Playing animation with delta {delta}");
                animator.tweenable_mut().tick(
                    Duration::from_secs_f32(delta),
                    &mut target,
                    entity,
                    &mut events,
                );
            } else {
                info!("setting tween time to {}", time.time);
                let tween = animator.tweenable_mut();
                tween.set_elapsed(Duration::from_secs_f32(time.time.max(0.)));
                info!("Tween time set successfully");
                tween.tick(Duration::ZERO, &mut target, entity, &mut events);
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
        interaction::MarkedForDeletion, scene::SceneElement, wasm::{FromStr, StaticBuffer, WasmMessageBuffer}
    };

    use super::{
        AnimationTarget, EaseType, GlobalSeekTime, Keyframe, KeyframeInteractionEvent, TimedAnimationClipAsset, TimelineInteractionEvent, TransformKeyframe
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

    #[wasm_bindgen(js_name = addEntityTrack)]
    pub fn add_entity_track(entity: String) {
        let Ok(target) = Entity::from_str(&entity) else {
            return;
        };
        KEYFRAME_EVENT.write_message(vec![KeyframeInteractionEvent::AddEntityTrack { target }]);
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
        removed: Query<Entity, With<MarkedForDeletion>>,
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

        let removed = removed.iter().map(|v| format!("{v:?}")).collect::<Vec<_>>();

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
