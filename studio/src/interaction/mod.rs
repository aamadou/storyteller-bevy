use bevy::prelude::*;
use bitflags::bitflags;
use wasm_bindgen::prelude::*;

mod gizmos;

pub struct InteractionPlugin;

impl Plugin for InteractionPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SelectionMode::default())
            .insert_resource(InteractionMode::default())
            .add_systems(Update, gizmos::draw);

        app.add_systems(
            PreUpdate,
            (wasm::update_selection_mode, wasm::update_interaction_mode),
        );
    }
}

#[derive(Component, Clone, Copy)]
pub struct Selectable(SelectionChannel);

#[derive(Clone, Copy)]
#[wasm_bindgen]
pub enum SelectionChannel {
    Objects,
    Bones,
    Widgets,
}

impl Default for SelectionChannel {
    fn default() -> Self {
        Self::Objects
    }
}

#[derive(Component, Clone, Copy)]
pub struct Selected;

#[derive(Resource, Default)]
pub struct SelectionMode(SelectionChannel);

#[derive(Resource)]
pub struct InteractionMode {
    pub transform: InteractionTransform,
    pub space: TransformSpace,
}

impl Default for InteractionMode {
    fn default() -> Self {
        Self {
            transform: InteractionTransform::TRANSLATE | InteractionTransform::ROTATE,
            space: default(),
        }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
#[wasm_bindgen]
pub struct InteractionTransform(u8);

bitflags! {
    impl InteractionTransform: u8 {
        const TRANSLATE = 0b0001;
        const ROTATE    = 0b0010;
    }
}

#[derive(Clone, Copy, Debug)]
#[wasm_bindgen]
pub enum TransformSpace {
    Local,
    World,
}

impl Default for TransformSpace {
    fn default() -> Self {
        Self::Local
    }
}

// #[cfg(target_arch = "wasm32")]
pub mod wasm {
    use bevy::prelude::*;
    use wasm_bindgen::prelude::*;

    use crate::wasm::{self, WasmMessageBuffer};

    use super::{
        InteractionMode, InteractionTransform, SelectionChannel, SelectionMode, TransformSpace,
    };

    static SELECT_MODE_BUF: WasmMessageBuffer<SelectionChannel> = WasmMessageBuffer::new();
    static INTERACTION_TRANSFORM_BUF: WasmMessageBuffer<InteractionTransform> =
        WasmMessageBuffer::new();
    static INTERACTION_SPACE_BUF: WasmMessageBuffer<TransformSpace> = WasmMessageBuffer::new();

    #[wasm_bindgen(js_name = setSelectionMode)]
    pub fn set_selection_mode(mode: SelectionChannel) {
        wasm::write_message!(SELECT_MODE_BUF, mode);
    }

    #[wasm_bindgen(js_name = setInteractionTransform)]
    pub fn set_interaction_transform(flags: u8) {
        wasm::write_message!(INTERACTION_TRANSFORM_BUF, InteractionTransform(flags));
    }

    #[wasm_bindgen(js_name = setTransformSpace)]
    pub fn set_transform_space(space: TransformSpace) {
        wasm::write_message!(INTERACTION_SPACE_BUF, space);
    }

    pub(super) fn update_selection_mode(mut selection_mode: ResMut<SelectionMode>) {
        wasm::read_buffer!(SELECT_MODE_BUF, mode => {
            selection_mode.0 = mode;
        });
    }

    pub(super) fn update_interaction_mode(mut interaction_mode: ResMut<InteractionMode>) {
        wasm::read_buffer!(INTERACTION_TRANSFORM_BUF, transform => {
            interaction_mode.transform = transform;
        });
        wasm::read_buffer!(INTERACTION_SPACE_BUF, space => {
            interaction_mode.space = space;
        });
    }
}
