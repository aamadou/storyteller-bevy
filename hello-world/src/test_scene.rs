use bevy::{
    pbr::NotShadowCaster,
    prelude::*,
    reflect::{TypePath, TypeUuid},
    render::render_resource::{AsBindGroup, ShaderRef},
};

pub struct TestScenePlugin;

impl Plugin for TestScenePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(Color::hsl(0.0, 0.0, 0.25)))
            .add_plugins(MaterialPlugin::<ProceduralGridMaterial> {
                prepass_enabled: true,
                ..default()
            })
            .add_systems(Startup, spawn_scene);
    }
}

fn spawn_scene(
    mut cmd: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut std_mats: ResMut<Assets<StandardMaterial>>,
    mut mat_proc_grid: ResMut<Assets<ProceduralGridMaterial>>,
) {
    let mut light_xform = Transform::IDENTITY;
    light_xform.rotation = Quat::from_rotation_y(f32::to_radians(-70.0)) * light_xform.rotation;
    light_xform.rotation *= Quat::from_rotation_x(f32::to_radians(-60.0));

    cmd.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 25_000.0,
            shadows_enabled: true,
            ..default()
        },
        transform: light_xform,
        ..default()
    });

    // Cube
    cmd.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: std_mats.add(StandardMaterial {
            base_color: Color::SILVER,
            ..default()
        }),
        transform: Transform::IDENTITY,
        ..default()
    });

    // Plane
    cmd.spawn((
        MaterialMeshBundle {
            mesh: meshes.add(Mesh::from(shape::Plane {
                size: 10_000.0,
                subdivisions: 0,
            })),
            material: mat_proc_grid.add(ProceduralGridMaterial {}),
            ..default()
        },
        NotShadowCaster,
    ));
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
