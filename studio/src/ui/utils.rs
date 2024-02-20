use ::serde::{Deserialize, Serialize};
use bevy::{ecs::component::ComponentId, prelude::*, reflect::{DynamicEnum, DynamicStruct, DynamicVariant, ReflectFromPtr, TypeInfo, VariantInfo}, utils::HashMap};
use space_editor::{prelude::PrefabMarker, space_prefab::component::*};

use crate::scene::SceneElement;

trait NodeProp: Clone + PartialEq {}

#[derive(Serialize, Deserialize)]
pub enum Mutation {
    SetAttibute {
        id: String,
        attribute: String,
        value: String,
    },
    ReplaceAll(VNode),
    RemoveAll,
}

#[derive(Default)]
pub struct Component {
    pub tag: String,
    pub attributes: Vec<(String, String)>,
    pub children: Vec<Element>,
    pub onchange: Option<Box<dyn WebEventExecutor + Send + Sync>>,
    pub onclick: Option<Box<dyn WebEventExecutor + Send + Sync>>,
}

pub enum Element {
    Text(String),
    Component(Component),
}

pub trait WebEventExecutor {
    fn execute(&mut self, world: &mut World, data: String);
}

#[derive(Default)]
pub struct WebEventStorage {
    pub events: HashMap<String, Box<dyn WebEventExecutor + Send + Sync>>,
}

impl WebEventStorage {
    pub fn add_event(&mut self, id: String, event: Box<dyn WebEventExecutor + Send + Sync>) {
        self.events.insert(id, event);
    }

    pub fn execute(&mut self, world: &mut World, id: String, data: String) {
        if let Some(event) = self.events.get_mut(&id) {
            event.execute(world, data);
        } else {
            println!("No event found for id: {}", id);
        }
    }
}

#[derive(Default, Resource)]
pub struct VirtualDom {
    last_node: Option<VNode>,
    pub on_change_events: WebEventStorage,
    pub on_click_events: WebEventStorage,
}

impl VirtualDom {
    pub fn render(&mut self, element: Option<Element>) -> Vec<Mutation> {
        let mut id_counter = 0;

        if let Some(element) = element {
            let node = self.recursive_unpack_element(&mut id_counter, element);
            self.create_mutations(node)
        } else {
            self.create_mutations(VNode::Text("".to_string()))
        }
    }

    fn recursive_unpack_element(&mut self, counter: &mut usize, element: Element) -> VNode {
        match element {
            Element::Text(text) => VNode::Text(text),
            Element::Component(component) => {
                let id = format!("node_{}", *counter);
                *counter += 1;

                let mut onchange_enabled = false;
                if let Some(onchange) = component.onchange {
                    self.on_change_events.add_event(id.clone(), onchange);
                    onchange_enabled = true;
                }

                let mut onclick_enabled = false;
                if let Some(onclick) = component.onclick {
                    self.on_click_events.add_event(id.clone(), onclick);
                    onclick_enabled = true;
                }

                VNode::Component(VComponent {
                    tag: component.tag,
                    attributes: component.attributes.into_iter().collect(),
                    id: Some(id),
                    onchange: onchange_enabled,
                    onclick: onclick_enabled,
                    children: component
                        .children
                        .into_iter()
                        .map(|e| self.recursive_unpack_element(counter, e))
                        .collect(),
                })
            }
        }
    }

    pub fn create_mutations(&mut self, node: VNode) -> Vec<Mutation> {
        if self.last_node == Some(node.clone()) {
            vec![]
        } else {
            if let Some(last_node) = &self.last_node {
                if last_node.is_same_structure(&node) {
                    let muts = node.get_same_structure_muts(last_node);
                    self.last_node = Some(node.clone());
                    return muts;
                }
            }

            self.last_node = Some(node.clone());
            vec![Mutation::ReplaceAll(node)]
        }
    }
}


