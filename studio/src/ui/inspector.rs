use std::{any::Any, sync::{Arc, Mutex, RwLock}, thread::yield_now};

use bevy::{ecs::{component::ComponentId, world::unsafe_world_cell::{UnsafeEntityCell, UnsafeWorldCell}}, prelude::*, reflect::{ReflectFromPtr, TypeData}, utils::{hashbrown::HashMap, tracing::field}, window::PrimaryWindow};
use serde::{Deserialize, Serialize};
use space_editor::{prelude::Selected, space_editor_ui::inspector::inspect};
use wasm_bindgen::JsCast;
use web_sys::HtmlElement;

use crate::{ui::inspector::wasm::notify_dom_changed, wasm::{StaticBuffer, WasmMessageBuffer}};

use super::utils::{show_reflect, ComponentProp, InspectedEntity, ReflectChange, VComponent, VNode, VirtualDom};

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


fn show_selected(
    world: &mut World
) {
    if !world.is_resource_added::<VirtualDom>() {
        world.init_resource::<VirtualDom>();
    }
    unsafe {
        let cell = world.as_unsafe_world_cell();
        
        let Some(selection) = cell.get_resource::<crate::interaction::Selection>() else {
            return;
        };

        let selected_entity = selection.0;

        let mut functions = cell.get_resource_mut::<InspectorFunctions>().unwrap();

        let mut virtial_dom = cell.get_resource_mut::<VirtualDom>().unwrap();
        let app_type_registry = cell.get_resource::<AppTypeRegistry>().unwrap();
        let app_type_registry = app_type_registry.read();

        let e = cell.get_entity(selected_entity).unwrap();

        if let Some(changes) = super::utils::wasm::ONCHAGE_BUFFER.take_message() {
            for (change_id, value) in changes {
                info!("change: {} : {}", change_id, value);

                if let Ok(change) = serde_json::from_str::<ReflectChange>(&change_id) {
                    let c_id = ComponentId::new(
                        change.compoent_id.split('(').nth(1).unwrap().split(')').next().unwrap().parse().unwrap()
                    );

                    if let Some(mut c) = e.get_mut_by_id(c_id) {

                        let mut refl = app_type_registry.get_type_data::<ReflectFromPtr>(cell.components().get_info_unchecked(c_id).type_id().unwrap())
                            .unwrap()
                            .as_reflect_mut(c.as_mut());

                        if let Ok(refl_value) = refl.reflect_path_mut(change.reflect_path.as_str()) {
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


        let mut inspected = InspectedEntity::none();
        inspected.id = Some(e.id());
        inspected.components = functions.get_reflect_copy(&e);

        let id = e.id();

        let components = cell.components().clone();
        let vnode = VNode::Component(VComponent {
            tag: "div".to_string(),
            attributes: vec![

            ],
            children: inspected.components.into_iter().map(|c| {
                    show_reflect(
                        c.as_ref(), 
                        ReflectChange {
                            entity_id: id.index(),
                            generation: id.generation(),
                            reflect_path: "".to_string(),
                            compoent_id: format!("{:?}", components.get_id(c.get_represented_type_info().unwrap().type_id()).unwrap()),
                        }
                    )
                })
                .filter_map(|v| v)
                .collect(),
            ..Default::default()
        });

        let muts = virtial_dom.create_mutations(Some(vnode));
        

        let q_window = cell.world_mut().query_filtered::<&Window, With<PrimaryWindow>>().single(cell.world());

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

    
}
