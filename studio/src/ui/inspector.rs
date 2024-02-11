use std::{sync::{Arc, Mutex, RwLock}, thread::yield_now};

use bevy::{ecs::world::unsafe_world_cell::{UnsafeEntityCell, UnsafeWorldCell}, prelude::*, reflect::TypeData, utils::{hashbrown::HashMap, tracing::field}, window::PrimaryWindow};
use serde::{Deserialize, Serialize};
use space_editor::{prelude::Selected, space_editor_ui::inspector::inspect};
use wasm_bindgen::JsCast;
use web_sys::HtmlElement;

use crate::{ui::inspector::wasm::notify_dom_changed, wasm::{StaticBuffer, WasmMessageBuffer}};

use super::utils::{show_reflect, ComponentProp, InspectedEntity, VComponent, VNode};

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

        let vnode = VNode::Component(VComponent {
            tag: "div".to_string(),
            attributes: vec![
                
            ],
            children: cell.components.into_iter().map(|c| show_reflect(c.as_ref())).filter_map(|c| c).collect(),
            ..Default::default()
        });
        

        let q_window = world.query_filtered::<&Window, With<PrimaryWindow>>().single(world);

        let _ = notify_dom_changed(q_window, serde_json::to_string(&vnode).unwrap());

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
