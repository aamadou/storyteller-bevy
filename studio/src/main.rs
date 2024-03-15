use bevy::{
    app::Update, ecs::{query::With, system::Query}, window::{close_on_esc, Window}, prelude::*
};
use clap::Parser;
use studio::{anim::Skeleton, scene::{tag_scene_elements, SceneState}, screenshot::ScreenShotPlugin, InitAsset, InitParams, StudioMode};

#[derive(Parser)]
#[command(name = "StorytellerStudio")]
#[command(author = "The Storyteller Team")]
#[command(version = "1.0")]
#[command(about = "Executable for Storyteller Studio.")]
#[command(propagate_version = true)]
struct StudioArgs {
    model: Option<String>,
    skybox: Option<String>,
}

fn main() {
    let args = StudioArgs::parse();

    let object_id = args.model.unwrap_or("Mannequin.gltf".to_string());
    let skybox_id = args.skybox.unwrap_or("gum_trees_4k".to_string());

    // Edit the arguments here to change the default startup scene or skybox
    let mut app = studio::start(
        InitParams::FromAsset {
            asset: InitAsset::LibraryObject(object_id),
            skybox_id,
        },
        StudioMode::Editor,
        Some(Window {
            title: "Storyteller Studio".into(),
            ..Default::default()
        }),
    );

    // Add any additional systems, resources, etc. for local debugging
    app.add_systems(Update, close_on_esc);
    app.add_systems(Update, add_anim);
    app.add_plugins(ScreenShotPlugin);

    app.run();
}

fn add_anim(
    mut commands : Commands,
    skeletons : Query<Entity, Added<Skeleton>>,
    asset_server : Res<AssetServer>,
) {
    let bvh_anim : Handle<AnimationClip> = asset_server.load("mocap/shuffle.bvh");
    
    if skeletons.iter().count() > 0 {
        println!("Adding animation to skeletons {}", skeletons.iter().count());
    }
    
    for skeleton in skeletons.iter() {
        let mut animation_player = AnimationPlayer::default();
        animation_player.play(bvh_anim.clone());
        println!("{:?}", skeleton);
        commands.entity(skeleton).insert((
            animation_player,
            bvh_anim.clone()
        ));
    }
}