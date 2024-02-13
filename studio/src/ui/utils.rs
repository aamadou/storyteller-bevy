
use std::any::Any;

use bevy::{ecs::component::ComponentId, prelude::*, reflect::ReflectFromPtr, utils::HashMap};
use ::serde::{Serialize, Deserialize};
use uuid::*;

trait NodeProp : Clone + PartialEq {

}

#[derive(Serialize, Deserialize)]
pub enum Mutation {
    ReplaceAll(VNode),
    RemoveAll
}

#[derive(Default)]
pub struct Component {
    pub tag : String,
    pub attributes : Vec<(String, String)>,
    pub children : Vec<Element>,
    pub onchange: Option<Box<dyn WebEventExecutor + Send + Sync>>
}

pub enum Element {
    Text(String),
    Component(Component)
}

pub trait WebEventExecutor {
    fn execute(&mut self, world: &mut World, data: String); 
}

#[derive(Default)]
pub struct WebEventStorage {
    pub events : HashMap<String, Box<dyn WebEventExecutor + Send + Sync>>,
}

impl WebEventStorage {
    pub fn add_event(&mut self, id : String, event : Box<dyn WebEventExecutor + Send + Sync>) {
        self.events.insert(id, event);
    }

    pub fn execute(&mut self, world: &mut World, id : String, data : String) {
        if let Some(event) = self.events.get_mut(&id) {
            event.execute(world, data);
        } else {
            println!("No event found for id: {}", id);
        }
    }
}

#[derive(Default, Resource)]
pub struct VirtualDom {
    last_node : Option<VNode>,
    pub on_change_events: WebEventStorage,
}

impl VirtualDom {

    pub fn render(&mut self, element: Option<Element>) -> Vec<Mutation> {
        let mut id_counter = 0;

        if let Some(element) = element {
            let node = self.recursive_unpack_element(&mut id_counter, element);
            return self.create_mutations(Some(node));
        } else {
            return self.create_mutations(None);
        }
    }

    fn recursive_unpack_element(&mut self, counter : &mut usize, element : Element) -> VNode {
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

                VNode::Component(VComponent {
                    tag: component.tag,
                    attributes: component.attributes,
                    id: Some(id),
                    onchange: onchange_enabled,
                    children: component.children.into_iter().map(|e| self.recursive_unpack_element(counter, e)).collect()
                })
            },
        }
    } 

    pub fn create_mutations(&mut self, node : Option<VNode>) -> Vec<Mutation> {
        if self.last_node == node {
            return vec![]
        } else {
            self.last_node = node.clone();
            return vec![Mutation::ReplaceAll(node.unwrap())]
        }
    }
}

#[cfg(feature = "wasm")]
pub mod wasm {
    use wasm_bindgen::prelude::*;

    use crate::wasm::{StaticBuffer, WasmMessageBuffer};

    pub static ONCHAGE_BUFFER: WasmMessageBuffer<Vec<(String, String)>> = WasmMessageBuffer::new();

    #[wasm_bindgen(js_name = register_new_change)]
    pub fn register_new_change(id : String, value : String) {
        let mut data = ONCHAGE_BUFFER.take_message().unwrap_or_default();
        data.push((id, value));
        ONCHAGE_BUFFER.write_message(data);
    }
}

struct ReflectValueExecutor {
    change : ReflectChange
}

impl From<ReflectChange> for ReflectValueExecutor {
    fn from(change : ReflectChange) -> Self {
        ReflectValueExecutor { change }
    }
}

