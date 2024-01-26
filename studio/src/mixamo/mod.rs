use bevy::{gltf::Gltf, prelude::*, scene::InstanceId, utils::HashMap};

use crate::scene::Skeleton;

pub struct MixamoPlugin;

impl Plugin for MixamoPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<MixamoImportEvent>()
            .add_state::<MixamoState>()
            .insert_resource(MixamoRetargetMap::default())
            .add_systems(
                Update,
                (
                    import_mixamo_scene,
                    tick_mixamo_import.run_if(resource_exists::<MixamoImport>()),
                    tick_scene_spawning.run_if(in_state(MixamoState::Spawning)),
                    retarget_mixamo_anim.run_if(in_state(MixamoState::Processing)),
                ),
            )
            .add_systems(OnEnter(MixamoState::Spawning), spawn_mixamo_scene);
    }
}

#[derive(States, Clone, Copy, Default, Debug, PartialEq, Eq, Hash)]
pub enum MixamoState {
    #[default]
    None,
    Spawning,
    Processing,
}

#[derive(Resource, Deref)]
pub struct MixamoImport(pub Handle<Gltf>);

#[derive(Event, Deref)]
pub struct MixamoImportEvent(pub String);

#[derive(Resource, Deref)]
pub struct MixamoScene(pub InstanceId);

#[derive(Component)]
pub struct MixamoRoot;

#[derive(Resource, Deref)]
pub struct MixamoRetargetMap(pub HashMap<String, String>);

fn import_mixamo_scene(
    mut cmd: Commands,
    r_assets: Res<AssetServer>,
    mut er: EventReader<MixamoImportEvent>,
) {
    if let Some(event) = er.read().last() {
        let path = &**event;
        cmd.insert_resource(MixamoImport(r_assets.load(path)));
    }
}

fn tick_mixamo_import(
    r_mixamo_import: Res<MixamoImport>,
    ra_gltfs: Res<Assets<Gltf>>,
    mut er_scene_events: EventReader<AssetEvent<Scene>>,
    mut st_next: ResMut<NextState<MixamoState>>,
) {
    for event in er_scene_events.read() {
        if let AssetEvent::LoadedWithDependencies { id } = event {
            let gltf = ra_gltfs.get(&**r_mixamo_import).unwrap();
            if gltf.default_scene.is_some() && gltf.default_scene.as_ref().unwrap().id() == *id {
                st_next.set(MixamoState::Spawning);
                break;
            }
        }
    }
}

fn spawn_mixamo_scene(
    mut cmd: Commands,
    ra_gltfs: Res<Assets<Gltf>>,
    r_mixamo_import: Res<MixamoImport>,
    mut r_scene_spawner: ResMut<SceneSpawner>,
) {
    let gltf = ra_gltfs.get(&**r_mixamo_import).unwrap();
    let scene = gltf.default_scene.as_ref().unwrap();
    let parent = cmd
        .spawn((
            Name::new("Mixamo"),
            MixamoRoot,
            SpatialBundle {
                visibility: Visibility::Visible,
                ..default()
            },
        ))
        .id();

    let inst = r_scene_spawner.spawn_as_child(scene.clone(), parent);
    cmd.insert_resource(MixamoScene(inst));
}

fn tick_scene_spawning(
    r_scene_spawner: Res<SceneSpawner>,
    r_mixamo_scene: Res<MixamoScene>,
    mut st_next: ResMut<NextState<MixamoState>>,
) {
    if r_scene_spawner.instance_is_ready(**r_mixamo_scene) {
        st_next.set(MixamoState::Processing);
    }
}

