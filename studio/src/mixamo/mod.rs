use bevy::{
    gltf::Gltf, prelude::*, render::mesh::skinning::SkinnedMesh, scene::InstanceId,
    transform::TransformSystem, utils::HashMap,
};

use crate::{
    anim::{BindPoseGlobal, BindPoseLocal, EntityPathExtras, InvBindPose, SkeletalJoint, Skeleton},
    math::NearlyEq,
    scene::SceneState,
};

use self::tree::RetargetTree;

mod tree;

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
                    build_retarget_trees.run_if(
                        in_state(SceneState::Active)
                            .and_then(in_state(MixamoState::Processing).and_then(run_once())),
                    ),
                ),
            )
            .add_systems(
                PostUpdate,
                copy_poses
                    .run_if(resource_exists::<RetargetTree>())
                    .after(TransformSystem::TransformPropagate),
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
    info!("tag_mixamo_elements");

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

fn build_retarget_trees(
    mut cmd: Commands,
    ra_gltfs: Res<Assets<Gltf>>,
    r_mixamo_import: Res<MixamoImport>,
    r_mixamo_scene: Res<MixamoScene>,
    ra_animations: Res<Assets<AnimationClip>>,
    r_scene_spawner: Res<SceneSpawner>,
    q_target: Query<Entity, With<Skeleton>>,
    q_joints: Query<&SkeletalJoint>,
    q_names: Query<&Name>,
    q_children: Query<&Children>,
) {
    let gltf = ra_gltfs.get(&**r_mixamo_import).unwrap();
    let src_anim_handle = gltf.animations[0].clone();
    let src_anim = ra_animations.get(src_anim_handle.clone()).unwrap();

    let Some(src_root) = r_scene_spawner
        .iter_instance_entities(**r_mixamo_scene)
        .find(|ent| match q_names.get(*ent) {
            Ok(name) => src_anim.compatible_with(name),
            Err(_) => false,
        })
    else {
        error!("Failed to find compatible entity for animation");
        return;
    };

    let mut src_player = AnimationPlayer::default();
    src_player.play(src_anim_handle.clone()).repeat();
    cmd.entity(src_root).insert(src_player);

    let Ok(tgt_root) = q_target.get_single() else {
        error!("Failed to find target root");
        return;
    };

    let tgt_tree = RetargetTree::build(tgt_root, &q_names, &q_children, &q_joints);

    cmd.insert_resource(tgt_tree);
}

fn copy_poses(
    // TODO: Refactor to make this a component-based approach instead of resource-based
    r_retarget_tree: Res<RetargetTree>,
    r_retarget_map: Res<MixamoRetargetMap>,
    q_ent_names: Query<(Entity, &Name), With<SkeletalJoint>>,
    q_inv_bind_poses: Query<&InvBindPose>,
    q_parents: Query<&Parent>,
    q_world_bind_poses: Query<&BindPoseGlobal>,
    q_local_bind_poses: Query<&BindPoseLocal>,
    mut q_xforms: Query<&mut Transform>,
    q_world_xforms: Query<&GlobalTransform>,
) {
    // FIXME:
    // This is very much not fast, and probably shouldn't be done every frame.
    // Tentative plan to optimize this without adding too much additional
    // complexity:
    //
    // 1) Add some sort of "AnimationRecorder" component to the target root
    // 2) Beginning with `src_anim.seek_time == 0.0`, record each local
    //    rotation and translation performed on the target bones as a new
    //    keyframe on an `AnimationClip` held by `AnimationRecorder`
    // 3) When `src_anim.seek_time == src_anim.duration`, finalize the new
    //    `AnimationClip` and add it to the asset server
    // 4) Stop running this system and instead start playing the new animation
    //    with an `AnimationPlayer` component on the target root
    r_retarget_tree.for_each(|node| {
        // Look up the source bone in the retarget map
        let Some(src_ent) = r_retarget_map
            .tgt_to_src
            .get(node.path.leaf().as_str())
            .and_then(|src_name| {
                // FIXME: This is a slow O(n) lookup where n = the number of
                //        named entities in the world. We can make this O(1) on
                //        subsequent system runs by adding the results to a
                //        local hashmap as they're found, and checking that map
                //        before searching by name.
                q_ent_names.iter().find_map(|(ent, name)| {
                    if name.as_str() == src_name {
                        Some(ent)
                    } else {
                        None
                    }
                })
            })
        else {
            return;
        };

        // Get the query data needed to compute the target bone's local rotation
        let Ok(src_inv_bind_pose) = q_inv_bind_poses.get(src_ent) else {
            return;
        };
        let Ok(src_world_xform) = q_world_xforms.get(src_ent) else {
            return;
        };
        let Ok(tgt_world_bind_pose) = q_world_bind_poses.get(node.id) else {
            return;
        };

        // The math here was adapted from this writeup:
        // https://wickedengine.net/2022/09/23/animation-retargeting/
        //
        // ```pseudocode
        // // Apply target-local rotation
        // tgt_working_xform = src_world_xform * src_inv_bind_pose
        // tgt_working_xform = tgt_inv_parent_xform * local_xform * tgt_world_bind_pose
        // tgt_xform.rotation = tgt_working_xform.rotation
        //
        // // Check if the translation component is animated
        // if src_local_xform.translation != src_local_bind_pose.translation {
        //     // Apply target-local translation
        //     src_world_xlation = src_inv_bind_pose * src_local_xform.translation
        //     tgt_local_xlation = tgt_inv_parent_xform * src_world_xlation
        //     tgt_xform.translation = tgt_local_xlation
        // }
        // ```
        let mut tgt_working_xform = src_world_xform.affine() * (**src_inv_bind_pose);

        // The parent bone's world transform must be computed each iteration,
        // since it will be affected by the rotations applied on the previous
        // iteration.
        let parent_xform = node
            .parent
            .and_then(|parent| q_xforms.get(parent).ok())
            .copied()
            .unwrap_or_default();

        let mut parent_world_xform = GlobalTransform::from(parent_xform);
        if let Some(parent) = node.parent {
            for ancestor in q_parents.iter_ancestors(parent) {
                let xform = q_xforms.get(ancestor).copied().unwrap_or_default();
                parent_world_xform = xform * parent_world_xform;
            }
        }

        let tgt_inv_parent_xform = parent_world_xform.affine().inverse();
        tgt_working_xform = tgt_inv_parent_xform * tgt_working_xform * (**tgt_world_bind_pose);

        // Apply the local rotation
        if let Ok(mut tgt_xform) = q_xforms.get_mut(node.id) {
            tgt_xform.rotation = Quat::from_affine3(&tgt_working_xform);
        }

        // Get the query data needed to compute the target bone's local translation
        let Ok(src_local_bind_pose) = q_local_bind_poses.get(src_ent) else {
            return;
        };
        let Ok(src_world_bind_pose) = q_world_bind_poses.get(src_ent) else {
            return;
        };
        let Ok(src_local_xlation) = q_xforms.get(src_ent).map(|xform| xform.translation) else {
            return;
        };

        // If the source bone translation != its bind pose translation,
        // this translation component is animated (likely root motion)
        if !src_local_xlation.nearly_eq_within(src_local_bind_pose.translation, 1e-3) {
            let src_world_xlation = src_world_bind_pose.transform_vector3(src_local_xlation);
            let tgt_local_xlation = tgt_inv_parent_xform.transform_vector3(src_world_xlation);

            // Apply the local translation
            if let Ok(mut tgt_xform) = q_xforms.get_mut(node.id) {
                tgt_xform.translation = tgt_local_xlation;
            }
        }
    });
}

#[derive(Resource)]
pub struct MixamoRetargetMap {
    src_to_tgt: HashMap<String, String>,
    tgt_to_src: HashMap<String, String>,
}

impl MixamoRetargetMap {
    const SRC_TO_TGT: [(&str, &str); 60] = [
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
    ];
}

impl Default for MixamoRetargetMap {
    fn default() -> Self {
        Self {
            src_to_tgt: Self::SRC_TO_TGT
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            tgt_to_src: Self::SRC_TO_TGT
                .iter()
                .map(|(v, k)| (k.to_string(), v.to_string()))
                .collect(),
        }
    }
}