pub fn blocking_component() -> Option<Element> {
    Some(Element::Component(Component {
        tag: "div".to_string(),
        attributes: vec![
            ("style".to_string(), "padding: 10px;".to_string()),
            ("class".to_string(), "inspector".to_string()),
        ],
        children: vec![
            Element::Component(Component {
                tag: "button".to_string(),
                attributes: vec![
                    ("class".to_string(), "blocking-button".to_string()),
                ],
                onclick: Some(Box::new(WorldCommandExecutor {
                    func: Box::new(|world| {
                        world.spawn((
                            MeshPrimitivePrefab::Cube(1.0),
                            SpatialBundle::default(),
                            SceneElement,
                            PrefabMarker
                        ));
                    })
                })),
                children: vec![
                    Element::Text("Add blocking mesh".to_string())
                ],
                ..default()
            })
        ],
        ..default()
    }))
}

#[cfg(feature = "wasm")]
pub mod wasm {
    use wasm_bindgen::prelude::*;

    use crate::wasm::{StaticBuffer, WasmMessageBuffer};

    pub static ONCHAGE_BUFFER: WasmMessageBuffer<Vec<(String, String, String)>> = WasmMessageBuffer::new();

    #[wasm_bindgen(js_name = register_new_change)]
    pub fn register_new_change(id: String, kind: String, value: String) {
        let mut data = ONCHAGE_BUFFER.take_message().unwrap_or_default();
        data.push((id, kind, value));
        ONCHAGE_BUFFER.write_message(data);
    }
}

struct ReflectValueExecutor {
    change: ReflectChange,
}

impl From<ReflectChange> for ReflectValueExecutor {
    fn from(change: ReflectChange) -> Self {
        ReflectValueExecutor { change }
    }
}

impl WebEventExecutor for ReflectValueExecutor {
    fn execute(&mut self, world: &mut World, value: String) {
        unsafe {
            let world = world.as_unsafe_world_cell();
            let app_type_registry: &AppTypeRegistry =
                world.get_resource::<AppTypeRegistry>().unwrap();
            let type_registry = app_type_registry.read();

            let e = world.get_entity(self.change.entity_id).unwrap();
            let c_id = ComponentId::new(
                self.change
                    .compoent_id
                    .split('(')
                    .nth(1)
                    .unwrap()
                    .split(')')
                    .next()
                    .unwrap()
                    .parse()
                    .unwrap(),
            );

            if let Some(mut c) = e.get_mut_by_id(c_id) {
                let refl = type_registry
                    .get_type_data::<ReflectFromPtr>(
                        world
                            .components()
                            .get_info_unchecked(c_id)
                            .type_id()
                            .unwrap(),
                    )
                    .unwrap()
                    .as_reflect_mut(c.as_mut());

                if let Ok(refl_value) = refl.reflect_path_mut(self.change.reflect_path.as_str()) {
                    if let Some(v) = refl_value.downcast_mut::<String>() {
                        *v = value;
                    } else if let Some(v) = refl_value.downcast_mut::<f32>() {
                        if let Ok(value) = value.parse::<f32>() {
                            *v = value;
                        }
                    } else if let Some(v) = refl_value.downcast_mut::<f64>() {
                        if let Ok(value) = value.parse::<f64>() {
                            *v = value;
                        }
                    } else if let Some(v) = refl_value.downcast_mut::<bool>() {
                        if let Ok(value) = value.parse::<bool>() {
                            *v = value;
                        }
                    } else if let Some(v) = refl_value.downcast_mut::<i32>() {
                        if let Ok(value) = value.parse::<i32>() {
                            *v = value;
                        }
                    } else if let Some(v) = refl_value.downcast_mut::<i64>() {
                        if let Ok(value) = value.parse::<i64>() {
                            *v = value;
                        }
                    } else if let Some(v) = refl_value.downcast_mut::<u32>() {
                        if let Ok(value) = value.parse::<u32>() {
                            *v = value;
                        }
                    } else if let Some(v) = refl_value.downcast_mut::<u64>() {
                        if let Ok(value) = value.parse::<u64>() {
                            *v = value;
                        }
                    }
                }
            }
        }
    }
}

struct WorldCommandExecutor {
    func: Box<dyn Fn(&mut World) + Send + Sync>,
}

impl WebEventExecutor for WorldCommandExecutor {
    fn execute(&mut self, world: &mut World, _value: String) {
        (self.func)(world);
    }
}