fn retarget_mixamo_anim(
    mut cmd: Commands,
    ra_gltfs: Res<Assets<Gltf>>,
    mut ra_animations: ResMut<Assets<AnimationClip>>,
    r_scene_spawner: Res<SceneSpawner>,
    r_mixamo_import: Res<MixamoImport>,
    r_mixamo_scene: Res<MixamoScene>,
    r_retarget_map: Res<MixamoRetargetMap>,
    q_target: Query<(Entity, &Name, &Children), With<Skeleton>>,
    q_nodes: Query<(&Name, Option<&Children>)>,
    mut st_next: ResMut<NextState<MixamoState>>,
) {
    let gltf = ra_gltfs.get(&**r_mixamo_import).unwrap();
    let src_anim = ra_animations.get(gltf.animations[0].clone_weak()).unwrap();

    let Some(src_root) = r_scene_spawner
        .iter_instance_entities(**r_mixamo_scene)
        .find(|&ent| {
            matches!(
                q_nodes
                    .get(ent)
                    .ok()
                    .map(|(name, _)| src_anim.compatible_with(name)),
                Some(true)
            )
        })
    else {
        error!("Failed to find compatible entity for animation");
        return;
    };
    let Ok((src_root_name, Some(src_root_children))) = q_nodes.get(src_root) else {
        error!("Failed to find source root with children");
        return;
    };

    let Ok((tgt_root, tgt_root_name, tgt_root_children)) = q_target.get_single() else {
        error!("Failed to find target root");
        return;
    };

    let mut tgt_anim = AnimationClip::default();

    info!("Source root: {src_root_name}");
    info!("Target root: {tgt_root_name}");

    retarget_node(
        src_anim,
        src_root_name,
        Some(src_root_children),
        &EntityPath {
            parts: vec![src_root_name.clone()],
        },
        &mut tgt_anim,
        tgt_root_name,
        Some(tgt_root_children),
        &EntityPath {
            parts: vec![tgt_root_name.clone()],
        },
        &r_retarget_map,
        &q_nodes,
    );

    let mut player = AnimationPlayer::default();
    player.play(ra_animations.add(tgt_anim)).repeat();

    cmd.entity(tgt_root).insert(player);

    st_next.set(MixamoState::None);
}

fn retarget_node(
    src_anim: &AnimationClip,
    src_name: &Name,
    src_children: Option<&Children>,
    src_path: &EntityPath,
    tgt_anim: &mut AnimationClip,
    tgt_name: &Name,
    tgt_children: Option<&Children>,
    tgt_path: &EntityPath,
    retarget_map: &MixamoRetargetMap,
    q_nodes: &Query<(&Name, Option<&Children>)>,
) {
    'retarget: {
        if let Some(tn) = retarget_map.get(src_name.as_str()) {
            if tn != tgt_name.as_str() {
                // TODO: BFS descendants..?
                break 'retarget;
            }
        }

        let Some(src_curves) = src_anim.get_curves_by_path(src_path) else {
            break 'retarget;
        };

        for curve in src_curves.iter().cloned() {
            // TODO: Retarget the keyframe values
            tgt_anim.add_curve_to_path(tgt_path.clone(), curve);
        }
    }

    let Some(src_children) = src_children else {
        return;
    };
    let Some(tgt_children) = tgt_children else {
        return;
    };

    for &src_next in src_children.iter() {
        let Ok((src_name, src_children)) = q_nodes.get(src_next) else {
            continue;
        };
        let Some(tn) = retarget_map.get(src_name.as_str()) else {
            continue;
        };
        let Some((tgt_name, tgt_children)) =
            tgt_children.iter().find_map(|&tgt| match q_nodes.get(tgt) {
                Ok((name, children)) => {
                    if name.as_str() == tn {
                        Some((name, children))
                    } else {
                        None
                    }
                }
                _ => None,
            })
        else {
            // TODO: BFS descendants..?
            continue;
        };

        let mut src_path_parts = src_path.parts.clone();
        src_path_parts.push(src_name.clone());

        let mut tgt_path_parts = tgt_path.parts.clone();
        tgt_path_parts.push(tgt_name.clone());

        retarget_node(
            src_anim,
            src_name,
            src_children,
            &EntityPath {
                parts: src_path_parts,
            },
            tgt_anim,
            tgt_name,
            tgt_children,
            &EntityPath {
                parts: tgt_path_parts,
            },
            retarget_map,
            q_nodes,
        );
    }
}

