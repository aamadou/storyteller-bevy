use bevy::{prelude::*, window::WindowResolution};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::{InitAsset, InitParams, StudioMode, ViewportSize};

#[wasm_bindgen(typescript_custom_section)]
const WASM_ENTRY_POINTS: &str = r#"
export interface InitOptions {
    mode: StudioMode;
    /** Required for all entry-points except `initFromStorytellerScene()` */
    skyboxId?: string;
    canvasSelector: string;
    canvasAlt: string;
    viewportSize: ViewportSize;
    scaleFactor: number;
}

export function initFromLibrary(objectId: string, options: InitOptions): void;
export function initFromBvh(animPath: string, options: InitOptions): void;
export function initFromMixamo(animPath: string, options: InitOptions): void;
export function initFromSceneImport(scenePath: string, options: InitOptions): void;
export function initFromStorytellerScene(scenePath: string, options: InitOptions): void;
"#;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct WasmInitOptions {
    skybox_id: Option<String>,
    mode: StudioMode,
    canvas_selector: String,
    canvas_alt: String,
    viewport_size: ViewportSize,
    scale_factor: f32,
}

#[wasm_bindgen(skip_typescript, js_name = initFromLibrary)]
pub fn init_from_library(object_id: String, options: JsValue) -> Result<(), JsValue> {
    let options = serde_wasm_bindgen::from_value::<WasmInitOptions>(options)?;
    info!("wasm::init_from_library: {object_id} {options:#?}");

    let skybox_id = options
        .skybox_id
        .expect("`skyboxId` is required for `initFromLibrary`");

    let ViewportSize { width, height } = options.viewport_size;

    crate::start(
        InitParams::FromAsset {
            asset: InitAsset::LibraryObject(object_id),
            skybox_id,
        },
        options.mode,
        Some(Window {
            resolution: WindowResolution::new(width, height)
                .with_scale_factor_override(options.scale_factor),
            title: options.canvas_alt,
            canvas: Some(options.canvas_selector),
            ..default()
        }),
    )
    .run();

    Ok(())
}

#[wasm_bindgen(skip_typescript, js_name = initFromBvh)]
pub fn init_from_bvh(anim_path: String, options: JsValue) -> Result<(), JsValue> {
    let options = serde_wasm_bindgen::from_value::<WasmInitOptions>(options)?;
    info!("wasm::init_from_bvh: {anim_path} {options:#?}");

    let skybox_id = options
        .skybox_id
        .expect("`skyboxId` is required for `initFromBvh`");

    let ViewportSize { width, height } = options.viewport_size;

    crate::start(
        InitParams::FromAsset {
            asset: InitAsset::BvhAnim(anim_path),
            skybox_id,
        },
        options.mode,
        Some(Window {
            resolution: WindowResolution::new(width, height)
                .with_scale_factor_override(options.scale_factor),
            title: options.canvas_alt,
            canvas: Some(options.canvas_selector),
            ..default()
        }),
    )
    .run();

    Ok(())
}

#[wasm_bindgen(skip_typescript, js_name = initFromMixamo)]
pub fn init_from_mixamo(anim_path: String, options: JsValue) -> Result<(), JsValue> {
    let options = serde_wasm_bindgen::from_value::<WasmInitOptions>(options)?;
    info!("wasm::init_from_mixamo: {anim_path} {options:#?}");

    let skybox_id = options
        .skybox_id
        .expect("`skyboxId` is required for `initFromMixamo`");

    let ViewportSize { width, height } = options.viewport_size;

    crate::start(
        InitParams::FromAsset {
            asset: InitAsset::MixamoAnim(anim_path),
            skybox_id,
        },
        options.mode,
        Some(Window {
            resolution: WindowResolution::new(width, height)
                .with_scale_factor_override(options.scale_factor),
            title: options.canvas_alt,
            canvas: Some(options.canvas_selector),
            ..default()
        }),
    )
    .run();

    Ok(())
}

#[wasm_bindgen(skip_typescript, js_name = initFromSceneImport)]
pub fn init_from_scene_import(scene_path: String, options: JsValue) -> Result<(), JsValue> {
    let options = serde_wasm_bindgen::from_value::<WasmInitOptions>(options)?;
    info!("wasm::init_from_scene_import: {scene_path} {options:#?}");

    let skybox_id = options
        .skybox_id
        .expect("`skyboxId` is required for `initFromSceneImport`");

    let ViewportSize { width, height } = options.viewport_size;

    crate::start(
        InitParams::FromAsset {
            asset: InitAsset::ImportedScene(scene_path),
            skybox_id,
        },
        options.mode,
        Some(Window {
            resolution: WindowResolution::new(width, height)
                .with_scale_factor_override(options.scale_factor),
            title: options.canvas_alt,
            canvas: Some(options.canvas_selector),
            ..default()
        }),
    )
    .run();

    Ok(())
}

#[wasm_bindgen(skip_typescript, js_name = initFromStorytellerScene)]
pub fn init_from_storyteller_scene(scene_path: String, options: JsValue) -> Result<(), JsValue> {
    let options = serde_wasm_bindgen::from_value::<WasmInitOptions>(options)?;
    info!("wasm::init_from_storyteller_scene: {scene_path} {options:#?}");

    let ViewportSize { width, height } = options.viewport_size;

    crate::start(
        InitParams::FromScene(scene_path),
        options.mode,
        Some(Window {
            resolution: WindowResolution::new(width, height)
                .with_scale_factor_override(options.scale_factor),
            title: options.canvas_alt,
            canvas: Some(options.canvas_selector),
            ..default()
        }),
    )
    .run();

    Ok(())
}
