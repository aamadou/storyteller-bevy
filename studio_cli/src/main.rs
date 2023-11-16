use bevy::{
    asset::{AssetPath, HandleId},
    core_pipeline::{core_3d::Camera3dDepthTextureUsage, fxaa},
    prelude::*,
    utils::HashSet,
};
use studio::{DebugMode, DebugPlugin, Inspectable, MainCamera, SkyboxPlugin};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, SkyboxPlugin, DebugPlugin))
        .insert_resource(DebugMode(true))
        .register_type::<Camera3dDepthTextureUsage>()
        .register_type::<Inspectable>()
        .register_type::<MainCamera>()
        .register_type::<fxaa::Sensitivity>()
        .register_type::<Vec<Vec<Option<Entity>>>>()
        .register_type::<Vec<Option<Entity>>>()
        .add_systems(Startup, load_scene)
        .add_systems(Update, log_asset_paths)
        .run();
}

fn load_scene(mut cmd: Commands, r_assets: Res<AssetServer>) {
    cmd.spawn(DynamicSceneBundle {
        scene: r_assets.load("scene.scn.ron"),
        ..default()
    });
}

fn log_asset_paths(
    mut l_logged: Local<HashSet<Entity>>,
    r_assets: Res<AssetServer>,
    // ra_meshes: Res<Assets<Mesh>>,
    // ra_mats: Res<Assets<StandardMaterial>>,
    q_meshes: Query<(Entity, &Name, &Handle<Mesh>, &Handle<StandardMaterial>)>,
) {
    for (ent, name, mesh, mat) in q_meshes.iter() {
        if l_logged.contains(&ent) {
            continue;
        }

        let mesh_id = mesh.id();
        if let HandleId::AssetPathId(path_id) = mesh_id {
            let source_id = path_id.source_path_id();
            info!("[{ent:?}] {name} | {source_id:?}");
            // let mesh_path = AssetPath::from(path_id);
        }

        // let mesh_path = r_assets.get_handle_path(mesh);
        // let mat_path = r_assets.get_handle_path(mat);
        // info!("[{ent:?}] {name} | mesh: {mesh_path:?}; mat: {mat_path:?}");

        l_logged.insert(ent);
    }
}
