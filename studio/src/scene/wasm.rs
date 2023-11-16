use std::collections::VecDeque;

use bevy::{prelude::*, utils::hashbrown::HashSet, window::PrimaryWindow};
use serde::Serialize;
use wasm_bindgen::prelude::*;

use crate::{
    wasm::{self, AsJson, WasmMessageBuffer},
    MainCamera,
};

use super::Inspectable;

static SCENE_REQUEST: WasmMessageBuffer<()> = WasmMessageBuffer::new();

#[wasm_bindgen(js_name = saveScene)]
pub fn save_scene() {
    wasm::write_message!(SCENE_REQUEST, ());
}

pub(super) fn scene_download_requested() -> bool {
    wasm::read_buffer!(SCENE_REQUEST).is_some()
}

pub(super) fn save_scene_system(world: &mut World) -> Result<(), String> {
    // Extract all entities with the `Inspectable` marker...
    let mut scene_entities = world
        .query_filtered::<Entity, With<Inspectable>>()
        .iter(world)
        .collect::<HashSet<_>>();

    // ...and their parents...
    scene_entities.extend(
        world
            .query_filtered::<&Parent, With<Inspectable>>()
            .iter(world)
            .map(|parent| **parent),
    );

    // ...and their children.
    scene_entities.extend(
        world
            // TODO: This is a bit of a hack to exclude the GizmoCamera entity,
            //       which is a child of the MainCamera.
            .query_filtered::<&Children, (With<Inspectable>, Without<MainCamera>)>()
            .iter(world)
            .flat_map(|children| children.iter().copied()),
    );

    // Build and serialize the scene
    let mut builder = DynamicSceneBuilder::from_world(world);
    builder
        .allow_all()
        .allow_all_resources()
        .deny::<ComputedVisibility>()
        .deny_resource::<Time>()
        .extract_entities(scene_entities.into_iter())
        .remove_empty_entities()
        .extract_resources();

    let scene = builder.build();
    let type_registry = world.resource::<AppTypeRegistry>();
    let serialized = scene
        .serialize_ron(type_registry)
        .map_err(|err| format!("{err}"))?;

    let window = world
        .query_filtered::<&Window, With<PrimaryWindow>>()
        .get_single(world)
        .map_err(|err| format!("{err}"))?;

    wasm::dispatch_to_js(window, "scene-download-ready", &serialized)
}

#[wasm_bindgen(typescript_custom_section)]
const TYPE_DECLARATIONS: &str = r#"
export interface SceneDownloadReadyEvent extends CustomEvent {
    type: "scene-download-ready";
    detail: string;
}

export interface SceneObject {
    /** The entity ID. */
    id: string;
    /** A user-friendly name for displaying in the UI */
    name: string;
    /** The IDs of this object's children in the hierarchy */
    children: string[];
    /** The ID of this object's parent in the hierarchy, if it has one */
    parent: string | null;
    /** The local-space transform */
    transform: Transform;
    /** The world-space transform */
    globalTransform: GlobalTransform;
}

/**
 * An event indicating that a new entity has spawned.
 */
export interface EntitySpawnEvent extends CustomEvent {
    type: "entity-spawn";
    detail: SceneObject;
}

/**
 * An event indicating that an entity has been de-spawned.
 */
export interface EntityDespawnEvent extends CustomEvent {
    type: "entity-despawn";
    /** The entity ID */
    detail: string;
}

/**
 * An event indicating that multiple entities have spawned.
 */
export interface EntityMultiSpawnEvent extends CustomEvent {
    type: "entity-multi-spawn";
    detail: SceneObject[];
}

declare global {
    export interface HTMLElementEventMap {
        "scene-download-ready": SceneDownloadReadyEvent;
        "entity-spawn": EntitySpawnEvent;
        "entity-despawn": EntityDespawnEvent;
        "entity-multi-spawn": EntityMultiSpawnEvent;
    }
}
    "#;

#[derive(Serialize)]
pub(super) struct SceneObject {
    id: String,
    name: String,
    children: Vec<String>,
    parent: Option<String>,
    transform: serde_json::Value,
    #[serde(rename(serialize = "globalTransform"))]
    global_transform: serde_json::Value,
}

struct SceneObjectBuilder<'a> {
    id: Entity,
    name: Option<String>,
    children: Option<Vec<Entity>>,
    parent: Option<Entity>,
    transform: Option<&'a Transform>,
    global_transform: Option<&'a GlobalTransform>,
}

