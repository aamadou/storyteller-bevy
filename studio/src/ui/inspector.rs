use bevy::{ecs::world::unsafe_world_cell::UnsafeEntityCell, prelude::*, window::PrimaryWindow};
use space_editor::space_prefab::component::MeshPrimitivePrefab;

use crate::{ui::inspector::wasm::notify_dom_changed, wasm::StaticBuffer};

use super::utils::{blocking_component, show_reflect, Element, InspectedEntity, ReflectChange, VirtualDom};

pub struct InspectorPlugin;

impl Plugin for InspectorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InspectorFunctions>();

        app.add_systems(Update, show_selected);

        app.world
            .resource_mut::<InspectorFunctions>()
            .add_simple::<Name>();
        app.world
            .resource_mut::<InspectorFunctions>()
            .add_simple::<Transform>();

        app.world
            .resource_mut::<InspectorFunctions>()
            .add_simple::<MeshPrimitivePrefab>();
    }
}

#[derive(Resource, Default)]
struct InspectorFunctions {
    get_reflects:
        Vec<Box<dyn FnMut(&UnsafeEntityCell<'_>) -> Option<Box<dyn Reflect>> + Send + Sync>>,
}

impl InspectorFunctions {
    fn add_getter<
        F: FnMut(&UnsafeEntityCell<'_>) -> Option<Box<dyn Reflect>> + Send + Sync + 'static,
    >(
        &mut self,
        f: F,
    ) {
        self.get_reflects.push(Box::new(f));
    }

    fn add_simple<T>(&mut self)
    where
        T: bevy::prelude::Component + Reflect + FromReflect + 'static,
    {
        unsafe {
            self.add_getter(|cell| {
                if let Some(val) = cell.get::<T>() {
                    Some(Box::new(<T as FromReflect>::from_reflect(val).unwrap())
                        as Box<dyn Reflect>)
                } else {
                    None
                }
            })
        }
    }

    fn get_reflect_copy(&mut self, cell: &UnsafeEntityCell<'_>) -> Vec<Box<dyn Reflect>> {
        self.get_reflects
            .iter_mut()
            .filter_map(|f| (*f)(cell))
            .collect()
    }
}

fn show_selected(world: &mut World) {
    if !world.is_resource_added::<VirtualDom>() {
        world.init_resource::<VirtualDom>();
    }
    unsafe {
        let cell = world.as_unsafe_world_cell();
        let mut virtial_dom = cell.get_resource_mut::<VirtualDom>().unwrap();

        //Process events from real DOM
        if let Some(changes) = super::utils::wasm::ONCHAGE_BUFFER.take_message() {
            for (id, kind, value) in changes {
                info!("change: {} {} {}", id, kind, value);
                if kind == "onchange" {
                    info!("change: {} : {}", id, value);
                    virtial_dom
                        .on_change_events
                        .execute(cell.world_mut(), id, value);
                } else if kind == "onclick" {
                    info!("click: {} : {}", id, value);
                    virtial_dom
                        .on_click_events
                        .execute(cell.world_mut(), id, value);
                }
            }
        }

        let blocking_element = blocking_component();

        let Some(selection) = cell.get_resource::<crate::interaction::Selection>() else {
            let muts = virtial_dom.render(blocking_element);
            let q_window = cell
                .world_mut()
                .query_filtered::<&Window, With<PrimaryWindow>>()
                .single(cell.world());
            let _ = notify_dom_changed(q_window, serde_json::to_string(&muts).unwrap());

            return;
        };

        let selected_entity = selection.0;

        let mut functions = cell.get_resource_mut::<InspectorFunctions>().unwrap();

        let _app_type_registry = cell.get_resource::<AppTypeRegistry>().unwrap();
        let e = cell.get_entity(selected_entity).unwrap();

        

        let mut inspected = InspectedEntity::none();
        inspected.id = Some(e.id());
        inspected.components = functions.get_reflect_copy(&e);

        let id = e.id();

        let components = cell.components();
        let element = Element::Component(super::utils::Component {
            tag: "div".to_string(),
            attributes: vec![
                (
                    "style".to_string(),
                    "display: flex; flex-direction: column; height: 100%; overflow: auto;"
                        .to_string(),
                ),
                ("class".to_string(), "inspector".to_string()),
            ],
            children: inspected
                .components
                .into_iter()
                .filter_map(|c| {
                    show_reflect(
                        c.as_ref(),
                        ReflectChange {
                            entity_id: id,
                            reflect_path: "".to_string(),
                            compoent_id: format!(
                                "{:?}",
                                components
                                    .get_id(c.get_represented_type_info().unwrap().type_id())
                                    .unwrap()
                            ),
                        },
                    )
                })
                .collect(),
            ..Default::default()
        });

        let super_root = Element::Component(super::utils::Component {
            tag: "div".to_string(),
            attributes: vec![
                ("style".to_string(), "height: 100%; display: flex; flex-direction: column".to_string()),
            ],
            children: vec![blocking_element.unwrap(), element],
            ..default()
        });


        let muts = virtial_dom.render(Some(super_root));

        let q_window = cell
            .world_mut()
            .query_filtered::<&Window, With<PrimaryWindow>>()
            .single(cell.world());

        let _ = notify_dom_changed(q_window, serde_json::to_string(&muts).unwrap());
    }
}

#[cfg(feature = "wasm")]
mod wasm {
    use crate::wasm::*;
    use bevy::prelude::*;

    use wasm_bindgen::prelude::*;

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

    pub(super) fn notify_dom_changed(q_window: &Window, msg: String) -> Result<(), String> {
        wasm::dispatch_to_js(q_window, "inspector-dom-changed", &msg)
    }
}
