
use bevy::prelude::*;
use dioxus::prelude::*;

#[derive(Props)]
pub struct InspectedEntity {
    pub id: Option<Entity>,
    pub components: Vec<Box<dyn Reflect>>,
}

#[derive(Props)]
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