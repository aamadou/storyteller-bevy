use bevy::math::vec3;
#[rustfmt::skip]
use bevy::{
    pbr::NotShadowCaster,
    prelude::*,
    reflect::{TypePath, TypeUuid},
    render::render_resource::{AsBindGroup, ShaderRef},
};

use crate::{
    interaction::Selected,
    orbit_cam::{CameraMotionSystem, OrbitCam},
};

pub struct TestScenePlugin;

impl Plugin for TestScenePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(Color::hsl(0.0, 0.0, 0.25)))
            .add_plugins(MaterialPlugin::<ProceduralGridMaterial>::default())
            .add_systems(Startup, spawn_scene)
            .add_systems(Update, right_round_round_round)
            .add_systems(PostUpdate, move_plane_with_camera.after(CameraMotionSystem));
    }
}

#[derive(Component)]
struct WorldGridPlane;

#[derive(Component)]
struct Spin(Vec3); // TODO: Debug

fn spawn_scene(
    mut cmd: Commands,
    assets: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mat_proc_grid: ResMut<Assets<ProceduralGridMaterial>>,
) {
    // Skeletal mesh
    cmd.spawn((SpatialBundle::default(), Spin(Vec3::Y * 0.5)))
        .with_children(|parent| {
            parent.spawn((
                SceneBundle {
                    scene: assets.load("gltf/base_human_male.gltf#Scene0"),
                    transform: Transform::from_translation(vec3(0., 0., 2.5)),
                    ..default()
                },
                Selected,
                Spin(vec3(0., -1.5, 1.)),
            ));
        });

    // Plane
    let plane = meshes.add(Mesh::from(shape::Plane {
        size: 200.0,
        subdivisions: 0,
    }));
    let proc_grid = mat_proc_grid.add(ProceduralGridMaterial {});

    cmd.spawn((
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

fn move_plane_with_camera(
    mut plane: Query<(&WorldGridPlane, &mut Transform)>,
    camera: Query<&OrbitCam, Changed<OrbitCam>>,
) {
    if let Ok(&OrbitCam { focus, .. }) = camera.get_single() {
        let (_, mut xform) = plane.single_mut();

        xform.translation.x = focus.x;
        xform.translation.z = focus.z;
    }
}

fn right_round_round_round(mut q_spin: Query<(&mut Transform, &Spin)>, time: Res<Time>) {
    let dt = time.delta_seconds();

    for (mut xform, spin) in q_spin.iter_mut() {
        let speed = spin.0.length();
        let axis = spin.0 / speed;

        xform.rotate_local_axis(axis, std::f32::consts::PI * speed * dt);
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