impl<'a> SceneObjectBuilder<'a> {
    fn new(id: Entity) -> Self {
        Self {
            id,
            name: None,
            children: None,
            parent: None,
            transform: None,
            global_transform: None,
        }
    }
    fn finish(self) -> SceneObject {
        SceneObject {
            id: format!("{:?}", self.id),
            name: self.name.unwrap(),
            children: self
                .children
                .map(|children| children.iter().map(|ent| format!("{ent:?}")).collect())
                .unwrap_or_default(),
            parent: self.parent.map(|ent| format!("{ent:?}")),
            transform: self.transform.unwrap().as_json(),
            global_transform: self.global_transform.unwrap().as_json(),
        }
    }
    fn with_name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }
    fn with_children(mut self, children: Vec<Entity>) -> Self {
        self.children = Some(children);
        self
    }
    fn with_parent(mut self, parent: Entity) -> Self {
        self.parent = Some(parent);
        self
    }
    fn with_transform(mut self, transform: &'a Transform) -> Self {
        self.transform = Some(transform);
        self
    }
    fn with_global_transform(mut self, global_transform: &'a GlobalTransform) -> Self {
        self.global_transform = Some(global_transform);
        self
    }
}

impl<'a> SceneObject {
    fn build(id: Entity) -> SceneObjectBuilder<'a> {
        SceneObjectBuilder::new(id)
    }
}

pub(super) fn notify_entity_spawn_changes(
    mut l_notified: Local<HashSet<Entity>>,
    // Note: We use locals for these data structures purely to reuse
    // existing allocations across multiple invocations of this system
    mut l_new_objects: Local<Vec<SceneObject>>,
    mut l_despawned_entities: Local<Vec<Entity>>,
    mut l_missing_parents: Local<VecDeque<Entity>>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    q_inspectables: Query<(Entity, &Transform, &GlobalTransform), With<Inspectable>>,
    q_others: Query<(Entity, &Transform, &GlobalTransform), Without<Inspectable>>,
    q_names: Query<&Name>,
    q_parents: Query<&Children>,
    q_children: Query<&Parent>,
) -> Result<(), String> {
    l_missing_parents.clear();

    l_new_objects.clear();
    l_new_objects.extend(
        q_inspectables
            .iter()
            .filter_map(|(ent, xform, world_xform)| {
                build_scene_object(
                    &mut l_notified,
                    &mut l_missing_parents,
                    ent,
                    xform,
                    world_xform,
                    &q_inspectables,
                    &q_names,
                    &q_parents,
                    &q_children,
                )
            }),
    );

    while let Some(parent) = l_missing_parents.pop_front() {
        let Ok((ent, xform, world_xform)) = q_others.get(parent) else {
            error!("Failed to find missing parent: {parent:?}");
            l_notified.insert(parent);

            continue;
        };

        if let Some(object) = build_scene_object(
            &mut l_notified,
            &mut l_missing_parents,
            ent,
            xform,
            world_xform,
            &q_inspectables,
            &q_names,
            &q_parents,
            &q_children,
        ) {
            l_new_objects.push(object);
        }
    }

    l_despawned_entities.clear();
    l_despawned_entities.extend(
        l_notified
            .iter()
            .filter(|ent| !q_inspectables.contains(**ent) && !q_others.contains(**ent))
            .cloned(),
    );

    let win = q_window.single();

    for ent in l_despawned_entities.iter() {
        // TODO: Make this take an array like the multi-spawn event
        wasm::dispatch_to_js(win, "entity-despawn", ent)?;
        l_notified.remove(ent);
    }

    if !l_new_objects.is_empty() {
        // TODO: Remove the single-spawn event
        wasm::dispatch_to_js(win, "entity-multi-spawn", &*l_new_objects)?;
    }

    Ok(())
}

fn build_scene_object(
    notified: &mut HashSet<Entity>,
    missing_parents: &mut VecDeque<Entity>,
    ent: Entity,
    xform: &Transform,
    world_xform: &GlobalTransform,
    q_inspectables: &Query<(Entity, &Transform, &GlobalTransform), With<Inspectable>>,
    q_names: &Query<&Name>,
    q_parents: &Query<&Children>,
    q_children: &Query<&Parent>,
) -> Option<SceneObject> {
    if notified.contains(&ent) {
        return None;
    }

    notified.insert(ent);

    let mut result = SceneObject::build(ent)
        .with_transform(xform)
        .with_global_transform(world_xform);

    if let Ok(children) = q_parents.get(ent) {
        result = result.with_children(children.iter().cloned().collect());
    }
    if let Ok(parent) = q_children.get(ent) {
        let parent_ent = **parent;
        result = result.with_parent(parent_ent);

        if !notified.contains(&parent_ent) && !q_inspectables.contains(parent_ent) {
            missing_parents.push_back(parent_ent);
        }
    }

    result = result.with_name(match q_names.get(ent) {
        Ok(name) => name.into(),
        Err(_) => format!("{ent:?}"),
    });

    Some(result.finish())
}
