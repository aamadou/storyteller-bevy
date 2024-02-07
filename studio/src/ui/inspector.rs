use std::sync::{Arc, Mutex, RwLock};

use bevy::{ecs::world::unsafe_world_cell::{UnsafeEntityCell, UnsafeWorldCell}, prelude::*, reflect::TypeData, utils::{hashbrown::HashMap, tracing::field}, window::PrimaryWindow};
use dioxus::{prelude::*, rsx::*};
use serde::{Deserialize, Serialize};
use space_editor::{prelude::Selected, space_editor_ui::inspector::inspect};
use wasm_bindgen::JsCast;
use web_sys::HtmlElement;

use crate::{ui::inspector::wasm::notify_dom_changed, wasm::{StaticBuffer, WasmMessageBuffer}};

use super::utils::{ComponentProp, InspectedEntity};

pub struct InspectorPlugin;

impl Plugin for InspectorPlugin {
    fn build(&self, app: &mut App) {

        app.init_resource::<InspectorFunctions>();

        app.add_systems(Update, show_selected);

        app.world.resource_mut::<InspectorFunctions>().add_simple::<Name>();
        app.world.resource_mut::<InspectorFunctions>().add_simple::<Transform>();
    }
}

#[derive(Resource, Default)]
struct InspectorFunctions {
    get_reflects: Vec<Box<dyn FnMut(&UnsafeEntityCell<'_>) -> Option<Box<dyn Reflect>> + Send + Sync>>
}

impl InspectorFunctions {
    fn add_getter<F: FnMut(&UnsafeEntityCell<'_>) -> Option<Box<dyn Reflect>> + Send + Sync + 'static>(&mut self, f: F) {
        self.get_reflects.push(Box::new(f));
    }

    fn add_simple<T>(&mut self) where T: bevy::prelude::Component + Reflect + FromReflect + 'static {

        unsafe {
            self.add_getter(
                |cell| {
                    if let Some(val) = cell.get::<T>() {
                        Some(Box::new(<T as FromReflect>::from_reflect(val).unwrap()) as Box<dyn Reflect>)
                    } else {
                        None
                    }
                }
            )
        }
    }

