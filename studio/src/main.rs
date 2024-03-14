use bevy::{
    app::Update, transform::components::Transform, window::{close_on_esc, Window}, prelude::*
};
use clap::Parser;
use studio::{scene::SceneState, screenshot::ScreenShotPlugin, InitAsset, InitParams, StudioMode };

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

    let object_id = args.model.unwrap_or("sample-room.gltf".to_string());
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
    // app.add_plugins(ScreenShotPlugin);
    
    app.insert_resource(CameraMotion::Zoom);
    app.add_systems(Update, update_camera_motion.run_if(in_state(SceneState::Active)));

    app.run();
}

//TODO: Remove this test code

pub const SPEED : f32 = 0.8;

#[derive(Resource)]
enum CameraMotion {
    Orbit,
    Pan,
    Zoom,
    Static
}

pub(crate) fn update_camera_motion(
    mut camera: Query<&mut Transform, (With<Camera>, Without<Parent>)>,
    camera_motion: Res<CameraMotion>,
    time: Res<Time>,
) {
    println!("{}", camera.iter().count());
    let Ok(mut main_cam_transform) = camera.get_single_mut() else {
        return;
    };
    
    println!("Updating camera motion");
    
    match *camera_motion {
        CameraMotion::Orbit => {
            orbit_camera(&mut main_cam_transform, &time);
        }
        CameraMotion::Pan => {
            pan_camera(&mut main_cam_transform, &time);
        }
        CameraMotion::Zoom => {
            zoom_camera(&mut main_cam_transform, &time);
        }
        CameraMotion::Static => {}
    }
    // main_cam_transform.look_at(Vec3::ZERO, Vec3::Y);
}

fn zoom_camera(main_cam_transform: &mut Transform, time: &Time<()>) {
    let scaled_time = time.elapsed().as_secs_f32() * SPEED;
    let normalized_cos = (-scaled_time.cos() + 1.0) / 2.0;
    let start_pos = Vec3::new(0.366, 1.1, 2.5);
    let end_pos = Vec3::new(0.366, 1.1, -0.5);
    
    main_cam_transform.translation = start_pos.lerp(end_pos, normalized_cos);
    main_cam_transform.rotation = Quat::from_euler(EulerRot::XYZ, 0.0, 0.0, 0.0);
}

fn pan_camera(main_cam_transform: &mut Transform, time: &Time<()>) {
    let scaled_time = time.elapsed().as_secs_f32() * SPEED;
    let normalized_cos = (scaled_time.cos() + 1.0) / 2.0;
    let start_pos = Vec3::new(1.617, 0.9, 2.177);
    let end_pos = Vec3::new(-0.365, 0.9, 2.323);
    
    main_cam_transform.translation = start_pos.lerp(end_pos, normalized_cos);
    main_cam_transform.rotation = Quat::from_euler(EulerRot::XYZ, 0.0, 0.0, 0.0);
}

fn orbit_camera(main_cam_transform: &mut Mut<'_, Transform>, time: &Res<'_, Time>) {
    println!("Orbiting camera");
    let scaled_time = time.elapsed().as_secs_f32() * SPEED;
    let max_angle = std::f32::consts::PI / 2.0;
    let normalized_cos = (scaled_time.cos() + 1.0) / 2.0;
    let angle = max_angle * normalized_cos;
    
    // let t = (time.elapsed().as_secs_f32() / 5.0) % 1.0;
    
    main_cam_transform.translation = Vec3::new(-2.2 * angle.cos(), 1.0, 2.5 * angle.sin());
    main_cam_transform.look_at(Vec3::new(0.347, 0.556, -1.0), Vec3::Y);
}