struct EntityChangeExecutor {
    entity: Entity,
    func: Box<dyn Fn(&mut World, Entity, String) + Send + Sync>,
}

impl WebEventExecutor for EntityChangeExecutor {
    fn execute(&mut self, world: &mut World, value: String) {
        (self.func)(world, self.entity, value);
    }
}

impl EntityChangeExecutor {
    fn new(
        entity: Entity,
        func: impl Fn(&mut World, Entity, String) + Send + Sync + 'static,
    ) -> Self {
        EntityChangeExecutor {
            entity,
            func: Box::new(func),
        }
    }

    fn new_boxed(
        entity: Entity,
        func: impl Fn(&mut World, Entity, String) + Send + Sync + 'static,
    ) -> Box<Self> {
        Box::new(EntityChangeExecutor::new(entity, func))
    }
}

fn euler_to_degrees(euler: (f32, f32, f32)) -> (f32, f32, f32) {
    (
        euler.0.to_degrees(),
        euler.1.to_degrees(),
        euler.2.to_degrees(),
    )
}

fn check_predefined(r: &dyn Reflect, onchange: ReflectChange) -> Option<Element> {
    if let Some(name) = r.downcast_ref::<Name>() {
        Some(Element::Component(Component {
            tag: "div".to_string(),
            attributes: vec![(
                "style".to_string(),
                "font-weight: bold; font-size: 24px".to_string(),
            )],
            children: vec![Element::Text(format!("{}", &name))],
            ..default()
        }))
    } else if let Some(transform) = r.downcast_ref::<Transform>() {
        Some(Element::Component(Component {
            tag: "div".to_string(),
            attributes: vec![(
                "style".to_string(),
                "display: flex; flex-direction: column".to_string(),
            )],
            children: vec![
                Element::Text("Pos: ".to_string()),
                show_reflect(&transform.translation, onchange.add_path("translation")).unwrap(),
                Element::Text("Rot: ".to_string()),
                show_reflect(&transform.rotation, onchange.add_path("rotation")).unwrap(),
                Element::Text("Scale: ".to_string()),
                show_reflect(&transform.scale, onchange.add_path("scale")).unwrap(),
            ],
            ..default()
        }))
    } else if let Some(quat) = r.downcast_ref::<Quat>() {
        let euler = quat.to_euler(EulerRot::XYZ);
        let degrees = euler_to_degrees(euler);
        Some(Element::Component(Component {
            tag: "div".to_string(),
            attributes: vec![("class".to_string(), "struct-grid".to_string())],
            children: vec![
                Element::Text("X: ".to_string()),
                Element::Component(Component {
                    tag: "input".to_string(),
                    attributes: vec![
                        ("type".to_string(), "number".to_string()),
                        ("value".to_string(), format!("{:.2}", degrees.0)),
                        ("step".to_string(), "0.01".to_string()),
                    ],
                    children: vec![],
                    onchange: Some(EntityChangeExecutor::new_boxed(
                        onchange.entity_id,
                        |world: &mut World, entity, value: String| {
                            let Some(mut value) = value.parse::<f32>().ok() else {
                                return;
                            };
                            if value == -180.0 {
                                value = 180.0;
                            }
                            let value = value.to_radians();
                            let mut e = world.get_entity_mut(entity).unwrap();

                            let mut transform = e.get_mut::<Transform>().unwrap();
                            if transform.is_changed() {
                                return;
                            }
                            let euler = transform.rotation.to_euler(EulerRot::XYZ);
                            transform.rotation =
                                Quat::from_euler(EulerRot::XYZ, value, euler.1, euler.2);
                        },
                    )),
                    ..default()
                }),
                Element::Text("Y: ".to_string()),
                Element::Component(Component {
                    tag: "input".to_string(),
                    attributes: vec![
                        ("type".to_string(), "number".to_string()),
                        ("value".to_string(), format!("{:.2}", degrees.1)),
                        ("step".to_string(), "0.01".to_string()),
                    ],
                    children: vec![],
                    onchange: Some(EntityChangeExecutor::new_boxed(
                        onchange.entity_id,
                        |world: &mut World, entity, value: String| {
                            let Some(mut value) = value.parse::<f32>().ok() else {
                                return;
                            };
                            if value == -180.0 {
                                value = 180.0;
                            }
                            let mut e = world.get_entity_mut(entity).unwrap();
                            let mut transform = e.get_mut::<Transform>().unwrap();
                            if transform.is_changed() {
                                return;
                            }
                            let value = value.to_radians();
                            let euler = transform.rotation.to_euler(EulerRot::XYZ);
                            transform.rotation *= Quat::from_axis_angle(Vec3::Y, value - euler.1);
                        },
                    )),
                    ..default()
                }),
                Element::Text("Z: ".to_string()),
                Element::Component(Component {
                    tag: "input".to_string(),
                    attributes: vec![
                        ("type".to_string(), "number".to_string()),
                        ("value".to_string(), format!("{:.2}", degrees.2)),
                        ("step".to_string(), "0.01".to_string()),
                    ],
                    children: vec![],
                    onchange: Some(EntityChangeExecutor::new_boxed(
                        onchange.entity_id,
                        |world: &mut World, entity, value: String| {
                            let Some(mut value) = value.parse::<f32>().ok() else {
                                return;
                            };
                            if value == -180.0 {
                                value = 180.0;
                            }
                            let value = value.to_radians();
                            let mut e = world.get_entity_mut(entity).unwrap();
                            let mut transform = e.get_mut::<Transform>().unwrap();
                            if transform.is_changed() {
                                return;
                            }
                            let euler = transform.rotation.to_euler(EulerRot::XYZ);
                            transform.rotation =
                                Quat::from_euler(EulerRot::XYZ, euler.0, euler.1, value);
                        },
                    )),
                    ..default()
                }),
            ],
            ..default()
        }))
    } else {
        r.downcast_ref::<Vec3>().map(|vec| {
            Element::Component(Component {
                tag: "div".to_string(),
                attributes: vec![("class".to_string(), "struct-grid".to_string())],
                children: vec![
                    Element::Text("X: ".to_string()),
                    Element::Component(Component {
                        tag: "input".to_string(),
                        attributes: vec![
                            ("type".to_string(), "number".to_string()),
                            ("value".to_string(), format!("{:.2}", vec.x)),
                            ("step".to_string(), "0.01".to_string()),
                        ],
                        children: vec![],
                        onchange: Some(Box::new(ReflectValueExecutor::from(
                            onchange.add_path("x"),
                        ))),
                        ..default()
                    }),
                    Element::Text("Y: ".to_string()),
                    Element::Component(Component {
                        tag: "input".to_string(),
                        attributes: vec![
                            ("type".to_string(), "number".to_string()),
                            ("value".to_string(), format!("{:.2}", vec.y)),
                            ("step".to_string(), "0.01".to_string()),
                        ],
                        children: vec![],
                        onchange: Some(Box::new(ReflectValueExecutor::from(
                            onchange.add_path("y"),
                        ))),
                        ..default()
                    }),
                    Element::Text("Z: ".to_string()),
                    Element::Component(Component {
                        tag: "input".to_string(),
                        attributes: vec![
                            ("type".to_string(), "number".to_string()),
                            ("value".to_string(), format!("{:.2}", vec.z)),
                            ("step".to_string(), "0.01".to_string()),
                        ],
                        children: vec![],
                        onchange: Some(Box::new(ReflectValueExecutor::from(
                            onchange.add_path("z"),
                        ))),
                        ..default()
                    }),
                ],
                ..default()
            })
        })
    }
}

