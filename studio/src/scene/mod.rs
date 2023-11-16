use bevy::{
    core_pipeline::{
        bloom::{BloomCompositeMode, BloomPrefilterSettings, BloomSettings},
        fxaa::Fxaa,
        tonemapping::Tonemapping,
    },
    gltf::{Gltf, GltfMesh, GltfNode},
    math::vec3,
    pbr::NotShadowCaster,
    prelude::*,
    reflect::{TypePath, TypeUuid},
    render::render_resource::{AsBindGroup, ShaderRef},
    scene::SceneInstance,
    utils::HashSet,
};
use bevy_mod_picking::PickableBundle;

use crate::{
    camera_controller::{CameraController, CameraControllerBundle, CameraMotionSystem},
    interaction::Selectable,
    skybox::SkyboxId,
    MainCamera,
};

#[cfg(feature = "wasm")]
mod wasm;

pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(Color::hsl(0.0, 0.0, 0.25)))
            .insert_resource(Msaa::Off)
            .add_plugins(MaterialPlugin::<ProceduralGridMaterial>::default())
            .add_systems(Startup, initialize)
            .add_systems(
                Update,
                mark_scene_objects.run_if(resource_exists::<NewScene>().and_then(scene_inst_ready)),
            )
            .add_systems(PostUpdate, move_plane_with_camera.after(CameraMotionSystem));

        #[cfg(feature = "wasm")]
        app.add_systems(
            PostUpdate,
            (
                wasm::notify_entity_spawn_changes.pipe(error),
                wasm::save_scene_system.pipe(error),
            ),
        );
    }
}

#[derive(Component)]
struct WorldGridPlane;

#[derive(Resource, Deref, DerefMut)]
struct NewScene {
    #[deref]
    pub scene: Handle<Scene>,
    pub gltf: Option<Handle<Gltf>>,
}

#[derive(Component)]
struct Inspectable;

fn initialize(
    mut cmd: Commands,
    assets: Res<AssetServer>,
    mut ra_meshes: ResMut<Assets<Mesh>>,
    mut ra_mat_proc_grid: ResMut<Assets<ProceduralGridMaterial>>,
) {
    // Skybox
    cmd.insert_resource(SkyboxId("test_scene".into()));

    // glTF scene
    let gltf = assets.load("gltf/sample-room.gltf");
    let scene = assets.load("gltf/sample-room.gltf#Scene0");
    cmd.insert_resource(NewScene {
        gltf: Some(gltf.clone()),
        scene: scene.clone(),
    });
    cmd.spawn((
        Name::new("Scene"),
        Inspectable,
        SceneBundle { scene, ..default() },
    ))
    .with_children(|cmd| {
        // Camera, graphics settings
        let focus = vec3(0.35011, 0.5, -0.99576);
        let camera_xform = Transform::from_xyz(-2.5, 4.5, 5.0).looking_at(focus, Vec3::Y);
        cmd.spawn((
            Name::new("Main Camera"),
            MainCamera,
            Inspectable,
            Camera3dBundle {
                transform: camera_xform,
                tonemapping: Tonemapping::BlenderFilmic,
                camera: Camera {
                    hdr: true,
                    ..default()
                },
                ..default()
            },
            CameraControllerBundle::new(focus, camera_xform.translation),
            BloomSettings {
                composite_mode: BloomCompositeMode::EnergyConserving,
                intensity: 0.25,
                prefilter_settings: BloomPrefilterSettings {
                    threshold: 0.8,
                    threshold_softness: 0.5,
                },
                ..default()
            },
            Fxaa::default(),
        ));
    });

    // Plane
    let plane = ra_meshes.add(Mesh::from(shape::Plane {
        size: 200.0,
        subdivisions: 0,
    }));
    let proc_grid = ra_mat_proc_grid.add(ProceduralGridMaterial {});

    cmd.spawn((
        Name::new("World Grid"),
        SpatialBundle::from_transform(Transform::IDENTITY),
        WorldGridPlane,
    ))
    .with_children(|parent| {
        parent.spawn((
            MaterialMeshBundle {
                mesh: plane.clone(),
                material: proc_grid.clone(),
                transform: Transform::IDENTITY,
                ..default()
            },
            NotShadowCaster,
        ));
        parent.spawn((
            MaterialMeshBundle {
                mesh: plane,
                material: proc_grid,
                transform: Transform::IDENTITY.looking_to(Vec3::Z, Vec3::NEG_Y),
                ..default()
            },
            NotShadowCaster,
        ));
    });
}