    fn get_reflect_copy(&mut self, cell: &UnsafeEntityCell<'_>) -> Vec<Box<dyn Reflect>> {
        self.get_reflects.iter_mut().map(|f| (*f)(cell)).flatten().collect()
    }
}


fn web_inspector(cx: Scope<InspectedEntity>) -> dioxus::core::Element {

    let e = cx.props;

    let components = e.components.iter().map(|c: &Box<dyn Reflect>| ComponentProp { component: c.clone_value() }).collect::<Vec<_>>();
    let components_rendered = components.iter().map(|c : &ComponentProp| {
        rsx! {
            web_reflect { component: c.component.clone_value() }
        }
    });

    if e.id.is_none() {
        render! {
            div {
                "No entity selected"
            }
        }
    } else {
        let id_name = format!("{:?}", e.id.unwrap());
        render! {
            div {
                id_name,
                components_rendered
            }
        }
    }
}

fn web_reflect(cx: Scope<ComponentProp>) -> dioxus::core::Element {

    //registered components
    if cx.props.component.represents::<Name>() {
        let name = <Name as FromReflect>::from_reflect(cx.props.component.as_ref()).unwrap();
        return render! {
            div {
                p { name.as_str() }
            }
        };
    }else if cx.props.component.represents::<Transform>() {
        let transform = <Transform as FromReflect>::from_reflect(cx.props.component.as_ref()).unwrap();
        let euler = transform.rotation.to_euler(EulerRot::XYZ);
        return render! {
            div {
                "style" : "flex-direction: column;  display: flex; ",
                "Transform\n",
                format!("pos: {:.2} {:.2} {:.2}\n", transform.translation.x, transform.translation.y, transform.translation.z),
                format!("rot: {:.2} {:.2} {:.2}\n", euler.0, euler.1, euler.2),
                format!("scale: {:.2} {:.2} {:.2}\n", transform.scale.x, transform.scale.y, transform.scale.z)
            }
        };
    }

    let res = match cx.props.component.reflect_ref() {
        
        bevy::reflect::ReflectRef::Struct(v) => {
            let field_count = v.field_len();
            
            render! {
                div {
                    p { v.type_name().split("::").last().unwrap() },
                    (0..field_count).map(|i| {
                        let field = v.field_at(i).unwrap();
                        render! {
                            format!("{}: ", v.name_at(i).unwrap()),
                            web_reflect { component: field.clone_value() } 
                            
                        }
                    })
                    
                }
            }
        },
        bevy::reflect::ReflectRef::TupleStruct(v) => {
            let field_count = v.field_len();
            
            render! {
                div {
                    p { v.type_name().split("::").last().unwrap(), },
                    v.iter_fields().map(|field| {
                        render! {
                            web_reflect { component: field.clone_value() } 
                        }
                    })
                }
            }
        },
        bevy::reflect::ReflectRef::Tuple(_) => {
            render! {
                div {
                    
                }
            }
        },
        bevy::reflect::ReflectRef::List(_) => {
            render! {
                div {

                }
            }
        },
        bevy::reflect::ReflectRef::Array(_) => {
            render! {
                div {
                    
                }
            }
        },
        bevy::reflect::ReflectRef::Map(_) => {
            render! {
                div {
                    
                }
            }
        },
        bevy::reflect::ReflectRef::Enum(_) => {
            render! {
                div {
                    
                }
            }
        },
        bevy::reflect::ReflectRef::Value(v) => {
            if let Some(v) = v.downcast_ref::<String>() {
                render! {
                    div {
                        format!("{}", v)
                    }
                }
            } else if let Some(v) = v.downcast_ref::<f32>() {
                render! {
                    div {
                        format!("{:?}", v)
                    }
                }
            } else if let Some(v) = v.downcast_ref::<f64>() {
                render! {
                    div {
                        format!("{:?}", v)
                    }
                }
            } else if let Some(v) = v.downcast_ref::<bool>() {
                render! {
                    div {
                        format!("{:?}", v)
                    }
                }
            } else if let Some(v) = v.downcast_ref::<i32>() {
                render! {
                    div {
                        format!("{:?}", v)
                    }
                }
            } else if let Some(v) = v.downcast_ref::<i64>() {
                render! {
                    div {
                        format!("{:?}", v)
                    }
                }
            } else if let Some(v) = v.downcast_ref::<u32>() {
                render! {
                    div {
                        format!("{:?}", v)
                    }
                }
            } else if let Some(v) = v.downcast_ref::<u64>() {
                render! {
                    div {
                        format!("{:?}", v)
                    }
                }
            } else if let Some(v) = v.downcast_ref::<u8>() {
                render! {
                    div {
                        format!("{:?}", v)
                    }
                }
            } else if let Some(v) = v.downcast_ref::<u16>() {
                render! {
                    div {
                        format!("{:?}", v)
                    }
                }
            } else if let Some(v) = v.downcast_ref::<i8>() {
                render! {
                    div {
                        format!("{:?}", v)
                    }
                }
            } else if let Some(v) = v.downcast_ref::<i16>() {
                render! {
                    div {
                        format!("{:?}", v)
                    }
                }
            } else {
                render! {
                    div {
                       "Not implemented reflect type" 
                    }
                }
            }
        },
    };
    return res;
}


fn show_selected(
    world: &mut World
) {
    unsafe {
        let cell = world.as_unsafe_world_cell();
        
        let Some(selection) = cell.get_resource::<crate::interaction::Selection>() else {
            return;
        };

        let selected_entity = selection.0;

        let mut functions = cell.get_resource_mut::<InspectorFunctions>().unwrap();

        let e = cell.get_entity(selected_entity).unwrap();

        let mut cell = InspectedEntity::none();
        cell.id = Some(e.id());
        cell.components = functions.get_reflect_copy(&e);
        

        let mut dom = VirtualDom::new_with_props(web_inspector, cell);
        
        let muts = dom.rebuild();

        let q_window = world.query_filtered::<&Window, With<PrimaryWindow>>().single(world);

        let _ = notify_dom_changed(q_window, serde_json::to_string(&muts).unwrap());

    }
}



#[cfg(feature = "wasm")]
mod wasm {
    use bevy::{prelude::*, window::PrimaryWindow};
    use serde_json::json;
    use wasm_bindgen::prelude::*;
    use crate::wasm::*;

    use super::*;

    #[wasm_bindgen(typescript_custom_section)]
    const EVENT_DECLARATION: &str = r#"
/**
 * An event indicating that an entity has been selected or that the selection
 * has been cleared.
 */
export interface InspectorDomChanged extends CustomEvent {
    /** The selected entity ID. If `null`, the selection has been cleared. */
    detail: string | null;
}

declare global {
    export interface HTMLElementEventMap {
        "inspector-dom-changed": InspectorDomChanged;
    }
}
    "#;

    pub(super) fn notify_dom_changed(
        q_window: &Window,
        msg : String,
    ) -> Result<(), String> {
        
        wasm::dispatch_to_js(q_window, "inspector-dom-changed", &msg)
    }

    pub static MUTATION_BUFFER: WasmMessageBuffer<String> = WasmMessageBuffer::new();

    #[wasm_bindgen(js_name = getMutation)]
    pub fn get_mutation() -> String {


        MUTATION_BUFFER.take_message().unwrap_or("{}".to_string())
    }
}
