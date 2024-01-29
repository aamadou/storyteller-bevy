use bevy::{
    gltf::Gltf,
    math::{Affine3A, Mat3A},
    prelude::*,
    render::mesh::skinning::SkinnedMesh,
    scene::InstanceId,
    utils::{HashMap, HashSet},
};

use crate::{
    anim::{BindPoseGlobal, BindPoseLocal, InvBindPose, SkeletalJoint, Skeleton},
    scene::SceneState,
};

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
                    retarget_mixamo_anim.run_if(
                        in_state(SceneState::Active).and_then(in_state(MixamoState::Processing)),
                    ),
                ),
            )
            .add_systems(OnEnter(MixamoState::Spawning), spawn_mixamo_scene)
            .add_systems(OnEnter(MixamoState::Processing), tag_mixamo_elements);
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
            let Some(gltf) = ra_gltfs.get(&**r_mixamo_import) else {
                return;
            };

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
                visibility: Visibility::Visible, // TODO: Hidden
                transform: Transform::from_xyz(0., 0., -1.),
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

fn tag_mixamo_elements(
    mut cmd: Commands,
    r_scene_spawner: Res<SceneSpawner>,
    r_mixamo_scene: Res<MixamoScene>,
    q_skinned_meshes: Query<&SkinnedMesh>,
    mut q_xforms: ParamSet<(TransformHelper, Query<&Transform>)>,
) {
    for skinned_mesh in r_scene_spawner
        .iter_instance_entities(**r_mixamo_scene)
        .filter_map(|ent| q_skinned_meshes.get(ent).ok())
    {
        for &joint in skinned_mesh.joints.iter() {
            let global_bind_pose = q_xforms.p0().compute_global_transform(joint).unwrap();
            let local_bind_pose = *q_xforms.p1().get(joint).unwrap();

            cmd.entity(joint).insert((
                SkeletalJoint(Color::WHITE),
                BindPoseLocal(local_bind_pose),
                BindPoseGlobal(global_bind_pose.affine()),
                InvBindPose(global_bind_pose.affine().inverse()),
            ));
        }
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
    q_parents: Query<&Parent>,
    q_bind_poses: Query<&BindPoseGlobal>,
    q_inv_bind_poses: Query<&InvBindPose>,
    mut st_next: ResMut<NextState<MixamoState>>,
) {
    let gltf = ra_gltfs.get(&**r_mixamo_import).unwrap();
    let src_anim = ra_animations.get(gltf.animations[0].clone_weak()).unwrap();

    let Some(src_root) = r_scene_spawner
        .iter_instance_entities(**r_mixamo_scene)
        .find(|&ent| {
            let Ok((name, _)) = q_nodes.get(ent) else {
                return false;
            };
            src_anim.compatible_with(name)
        })
    else {
        error!("Failed to find compatible entity for animation");
        return;
    };
    let Ok((src_root_name, Some(_))) = q_nodes.get(src_root) else {
        error!("Failed to find source root with children");
        return;
    };

    let Ok((tgt_root, tgt_root_name, _)) = q_target.get_single() else {
        error!("Failed to find target root");
        return;
    };

    let mut tgt_anim = AnimationClip::default();

    info!("Source root: {src_root_name}");
    info!("Target root: {tgt_root_name}");

    let mut done = HashSet::default();

    retarget_node(
        src_root,
        src_anim,
        &EntityPath {
            parts: vec![src_root_name.clone()],
        },
        tgt_root,
        &mut tgt_anim,
        &EntityPath {
            parts: vec![tgt_root_name.clone()],
        },
        &r_retarget_map,
        &q_nodes,
        &q_parents,
        &q_bind_poses,
        &q_inv_bind_poses,
        &mut done,
    );

    let mut tgt_player = AnimationPlayer::default();
    tgt_player.play(ra_animations.add(tgt_anim)).repeat();
    cmd.entity(tgt_root).insert(tgt_player);

    // TODO: Just for debug
    let mut src_player = AnimationPlayer::default();
    src_player.play(gltf.animations[0].clone_weak()).repeat();
    cmd.entity(src_root).insert(src_player);

    st_next.set(MixamoState::None);
}

fn retarget_node(
    // Source
    src: Entity,
    src_anim: &AnimationClip,
    src_path: &EntityPath,
    // Target
    tgt: Entity,
    tgt_anim: &mut AnimationClip,
    tgt_path: &EntityPath,
    // System params
    retarget_map: &MixamoRetargetMap,
    q_nodes: &Query<(&Name, Option<&Children>)>,
    q_parents: &Query<&Parent>,
    q_bind_poses: &Query<&BindPoseGlobal>,
    q_inv_bind_poses: &Query<&InvBindPose>,
    done: &mut HashSet<EntityPath>,
) {
    let (src_name, src_children) = q_nodes.get(src).unwrap();
    let (tgt_name, tgt_children) = q_nodes.get(tgt).unwrap();

    'retarget: {
        if done.contains(src_path) {
            break 'retarget;
        }

        if let Some(tn) = retarget_map.get(src_name.as_str()) {
            if tn != tgt_name.as_str() {
                let Some(tgt_children) = tgt_children else {
                    break 'retarget;
                };

                for &tgt_child in tgt_children.iter() {
                    let Ok((tgt_name, _)) = q_nodes.get(tgt_child) else {
                        continue;
                    };
                    let mut tgt_path_parts = tgt_path.parts.clone();
                    tgt_path_parts.push(tgt_name.clone());

                    retarget_node(
                        src,
                        src_anim,
                        src_path,
                        tgt_child,
                        tgt_anim,
                        &EntityPath {
                            parts: tgt_path_parts,
                        },
                        retarget_map,
                        q_nodes,
                        q_parents,
                        q_bind_poses,
                        q_inv_bind_poses,
                        done,
                    );
                }

                break 'retarget;
            }
        }

        let Some(src_curves) = src_anim.get_curves_by_path(src_path) else {
            break 'retarget;
        };

        for src_curve in src_curves.iter() {
            use Keyframes::*;

            let tgt_bind_pose = q_bind_poses.get(tgt).unwrap();
            let src_inv_bind_pose = q_inv_bind_poses.get(src).unwrap();
            let tgt_parent_inv_bind_pose = q_parents
                .get(tgt)
                .ok()
                .and_then(|parent| q_inv_bind_poses.get(**parent).ok())
                .map_or(Affine3A::IDENTITY, |pose| **pose);

            let tgt_curve_keys = match &src_curve.keyframes {
                Rotation(src_quats) => {
                    let tgt_quats = src_quats
                        .iter()
                        .map(|&q| {
                            let src_mat_world = Mat3A::from_quat(q);
                            let src_mat_local = src_mat_world * src_inv_bind_pose.matrix3;
                            let tgt_mat = tgt_parent_inv_bind_pose.matrix3
                                * src_mat_local
                                * tgt_bind_pose.matrix3;

                            Quat::from_mat3a(&tgt_mat)
                        })
                        .collect();

                    Rotation(tgt_quats)
                }
                Translation(vecs) => Translation(vecs.clone()),
                Scale(vecs) => Scale(vecs.clone()),
                Weights(weights) => Weights(weights.clone()),
            };

            tgt_anim.add_curve_to_path(
                tgt_path.clone(),
                VariableCurve {
                    keyframe_timestamps: src_curve.keyframe_timestamps.clone(),
                    keyframes: tgt_curve_keys,
                },
            );
        }

        done.insert(src_path.clone());
    }

    let Some(src_children) = src_children else {
        return;
    };
    let Some(tgt_children) = tgt_children else {
        return;
    };

    for &src_next in src_children.iter() {
        let Ok((src_name, _)) = q_nodes.get(src_next) else {
            continue;
        };
        let Some(tn) = retarget_map.get(src_name.as_str()) else {
            continue;
        };

        let mut src_path_parts = src_path.parts.clone();
        src_path_parts.push(src_name.clone());

        if let Some((tgt_next, tgt_name)) =
            tgt_children.iter().find_map(|&tgt| match q_nodes.get(tgt) {
                Ok((name, _)) => {
                    if name.as_str() == tn {
                        Some((tgt, name))
                    } else {
                        None
                    }
                }
                _ => None,
            })
        {
            let mut tgt_path_parts = tgt_path.parts.clone();
            tgt_path_parts.push(tgt_name.clone());

            retarget_node(
                src_next,
                src_anim,
                &EntityPath {
                    parts: src_path_parts,
                },
                tgt_next,
                tgt_anim,
                &EntityPath {
                    parts: tgt_path_parts,
                },
                retarget_map,
                q_nodes,
                q_parents,
                q_bind_poses,
                q_inv_bind_poses,
                done,
            );
        } else {
            for (tgt_next, tgt_name) in
                tgt_children
                    .iter()
                    .filter_map(|&child| match q_nodes.get(child) {
                        Ok((name, _)) => Some((child, name)),
                        _ => None,
                    })
            {
                let mut tgt_path_parts = tgt_path.parts.clone();
                tgt_path_parts.push(tgt_name.clone());

                retarget_node(
                    src_next,
                    src_anim,
                    &EntityPath {
                        parts: src_path_parts.clone(),
                    },
                    tgt_next,
                    tgt_anim,
                    &EntityPath {
                        parts: tgt_path_parts,
                    },
                    retarget_map,
                    q_nodes,
                    q_parents,
                    q_bind_poses,
                    q_inv_bind_poses,
                    done,
                );
            }
        };
    }
}

impl Default for MixamoRetargetMap {
    fn default() -> Self {
        Self(
            [
                // Spine
                ("mixamorig:Hips", "DEF-spine"),
                ("mixamorig:Spine", "DEF-spine.001"),
                ("mixamorig:Spine1", "DEF-spine.002"),
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