fn scene_inst_ready(r_scene_mgr: Res<SceneSpawner>, q_scene_inst: Query<&SceneInstance>) -> bool {
    let Ok(scene_inst) = q_scene_inst.get_single() else {
        return false;
    };

    r_scene_mgr.instance_is_ready(**scene_inst)
}

fn mark_scene_objects(
    mut cmd: Commands,
    r_scene: Res<NewScene>,
    r_assets: Res<AssetServer>,
    ra_gltfs: Res<Assets<Gltf>>,
    ra_gltf_nodes: Res<Assets<GltfNode>>,
    ra_gltf_meshes: Res<Assets<GltfMesh>>,
    q_scene_inst: Query<Entity, With<SceneInstance>>,
    q_parents: Query<(Entity, &Name, &Children), Without<Selectable>>,
    q_primitives: Query<&Handle<Mesh>>,
) {
    let scene_ent = q_scene_inst.single();

    // Spawn lights
    cmd.entity(scene_ent).with_children(|cmd| {
        // Main light
        // TODO: get Blender to export enough metadata to automate light placements
        cmd.spawn((
            Name::new("Ceiling Light"),
            Selectable,
            Inspectable,
            PointLightBundle {
                point_light: PointLight {
                    color: Color::hex("FFE5B4").unwrap(),
                    intensity: 200.,
                    radius: 0.1,
                    shadows_enabled: true,
                    ..default()
                },
                transform: Transform::from_xyz(0.35263, 2.4348, 0.19601),
                ..default()
            },
        ));

        // Monitor glow
        cmd.spawn((
            Name::new("Monitor Glow"),
            Selectable,
            Inspectable,
            PointLightBundle {
                point_light: PointLight {
                    color: Color::WHITE,
                    intensity: 6.,
                    radius: 0.25,
                    shadows_enabled: false,
                    ..default()
                },
                transform: Transform::from_xyz(0.42615, 1.0706, -2.0705),
                ..default()
            },
        ));
    });

    if let Some(gltf) = r_scene
        .gltf
        .as_ref()
        .and_then(|handle| ra_gltfs.get(handle))
    {
        for (ent_parent, name, children) in q_parents.iter() {
            if gltf.named_nodes.contains_key(&**name) {
                cmd.entity(ent_parent).insert(Inspectable);
            }

            // See if this name matches any of the meshes or nodes in our glTF scene
            let Some(gltf_mesh) = gltf
                .named_meshes
                .get(&**name)
                .cloned()
                .or_else(|| {
                    gltf.named_nodes.get(&**name).and_then(|node_handle| {
                        ra_gltf_nodes
                            .get(node_handle)
                            .and_then(|node| node.mesh.clone())
                    })
                })
                .and_then(|handle| ra_gltf_meshes.get(&handle))
            else {
                continue;
            };

            // FIXME: These hashsets don't actually contain anything. Ever. -_-
            //        This has "worked" this whole time because {} == {}

            // Collect all of the mesh handles in the glTF mesh
            let prims_from_gltf = gltf_mesh
                .primitives
                .iter()
                .filter_map(|prim| r_assets.get_handle_path(&prim.mesh))
                .collect::<HashSet<_>>();

            // Collect all of the mesh handles in this entity's children
            let prims_from_ecs = children
                .iter()
                .filter_map(|ent_child| {
                    q_primitives
                        .get(*ent_child)
                        .ok()
                        .and_then(|handle| r_assets.get_handle_path(handle))
                })
                .collect::<HashSet<_>>();

            // If the sets match, this is the entity that represents the glTF mesh
            if prims_from_gltf == prims_from_ecs {
                cmd.entity(ent_parent).insert(Selectable);

                for &ent_child in children.iter() {
                    if q_primitives.contains(ent_child) {
                        cmd.entity(ent_child).insert(PickableBundle::default());
                    }
                }
            }
        }
    }

    cmd.remove_resource::<NewScene>();
}

fn move_plane_with_camera(
    mut plane: Query<(&WorldGridPlane, &mut Transform)>,
    camera: Query<&CameraController, Changed<CameraController>>,
) {
    if let Ok(&CameraController { focus, .. }) = camera.get_single() {
        let (_, mut xform) = plane.single_mut();

        xform.translation.x = focus.x;
        xform.translation.z = focus.z;
    }
}

#[derive(AsBindGroup, Clone, TypePath, TypeUuid)]
#[uuid = "CE331A13-F247-4F5B-A75E-670B2BD5776A"]
struct ProceduralGridMaterial {}

impl Material for ProceduralGridMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/procedural_grid.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }
}
