
use std::any::Any;

use bevy::{prelude::*, utils::HashMap};
use ::serde::{Serialize, Deserialize};
use uuid::*;

trait NodeProp : Clone + PartialEq {

}

#[derive(Serialize, Deserialize)]
pub enum Mutation {
    ReplaceAll(VNode),
    RemoveAll
}

#[derive(Default, Resource)]
pub struct VirtualDom {
    last_node : Option<VNode>,
}

impl VirtualDom {
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
    pub fn register_new_change(change : String, value : String) {
        let mut data = ONCHAGE_BUFFER.take_message().unwrap_or_default();
        data.push((change, value));
        ONCHAGE_BUFFER.write_message(data);
    }
}

fn check_predefined(r : &dyn Reflect, onchange : ReflectChange) -> Option<VNode> {
    if let Some(name) = r.downcast_ref::<Name>() {
        Some(VNode::Component(VComponent {
            tag: "input".to_string(),
            attributes: vec![
                ("type".to_string(), "text".to_string()),
                ("value".to_string(), name.to_string()),
            ],
            children: vec![],
            onchange: Some(onchange.add_path("name").to_string())
        }))
    } else if let Some(transform) = r.downcast_ref::<Transform>() {
        
        Some(VNode::Component(VComponent {
            tag: "div".to_string(),
            attributes: vec![],
            children: vec![
                VNode::Text("Pos: ".to_string()),
                show_reflect(&transform.translation, onchange.add_path("translation")).unwrap(),
                VNode::Text("Rot: ".to_string()),
                show_reflect(&transform.rotation, onchange.add_path("rotation")).unwrap(),
                VNode::Text("Scale: ".to_string()),
                show_reflect(&transform.scale, onchange.add_path("scale")).unwrap(),
            ],
            onchange: None
        }))
    } else if let Some(quat) = r.downcast_ref::<Quat>() {
        let euler = quat.to_euler(EulerRot::XYZ);
        Some(VNode::Component(VComponent {
            tag: "div".to_string(),
            attributes: vec![],
            children: vec![
                VNode::Text("X: ".to_string()),
                VNode::Component(VComponent {
                    tag: "input".to_string(),
                    attributes: vec![
                        ("type".to_string(), "number".to_string()),
                        ("value".to_string(), euler.0.to_string()),
                    ],
                    children: vec![],
                    onchange: None
                }),
                VNode::Text("Y: ".to_string()),
                VNode::Component(VComponent {
                    tag: "input".to_string(),
                    attributes: vec![
                        ("type".to_string(), "number".to_string()),
                        ("value".to_string(), euler.1.to_string()),
                    ],
                    children: vec![],
                    onchange: None
                }),
                VNode::Text("Z: ".to_string()),
                VNode::Component(VComponent {
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
        Some(VNode::Component(VComponent {
            tag: "div".to_string(),
            attributes: vec![],
            children: vec![
                VNode::Text("X: ".to_string()),
                VNode::Component(VComponent {
                    tag: "input".to_string(),
                    attributes: vec![
                        ("type".to_string(), "number".to_string()),
                        ("value".to_string(), vec.x.to_string()),
                    ],
                    children: vec![],
                    onchange: None
                }),
                VNode::Text("Y: ".to_string()),
                VNode::Component(VComponent {
                    tag: "input".to_string(),
                    attributes: vec![
                        ("type".to_string(), "number".to_string()),
                        ("value".to_string(), vec.y.to_string()),
                    ],
                    children: vec![],
                    onchange: None
                }),
                VNode::Text("Z: ".to_string()),
                VNode::Component(VComponent {
                    tag: "input".to_string(),
                    attributes: vec![
                        ("type".to_string(), "number".to_string()),
                        ("value".to_string(), vec.z.to_string()),
                    ],
                    children: vec![],
                    onchange: None
                })
            ],
            onchange: None
        }))    
    
    } else {
        None
    }
}

pub fn show_reflect(r : &dyn Reflect, reflect: ReflectChange) -> Option<VNode> {

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
                Some(VNode::Component(VComponent {
                    tag: "input".to_string(),
                    attributes: vec![
                        ("type".to_string(), "text".to_string()),
                        ("value".to_string(), s.to_string()),
                    ],
                    children: vec![],
                    onchange: Some(reflect.to_string())
                }))
            } else if let Some(b) = v.downcast_ref::<bool>() {
                Some(VNode::Component(VComponent {
                    tag: "input".to_string(),
                    attributes: vec![
                        ("type".to_string(), "checkbox".to_string()),
                        ("value".to_string(), b.to_string()),
                    ],
                    children: vec![],
                    onchange: Some(reflect.to_string())
                }))
            } else if let Some(i) = v.downcast_ref::<i32>() {
                Some(VNode::Component(VComponent {
                    tag: "input".to_string(),
                    attributes: vec![
                        ("type".to_string(), "number".to_string()),
                        ("value".to_string(), i.to_string()),
                    ],
                    children: vec![],
                    onchange: Some(reflect.to_string())
                }))
            } else if let Some(f) = v.downcast_ref::<f32>() {
                Some(VNode::Component(VComponent {
                    tag: "input".to_string(),
                    attributes: vec![
                        ("type".to_string(), "number".to_string()),
                        ("value".to_string(), f.to_string()),
                    ],
                    children: vec![],
                    onchange: Some(reflect.to_string())
                }))
            } else {
                Some(VNode::Text(format!("{:?}", v)))
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
    pub onchange: Option<String> // id for onchange detection 
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
    pub entity_id : u32,
    pub generation : u32,
    pub reflect_path: String,
    pub compoent_id: String
}

impl ReflectChange {
    pub fn add_path(&self, path : &str) -> ReflectChange {
        ReflectChange {
            entity_id : self.entity_id,
            generation : self.generation,
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