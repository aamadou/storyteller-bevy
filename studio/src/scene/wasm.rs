use std::collections::VecDeque;

use bevy::{
    prelude::*, reflect::serde::TypedReflectSerializer, utils::HashSet, window::PrimaryWindow,
};
use serde::Serialize;
use wasm_bindgen::prelude::*;

use crate::wasm::{self, AsJson, WasmMessageBuffer};

use super::Inspectable;

static SCENE_REQUEST: WasmMessageBuffer<()> = WasmMessageBuffer::new();

#[wasm_bindgen(js_name = saveScene)]
pub fn save_scene() {
    wasm::write_message!(SCENE_REQUEST, ());
}

pub(super) fn save_scene_system(world: &mut World) {
    wasm::read_buffer!(SCENE_REQUEST, |_| {
        let mut scene = DynamicScene::from_world(world);
        let type_registry = world.resource::<AppTypeRegistry>();

        #[allow(clippy::borrowed_box)]
        let is_serializable = |reflect: &Box<dyn Reflect>| -> bool {
            TypedReflectSerializer::new(&**reflect, &type_registry.read())
                .serialize(&wasm::SERIALIZER)
                .is_ok()
        };

        // FIXME: This includes some stuff we don't want, including (most
        //        importantly) the _window_. We probably want to grab just the
        //        main camera, the skybox, the glTF scene instance and its
        //        children.
        scene.resources = scene
            .resources
            .into_iter()
            .filter(is_serializable)
            .collect();

        scene.entities = scene
            .entities
            .into_iter()
            .map(|mut ent| {
                ent.components = ent.components.into_iter().filter(is_serializable).collect();
                ent
            })
            .collect();

        match scene.serialize_ron(type_registry) {
            Ok(serialized) => {
                let Some(window) = world.iter_entities().find_map(|ent| {
                    if ent.contains::<PrimaryWindow>() {
                        ent.get::<Window>()
                    } else {
                        None
                    }
                }) else {
                    warn!("Failed to get window from &mut World");
                    return;
                };

                if let Err(err) = wasm::dispatch_to_js(window, "scene-download-ready", &serialized)
                {
                    error!("{err}");
                }
            }
            Err(err) => {
                error!("{err}");
            }
        }
    });
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
        "scene-download-ready":
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