pub fn show_reflect(r: &dyn Reflect, reflect: ReflectChange) -> Option<Element> {
    if let Some(vnode) = check_predefined(r, reflect.clone()) {
        return Some(vnode);
    }

    match r.reflect_ref() {
        bevy::reflect::ReflectRef::Struct(_v) => None,
        bevy::reflect::ReflectRef::TupleStruct(_) => None,
        bevy::reflect::ReflectRef::Tuple(_) => None,
        bevy::reflect::ReflectRef::List(_) => None,
        bevy::reflect::ReflectRef::Array(_) => None,
        bevy::reflect::ReflectRef::Map(_) => None,
        bevy::reflect::ReflectRef::Enum(v) => {
            let Some(type_info) = v.get_represented_type_info() else {
                error!("invalid reflect impl: type info mismatch");
                return None;
            };
            let type_info = match type_info {
                TypeInfo::Enum(info) => info,
                _ => unreachable!("invalid reflect impl: type info mismatch"),
            };


            Some(Element::Component(Component {
                tag: "div".to_string(),
                attributes: vec![
                    ("style".to_string(), "display: flex; flex-direction: column".to_string()),
                ],
                children: vec![
                    Element::Component(Component {
                        tag: "select".to_string(),
                        attributes: vec![],
                        children: type_info.iter().enumerate().map(|(i, variant)| {
                            let mut attributes = vec![
                                ("value".to_string(), variant.name().to_string()),
                            ];

                            let is_active_variant = (i == v.variant_index());
                            if is_active_variant {
                                attributes.push(("selected".to_string(), "".to_string()));
                            }

                            Element::Component(Component {
                                tag: "option".to_string(),
                                attributes,
                                children: vec![Element::Text(variant.name().to_string())],
                                ..default()
                            })
                        }).collect(),
                        onchange: Some(
                            EntityChangeExecutor::new_boxed(
                                reflect.entity_id,
                                |world, entity, value : String| {
                                    info!("MeshPrimitivePrefab change: {:?} {}", entity, value);

                                    let Some(mut e) = world.get_entity_mut(entity) else {
                                        return;
                                    };

                                    let Some(mut variant) = e.get_mut::<MeshPrimitivePrefab>() else {
                                        return;
                                    };

                                    // pub enum MeshPrimitivePrefab {
                                    //     Cube(f32),
                                    //     Box(BoxPrefab),
                                    //     Sphere(SpherePrefab),
                                    //     Quad(QuadPrefab),
                                    //     Capsule(CapsulePrefab),
                                    //     Circle(CirclePrefab),
                                    //     Cylinder(CylinderPrefab),
                                    //     Icosphere(IcospherePrefab),
                                    //     Plane(PlanePrefab),
                                    //     RegularPolygon(RegularPolygonPrefab),
                                    //     Torus(TorusPrefab),
                                    // }

                                    if value == "Cube" {
                                        *variant = MeshPrimitivePrefab::Cube(1.0);
                                    } else if value == "Box" {
                                        *variant = MeshPrimitivePrefab::Box(BoxPrefab::default());
                                    } else if value == "Sphere" {
                                        *variant = MeshPrimitivePrefab::Sphere(SpherePrefab::default());
                                    } else if value == "Quad" {
                                        *variant = MeshPrimitivePrefab::Quad(QuadPrefab::default());
                                    } else if value == "Capsule" {
                                        *variant = MeshPrimitivePrefab::Capsule(CapsulePrefab::default());
                                    } else if value == "Circle" {
                                        *variant = MeshPrimitivePrefab::Circle(CirclePrefab::default());
                                    } else if value == "Cylinder" {
                                        *variant = MeshPrimitivePrefab::Cylinder(CylinderPrefab::default());
                                    } else if value == "Icosphere" {
                                        *variant = MeshPrimitivePrefab::Icosphere(IcospherePrefab::default());
                                    } else if value == "Plane" {
                                        *variant = MeshPrimitivePrefab::Plane(PlanePrefab::default());
                                    } else if value == "RegularPolygon" {
                                        *variant = MeshPrimitivePrefab::RegularPolygon(RegularPolygonPrefab::default());
                                    } else if value == "Torus" {
                                        *variant = MeshPrimitivePrefab::Torus(TorusPrefab::default());
                                    }
                                }
                            )
                        ),
                        ..default()  
                    })
                ],
                ..default()
            }))
        },
        bevy::reflect::ReflectRef::Value(v) => {
            if let Some(s) = v.downcast_ref::<String>() {
                Some(Element::Component(Component {
                    tag: "input".to_string(),
                    attributes: vec![
                        ("type".to_string(), "text".to_string()),
                        ("value".to_string(), s.to_string()),
                    ],
                    children: vec![],
                    onchange: Some(Box::new(ReflectValueExecutor::from(reflect))),
                    ..default()
                }))
            } else if let Some(b) = v.downcast_ref::<bool>() {
                Some(Element::Component(Component {
                    tag: "input".to_string(),
                    attributes: vec![
                        ("type".to_string(), "checkbox".to_string()),
                        ("value".to_string(), b.to_string()),
                    ],
                    children: vec![],
                    onchange: Some(Box::new(ReflectValueExecutor::from(reflect))),
                    ..default()
                }))
            } else if let Some(i) = v.downcast_ref::<i32>() {
                Some(Element::Component(Component {
                    tag: "input".to_string(),
                    attributes: vec![
                        ("type".to_string(), "number".to_string()),
                        ("value".to_string(), i.to_string()),
                    ],
                    children: vec![],
                    onchange: Some(Box::new(ReflectValueExecutor::from(reflect))),
                    ..default()
                }))
            } else if let Some(f) = v.downcast_ref::<f32>() {
                Some(Element::Component(Component {
                    tag: "input".to_string(),
                    attributes: vec![
                        ("type".to_string(), "number".to_string()),
                        ("value".to_string(), f.to_string()),
                    ],
                    children: vec![],
                    onchange: Some(Box::new(ReflectValueExecutor::from(reflect))),
                    ..default()
                }))
            } else {
                Some(Element::Text(format!("{:?}", v)))
            }
        }
    }
}