impl WebEventExecutor for ReflectValueExecutor {
    fn execute(&mut self, world: &mut World, value: String) {
        unsafe {
            let world = world.as_unsafe_world_cell();
            let app_type_registry: &AppTypeRegistry = world.get_resource::<AppTypeRegistry>().unwrap();
            let type_registry = app_type_registry.read();


            let mut e = world.get_entity(self.change.entity_id).unwrap();
            let c_id = ComponentId::new(
                self.change.compoent_id.split('(').nth(1).unwrap().split(')').next().unwrap().parse().unwrap()
            );

            if let Some(mut c) = e.get_mut_by_id(c_id) {

                let mut refl = type_registry.get_type_data::<ReflectFromPtr>(world.components().get_info_unchecked(c_id).type_id().unwrap())
                    .unwrap()
                    .as_reflect_mut(c.as_mut());

                if let Ok(refl_value) = refl.reflect_path_mut(self.change.reflect_path.as_str()) {
                    if let Some(v) = refl_value.downcast_mut::<String>() {
                        *v = value;
                    } else if let Some(v) = refl_value.downcast_mut::<f32>() {
                        *v = value.parse().unwrap();
                    } else if let Some(v) = refl_value.downcast_mut::<f64>() {
                        *v = value.parse().unwrap();
                    } else if let Some(v) = refl_value.downcast_mut::<bool>() {
                        *v = value.parse().unwrap();
                    } else if let Some(v) = refl_value.downcast_mut::<i32>() {
                        *v = value.parse().unwrap();
                    } else if let Some(v) = refl_value.downcast_mut::<i64>() {
                        *v = value.parse().unwrap();
                    } else if let Some(v) = refl_value.downcast_mut::<u32>() {
                        *v = value.parse().unwrap();
                    } else if let Some(v) = refl_value.downcast_mut::<u64>() {
                        *v = value.parse().unwrap();
                    }
                }
            }
        }
    }
}

fn check_predefined(r : &dyn Reflect, onchange : ReflectChange) -> Option<Element> {
    if let Some(name) = r.downcast_ref::<Name>() {
        Some(Element::Component(Component {
            tag: "input".to_string(),
            attributes: vec![
                ("type".to_string(), "text".to_string()),
                ("value".to_string(), name.to_string()),
            ],
            children: vec![],
            onchange: Some(Box::new(ReflectValueExecutor::from(onchange.add_path("name"))))
        }))
    } else if let Some(transform) = r.downcast_ref::<Transform>() {
        
        Some(Element::Component(Component {
            tag: "div".to_string(),
            attributes: vec![],
            children: vec![
                Element::Text("Pos: ".to_string()),
                show_reflect(&transform.translation, onchange.add_path("translation")).unwrap(),
                Element::Text("Rot: ".to_string()),
                show_reflect(&transform.rotation, onchange.add_path("rotation")).unwrap(),
                Element::Text("Scale: ".to_string()),
                show_reflect(&transform.scale, onchange.add_path("scale")).unwrap(),
            ],
            onchange: None
        }))
    } else if let Some(quat) = r.downcast_ref::<Quat>() {
        let euler = quat.to_euler(EulerRot::XYZ);
        Some(Element::Component(Component {
            tag: "div".to_string(),
            attributes: vec![],
            children: vec![
                Element::Text("X: ".to_string()),
                Element::Component(Component {
                    tag: "input".to_string(),
                    attributes: vec![
                        ("type".to_string(), "number".to_string()),
                        ("value".to_string(), euler.0.to_string()),
                    ],
                    children: vec![],
                    onchange: None
                }),
                Element::Text("Y: ".to_string()),
                Element::Component(Component {
                    tag: "input".to_string(),
                    attributes: vec![
                        ("type".to_string(), "number".to_string()),
                        ("value".to_string(), euler.1.to_string()),
                    ],
                    children: vec![],
                    onchange: None
                }),
                Element::Text("Z: ".to_string()),
                Element::Component(Component {
                    tag: "input".to_string(),
                    attributes: vec![
                        ("type".to_string(), "number".to_string()),
                        ("value".to_string(), euler.2.to_string()),
                    ],
                    children: vec![],
                    onchange: None
                })
            ],
            onchange: None
        }))
    }
    else if let Some(vec) = r.downcast_ref::<Vec3>() {
        Some(Element::Component(Component {
            tag: "div".to_string(),
            attributes: vec![],
            children: vec![
                Element::Text("X: ".to_string()),
                Element::Component(Component {
                    tag: "input".to_string(),
                    attributes: vec![
                        ("type".to_string(), "number".to_string()),
                        ("value".to_string(), vec.x.to_string()),
                    ],
                    children: vec![],
                    onchange: Some(Box::new(ReflectValueExecutor::from(onchange.add_path("x"))))
                }),
                Element::Text("Y: ".to_string()),
                Element::Component(Component {
                    tag: "input".to_string(),
                    attributes: vec![
                        ("type".to_string(), "number".to_string()),
                        ("value".to_string(), vec.y.to_string()),
                    ],
                    children: vec![],
                    onchange: Some(Box::new(ReflectValueExecutor::from(onchange.add_path("y"))))
                }),
                Element::Text("Z: ".to_string()),
                Element::Component(Component {
                    tag: "input".to_string(),
                    attributes: vec![
                        ("type".to_string(), "number".to_string()),
                        ("value".to_string(), vec.z.to_string()),
                    ],
                    children: vec![],
                    onchange: Some(Box::new(ReflectValueExecutor::from(onchange.add_path("z"))))
                })
            ],
            onchange: None
        }))    
    
    } else {
        None
    }
}

