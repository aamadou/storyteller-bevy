use std::sync::{Arc, Mutex, RwLock};

use bevy::{prelude::*, utils::hashbrown::HashMap, window::PrimaryWindow};
use dioxus::{core::{ElementId, Mutations}, prelude::*};
use wasm_bindgen::JsCast;
use web_sys::HtmlElement;

use crate::{ui::inspector::wasm::notify_dom_changed, wasm::{StaticBuffer, WasmMessageBuffer}};

pub struct InspectorPlugin;

impl Plugin for InspectorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, init);

        app.add_systems(Update, apply_muts);
    }
}

struct InspectorDom {
    dom: VirtualDom,
}

fn init(
    world: &mut World,
) {
    let mut dom = VirtualDom::new(inspector);

    world.insert_non_send_resource(InspectorDom { dom });

    // let muts = dom.rebuild();

    // info!("{:?}", muts);

    // let json = serde_json::to_string(&muts).unwrap();

    // q_mutation_queue.edits.push(json.clone());

    // info!("{:?}", json);

    // #[cfg(feature = "wasm")]
    // {
    //     wasm::MUTATION_BUFFER.write_message(json);
    //     notify_dom_changed(q_window);
}

fn recursive_find(
    id : &str,
    element: &web_sys::Node,
) -> Option<web_sys::Element> {
    if let Some(el) = element.as_any().downcast_ref::<web_sys::Element>() {
        if el.id() == *id {
            return Some(el.clone());
        }
    }
    let children = element.child_nodes();

    for i in 0..children.length() {
        if let Some(child) = children.get(i) {
            if let Some(element) = recursive_find(id, &child) {
                return Some(element);
            }
        }
    }

    None
}

fn apply_muts(
    mut dom : NonSendMut<InspectorDom>,
) {
    if let Some(window) = web_sys::window() {
        window.capture_events();
        window.release_events();
        info!("apply_muts 1");
        if let Some(document) = window.document() {
    
            info!("apply_muts 2");

            if let Some(mut page_root) = document.get_elements_by_name("sts-inspector").item(0) {
                let mut page_root = page_root.dyn_into::<web_sys::Element>().unwrap();
                info!("apply_muts 3");

                let dom = &mut dom.dom;
                let muts = dom.render_immediate();

                let mut element_stack = vec![];

                for m in muts.edits.iter() {
                    match m {
                        dioxus::core::Mutation::AppendChildren { id, m } => {
                            for stack_idx in 0..*m {
                                let (id, template) = element_stack.pop().unwrap();
                                recursive_add_dom(
                                    &id,
                                    &template,
                                    &mut page_root,
                                    &document,
                                );
                            }
                        },
                        dioxus::core::Mutation::AssignId { path, id } => todo!(),
                        dioxus::core::Mutation::CreatePlaceholder { id } => todo!(),
                        dioxus::core::Mutation::CreateTextNode { value, id } => todo!(),
                        dioxus::core::Mutation::HydrateText { path, value, id } => todo!(),
                        dioxus::core::Mutation::LoadTemplate { name, index, id } => {
                            let mut template = None;
                            for t in muts.templates.iter() {
                                if t.name == *name {
                                    template = Some(t);
                                    break;
                                }
                            }

                            if let Some(template) = template {
                                element_stack.push((id.clone(), template.roots[*index].clone()));
                            }
                        },
                        dioxus::core::Mutation::ReplaceWith { id, m } => todo!(),
                        dioxus::core::Mutation::ReplacePlaceholder { path, m } => todo!(),
                        dioxus::core::Mutation::InsertAfter { id, m } => todo!(),
                        dioxus::core::Mutation::InsertBefore { id, m } => todo!(),
                        dioxus::core::Mutation::SetAttribute { name, value, id, ns } => todo!(),
                        dioxus::core::Mutation::SetText { value, id } => todo!(),
                        dioxus::core::Mutation::NewEventListener { name, id } => todo!(),
                        dioxus::core::Mutation::RemoveEventListener { name, id } => todo!(),
                        dioxus::core::Mutation::Remove { id } => todo!(),
                        dioxus::core::Mutation::PushRoot { id } => todo!(),
                    }
                }
            }
        }
    }
}

fn recursive_add_dom(
    element_id: &ElementId,
    template: &TemplateNode,
    parent: &mut web_sys::Element,
    document: &web_sys::Document,
) {
    match *template {
        TemplateNode::Element { tag, namespace, attrs, children } => {
            let mut element = document.create_element(tag).unwrap();
            element.set_attribute("id", format!("element_{}", element_id.0).as_str()).unwrap();

            for attr in attrs {
                match *attr {
                    TemplateAttribute::Static { name, value, namespace } => {
                        element.set_attribute(name, value).unwrap();
                    },
                    TemplateAttribute::Dynamic { id } => {
                        // TODO
                    },
                }
            }

            parent.append_child(&element).unwrap();

            for child in children {
                recursive_add_dom(element_id, child, &mut element, document);
            }
        },
        TemplateNode::Text { text } => {
            let text_node = document.create_element("p").unwrap();
            text_node.set_text_content(Some(text));
            parent.append_child(&text_node).unwrap();
        },
        TemplateNode::Dynamic { id } => {
            todo!()
        },
        TemplateNode::DynamicText { id } => {
            todo!()
        },
    }
}

fn inspector(
    cx: Scope
) -> Element {

    cx.render(rsx!{
        div {
            "Inspector",
            div {
                "hello world"
            }
        }
    })
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
        q_window: Query<&Window, With<PrimaryWindow>>,
    ) -> Result<(), String> {
        
        wasm::dispatch_to_js(q_window.single(), "inspector-dom-changed", &Option::<Entity>::None)
    }

    pub static MUTATION_BUFFER: WasmMessageBuffer<String> = WasmMessageBuffer::new();

    #[wasm_bindgen(js_name = getMutation)]
    pub fn get_mutation() -> String {


        MUTATION_BUFFER.take_message().unwrap_or("{}".to_string())
    }
}