impl<T: Clone + PartialEq> NodeProp for T {}

#[derive(Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct VComponent {
    pub tag: String,
    pub attributes: Vec<(String, String)>,
    pub children: Vec<VNode>,
    pub id: Option<String>,
    pub onchange: bool,
    pub onclick: bool,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum VNode {
    Text(String),
    Component(VComponent),
}

impl VNode {
    pub fn is_same_structure(&self, other: &VNode) -> bool {
        match (self, other) {
            (VNode::Text(s1), VNode::Text(s2)) => s1 == s2,
            (VNode::Text(_), VNode::Component(_)) => false,
            (VNode::Component(_), VNode::Text(_)) => false,
            (VNode::Component(c1), VNode::Component(c2)) => {
                if c1.tag != c2.tag {
                    false
                } else if c1.children.len() != c2.children.len() {
                    return false;
                } else {
                    for i in 0..c1.children.len() {
                        if !c1.children[i].is_same_structure(&c2.children[i]) {
                            return false;
                        }
                    }
                    return true;
                }
            }
        }
    }

    pub fn get_same_structure_muts(&self, older: &VNode) -> Vec<Mutation> {
        let mut res = vec![];
        match (self, older) {
            (VNode::Text(_s1), VNode::Text(_s2)) => res,
            (VNode::Text(_), VNode::Component(_)) => res,
            (VNode::Component(_), VNode::Text(_)) => res,
            (VNode::Component(new), VNode::Component(old)) => {
                for idx in 0..new.attributes.len() {
                    if new.attributes.get(idx) != old.attributes.get(idx) {
                        res.push(Mutation::SetAttibute {
                            id: old.id.as_ref().unwrap().clone(),
                            attribute: new.attributes[idx].0.clone(),
                            value: new.attributes[idx].1.clone(),
                        });
                    }
                }
                for i in 0..new.children.len() {
                    res.append(&mut new.children[i].get_same_structure_muts(&old.children[i]));
                }
                res
            }
        }
    }
}