pub fn show_reflect(r : &dyn Reflect, reflect: ReflectChange) -> Option<Element> {

    if let Some(vnode) = check_predefined(r, reflect.clone()) {
        return Some(vnode);
    }

    match r.reflect_ref() {
        bevy::reflect::ReflectRef::Struct(v) => {
            None

        },
        bevy::reflect::ReflectRef::TupleStruct(_) => {
            None
        },
        bevy::reflect::ReflectRef::Tuple(_) => {
            None
        },
        bevy::reflect::ReflectRef::List(_) => {
            None
        },
        bevy::reflect::ReflectRef::Array(_) => {
            None
        },
        bevy::reflect::ReflectRef::Map(_) => {
            None
        },
        bevy::reflect::ReflectRef::Enum(_) => {
            None
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
                    onchange: Some(Box::new(ReflectValueExecutor::from(reflect)))
                }))
            } else if let Some(b) = v.downcast_ref::<bool>() {
                Some(Element::Component(Component {
                    tag: "input".to_string(),
                    attributes: vec![
                        ("type".to_string(), "checkbox".to_string()),
                        ("value".to_string(), b.to_string()),
                    ],
                    children: vec![],
                    onchange: Some(Box::new(ReflectValueExecutor::from(reflect)))
                }))
            } else if let Some(i) = v.downcast_ref::<i32>() {
                Some(Element::Component(Component {
                    tag: "input".to_string(),
                    attributes: vec![
                        ("type".to_string(), "number".to_string()),
                        ("value".to_string(), i.to_string()),
                    ],
                    children: vec![],
                    onchange: Some(Box::new(ReflectValueExecutor::from(reflect)))
                }))
            } else if let Some(f) = v.downcast_ref::<f32>() {
                Some(Element::Component(Component {
                    tag: "input".to_string(),
                    attributes: vec![
                        ("type".to_string(), "number".to_string()),
                        ("value".to_string(), f.to_string()),
                    ],
                    children: vec![],
                    onchange: Some(Box::new(ReflectValueExecutor::from(reflect)))
                }))
            } else {
                Some(Element::Text(format!("{:?}", v)))
            }
        },
    }
}

impl<T: Clone + PartialEq> NodeProp for T {}

pub fn show_string(s : &str) -> Option<VNode> {
    Some(VNode::Text(s.to_string())
)}

#[derive(Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct VComponent {
    pub tag: String,
    pub attributes: Vec<(String, String)>,
    pub children: Vec<VNode>,
    pub id: Option<String>,
    pub onchange: bool,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum VNode {
    Text(String),
    Component(VComponent),
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
            component: self.component.clone_value()
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ReflectChange {
    pub entity_id : Entity,
    pub reflect_path: String,
    pub compoent_id: String
}

impl ReflectChange {
    pub fn add_path(&self, path : &str) -> ReflectChange {
        ReflectChange {
            entity_id : self.entity_id,
            compoent_id : self.compoent_id.clone(),
            reflect_path : format!("{}.{}", self.reflect_path, path)
        }
    }

    pub fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

impl<'a> PartialEq for ComponentProp {
    fn eq(&self, other: &Self) -> bool {
        recursive_reflect_eq(self.component.as_ref(), other.component.as_ref())
    }
}

pub fn recursive_reflect_eq(a : &dyn Reflect, b : &dyn Reflect) -> bool {
    let ref_a = a.reflect_ref();
    let ref_b = b.reflect_ref();

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
        },
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
        },
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
        },
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
        },
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
        },
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
        },
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
        },
        bevy::reflect::ReflectRef::Value(a) => {
            let bevy::reflect::ReflectRef::Value(b) = b.reflect_ref() else {
                return false;
            };

            if let Some(res) = a.reflect_partial_eq(b) {
                return res;
            } else {
                return false;
            }
        },
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