impl Default for MixamoRetargetMap {
    fn default() -> Self {
        Self(
            [
                // Spine
                ("mixamorig:Hips", "DEF-spine"),
                ("mixamorig:Spine", "DEF-spine.002"),
                ("mixamorig:Spine2", "DEF-spine.003"),
                ("mixamorig:Neck", "DEF-spine.004"),
                ("mixamorig:Head", "DEF-spine.006"),
                // Right arm
                ("mixamorig:RightShoulder", "DEF-shoulder.R"),
                ("mixamorig:RightArm", "DEF-upper_arm.R"),
                ("mixamorig:RightForeArm", "DEF-forearm.R"),
                ("mixamorig:RightHand", "DEF-hand.R"),
                // Right fingers
                // -- index
                ("mixamorig:RightHandIndex1", "DEF-palm.01.R"),
                ("mixamorig:RightHandIndex2", "DEF-f_index.01.R"),
                ("mixamorig:RightHandIndex3", "DEF-f_index.02.R"),
                ("mixamorig:RightHandIndex4", "DEF-f_index.03.R"),
                // -- middle
                ("mixamorig:RightHandMiddle1", "DEF-palm.02.R"),
                ("mixamorig:RightHandMiddle2", "DEF-f_middle.01.R"),
                ("mixamorig:RightHandMiddle3", "DEF-f_middle.02.R"),
                ("mixamorig:RightHandMiddle4", "DEF-f_middle.03.R"),
                // -- ring
                ("mixamorig:RightHandRing1", "DEF-palm.03.R"),
                ("mixamorig:RightHandRing2", "DEF-f_ring.01.R"),
                ("mixamorig:RightHandRing3", "DEF-f_ring.02.R"),
                ("mixamorig:RightHandRing4", "DEF-f_ring.03.R"),
                // -- pinky
                ("mixamorig:RightHandPinky1", "DEF-palm.04.R"),
                ("mixamorig:RightHandPinky2", "DEF-f_pinky.01.R"),
                ("mixamorig:RightHandPinky3", "DEF-f_pinky.02.R"),
                ("mixamorig:RightHandPinky4", "DEF-f_pinky.03.R"),
                // -- thumb
                ("mixamorig:RightHandThumb1", "DEF-thumb.01.R"),
                ("mixamorig:RightHandThumb2", "DEF-thumb.02.R"),
                ("mixamorig:RightHandThumb3", "DEF-thumb.03.R"),
                // Right Leg
                ("mixamorig:RightUpLeg", "DEF-thigh.R"),
                ("mixamorig:RightLeg", "DEF-shin.R"),
                ("mixamorig:RightFoot", "DEF-foot.R"),
                ("mixamorig:RightToeBase", "DEF-toe.R"),
                // Left arm
                ("mixamorig:LeftShoulder", "DEF-shoulder.L"),
                ("mixamorig:LeftArm", "DEF-upper_arm.L"),
                ("mixamorig:LeftForeArm", "DEF-forearm.L"),
                ("mixamorig:LeftHand", "DEF-hand.L"),
                // Left fingers
                // -- index
                ("mixamorig:LeftHandIndex1", "DEF-palm.01.L"),
                ("mixamorig:LeftHandIndex2", "DEF-f_index.01.L"),
                ("mixamorig:LeftHandIndex3", "DEF-f_index.02.L"),
                ("mixamorig:LeftHandIndex4", "DEF-f_index.03.L"),
                // -- middle
                ("mixamorig:LeftHandMiddle1", "DEF-palm.02.L"),
                ("mixamorig:LeftHandMiddle2", "DEF-f_middle.01.L"),
                ("mixamorig:LeftHandMiddle3", "DEF-f_middle.02.L"),
                ("mixamorig:LeftHandMiddle4", "DEF-f_middle.03.L"),
                // -- ring
                ("mixamorig:LeftHandRing1", "DEF-palm.03.L"),
                ("mixamorig:LeftHandRing2", "DEF-f_ring.01.L"),
                ("mixamorig:LeftHandRing3", "DEF-f_ring.02.L"),
                ("mixamorig:LeftHandRing4", "DEF-f_ring.03.L"),
                // -- pinky
                ("mixamorig:LeftHandPinky1", "DEF-palm.04.L"),
                ("mixamorig:LeftHandPinky2", "DEF-f_pinky.01.L"),
                ("mixamorig:LeftHandPinky3", "DEF-f_pinky.02.L"),
                ("mixamorig:LeftHandPinky4", "DEF-f_pinky.03.L"),
                // -- thumb
                ("mixamorig:LeftHandThumb1", "DEF-thumb.01.L"),
                ("mixamorig:LeftHandThumb2", "DEF-thumb.02.L"),
                ("mixamorig:LeftHandThumb3", "DEF-thumb.03.L"),
                // Left Leg
                ("mixamorig:LeftUpLeg", "DEF-thigh.L"),
                ("mixamorig:LeftLeg", "DEF-shin.L"),
                ("mixamorig:LeftFoot", "DEF-foot.L"),
                ("mixamorig:LeftToeBase", "DEF-toe.L"),
            ]
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect(),
        )
    }
}