pub struct InspectedEntity {
    pub id: Option<Entity>,
    pub components: Vec<Box<dyn Reflect>>,
}

pub struct ComponentProp {
    pub component: Box<dyn Reflect>,
}

impl Clone for ComponentProp {
    fn clone(&self) -> Self {
        Self {
            component: self.component.clone_value(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ReflectChange {
    pub entity_id: Entity,
    pub reflect_path: String,
    pub compoent_id: String,
}

impl ReflectChange {
    pub fn add_path(&self, path: &str) -> ReflectChange {
        ReflectChange {
            entity_id: self.entity_id,
            compoent_id: self.compoent_id.clone(),
            reflect_path: format!("{}.{}", self.reflect_path, path),
        }
    }
}

impl PartialEq for ComponentProp {
    fn eq(&self, other: &Self) -> bool {
        recursive_reflect_eq(self.component.as_ref(), other.component.as_ref())
    }
}

pub fn recursive_reflect_eq(a: &dyn Reflect, b: &dyn Reflect) -> bool {
    let ref_a = a.reflect_ref();
    let _ref_b = b.reflect_ref();

    match ref_a {
        bevy::reflect::ReflectRef::Struct(a) => {
            let bevy::reflect::ReflectRef::Struct(b) = b.reflect_ref() else {
                return false;
            };

            if a.field_len() != b.field_len() {
                return false;
            }

            for i in 0..a.field_len() {
                if !recursive_reflect_eq(a.field_at(i).unwrap(), b.field_at(i).unwrap()) {
                    return false;
                }
            }
        }
        bevy::reflect::ReflectRef::TupleStruct(a) => {
            let bevy::reflect::ReflectRef::TupleStruct(b) = b.reflect_ref() else {
                return false;
            };

            if a.field_len() != b.field_len() {
                return false;
            }

            for i in 0..a.field_len() {
                if !recursive_reflect_eq(a.field(i).unwrap(), b.field(i).unwrap()) {
                    return false;
                }
            }
        }
        bevy::reflect::ReflectRef::Tuple(a) => {
            let bevy::reflect::ReflectRef::Tuple(b) = b.reflect_ref() else {
                return false;
            };

            if a.field_len() != b.field_len() {
                return false;
            }

            for i in 0..a.field_len() {
                if !recursive_reflect_eq(a.field(i).unwrap(), b.field(i).unwrap()) {
                    return false;
                }
            }
        }
        bevy::reflect::ReflectRef::List(a) => {
            let bevy::reflect::ReflectRef::List(b) = b.reflect_ref() else {
                return false;
            };

            if a.len() != b.len() {
                return false;
            }

            for i in 0..a.len() {
                if !recursive_reflect_eq(a.get(i).unwrap(), b.get(i).unwrap()) {
                    return false;
                }
            }
        }
        bevy::reflect::ReflectRef::Array(a) => {
            let bevy::reflect::ReflectRef::Array(b) = b.reflect_ref() else {
                return false;
            };

            if a.len() != b.len() {
                return false;
            }

            for i in 0..a.len() {
                if !recursive_reflect_eq(a.get(i).unwrap(), b.get(i).unwrap()) {
                    return false;
                }
            }
        }
        bevy::reflect::ReflectRef::Map(a) => {
            let bevy::reflect::ReflectRef::Map(b) = b.reflect_ref() else {
                return false;
            };

            if a.len() != b.len() {
                return false;
            };

            for i in 0..a.len() {
                let (a_key, a_value) = a.get_at(i).unwrap();
                let Some(b_value) = b.get(a_key) else {
                    return false;
                };

                if !recursive_reflect_eq(a_value, b_value) {
                    return false;
                }
            }
        }
        bevy::reflect::ReflectRef::Enum(a) => {
            let bevy::reflect::ReflectRef::Enum(b) = b.reflect_ref() else {
                return false;
            };

            if a.variant_name() != b.variant_name() {
                return false;
            }

            if a.field_len() != b.field_len() {
                return false;
            }

            for i in 0..a.field_len() {
                if !recursive_reflect_eq(a.field_at(i).unwrap(), b.field_at(i).unwrap()) {
                    return false;
                }
            }
        }
        bevy::reflect::ReflectRef::Value(a) => {
            let bevy::reflect::ReflectRef::Value(b) = b.reflect_ref() else {
                return false;
            };

            if let Some(res) = a.reflect_partial_eq(b) {
                return res;
            } else {
                return false;
            }
        }
    }

    false
}

impl PartialEq for InspectedEntity {
    fn eq(&self, other: &Self) -> bool {
        if self.id != other.id {
            return false;
        }

        if self.components.len() != other.components.len() {
            return false;
        }

        for i in 0..self.components.len() {
            if !recursive_reflect_eq(self.components[i].as_ref(), other.components[i].as_ref()) {
                return false;
            }
        }

        true
    }
}

impl InspectedEntity {
    pub fn none() -> Self {
        Self {
            id: None,
            components: Vec::new(),
        }
    }
}
