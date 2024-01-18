use std::backtrace::Backtrace;

use bevy::{
    asset::{AsyncReadExt, LoadState},
    ecs::{
        query::{self, ReadOnlyWorldQuery, WorldQuery},
        system::{Command, EntityCommands, SystemParam},
    },
    gltf::Gltf,
    prelude::*,
    render::INSTANCE_INDEX_SHADER_HANDLE,
    scene::{self, InstanceId},
    transform::commands,
    ui::debug::print_ui_layout_tree,
    utils::hashbrown::HashMap,
};
use bevy_inspector_egui::egui::TextBuffer;
use bevy_mod_picking::prelude::EntityEvent;
use bevy_rapier3d::parry::transformation;
use studio_bvh::{frames::Frame, joint, Bvh as BvhFile};

mod error;
mod loader;

use crate::{
    debug::print_mat3,
    scene::{ActiveScene, SceneState},
};

use self::loader::BvhLoader;

pub struct BvhPlugin;

impl Plugin for BvhPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<Bvh>()
            .register_asset_loader(BvhLoader)
            .init_resource::<PendingBVHAnims>()
            .add_systems(PreUpdate, tick_pending_bvh_anims.run_if(in_state(SceneState::Spawning)))
            
            .add_systems(Update, animate_bvh.run_if(in_state(SceneState::Active)))
            
            .register_type::<BvhJoint>()
            .register_type::<BvhAnim>()
            //Events
                ;
    }
}

#[derive(Clone, Debug, Default, Asset, TypePath)]
pub struct Bvh(studio_bvh::Bvh);

//==============================================================================
//                        Bvh Anim Component
//==============================================================================

#[derive(Component, Debug, Reflect)]
pub struct BvhAnim {
    #[reflect(ignore)]
    bvh: Handle<Bvh>,
    should_play: bool,
    frame: usize,
    // #[reflect(ignore)]
    // retarget: Option<RetargetMap>,
    #[reflect(ignore)]
    timer: Timer,
    #[reflect(ignore)]
    last_frame: usize,
}

impl BvhAnim {
    fn new(handle : Handle<Bvh>, bvh : &BvhFile) -> Self {
        BvhAnim { bvh: handle, should_play: true, frame: 0, timer: Timer::from_seconds(bvh.frame_time().as_secs_f32(), TimerMode::Repeating), last_frame:0  }
    }
}

pub fn animate_bvh(
    mut command: Commands,
    mut bvh_anims: Query<(Entity, &mut BvhAnim, &mut Transform, &mut Children)>,
    mut bvh_joints: Query<JointQuery, Without<BvhAnim>>,
    bvh_assets: Res<Assets<Bvh>>,
    time: Res<Time>,
) {
    for (entity, mut anim, transform, children) in bvh_anims.iter_mut() {
        let bvh = &bvh_assets
            .get(anim.bvh.clone())
            .expect("Could not animate the BVH because the bvh file has not been loaded yet")
            .0;

        if anim.should_play {
            anim.timer.tick(time.delta());
            if anim.timer.finished() {
                anim.frame = (anim.frame + 1) % bvh.frames().len();
                anim.timer.reset();
                // command.entity(entity).remove::<BvhAnim>();
            }
        }

        if anim.last_frame != anim.frame {
            let frame = bvh
                .frames()
                .nth(anim.frame)
                .expect("Invalid BVH format: the frame index is out of bounds");
            for child in children.iter() {
                apply_frame(&mut bvh_joints, *child, &frame, &bvh);
            }
            anim.last_frame = anim.frame;
        }
    }
}

fn apply_frame<T: ReadOnlyWorldQuery>(
    joint_query: &mut Query<JointQuery, T>,
    current_joint: Entity,
    frame: &Frame,
    bvh_file: &studio_bvh::Bvh,
) {
    let Ok(mut joint_entity) = joint_query.get_mut(current_joint) else {
        return;
    };

    if let Some(joint) = joint_entity.joint {
        let joint_data = bvh_file
            .get_joint(joint.0)
            .expect("Invalid BVH format: the joint index is out of bounds");

        //new Positions and rotations
        let mut new_pos = joint_entity.transform.translation;
        let mut new_rot: Vec3 = joint_entity
            .transform
            .rotation
            .to_euler(EulerRot::ZXY)
            .into();

        for channel in joint_data.channels() {
            // info!("Channel {}: {:?} : {}", channel.motion_index(), channel.channel_type(), frame.as_slice()[channel.motion_index()]);
            match channel.channel_type() {
                studio_bvh::ChannelType::PositionX => {
                    new_pos.x = frame.as_slice()[channel.motion_index()]
                }
                studio_bvh::ChannelType::PositionY => {
                    new_pos.y = frame.as_slice()[channel.motion_index()]
                }
                studio_bvh::ChannelType::PositionZ => {
                    new_pos.z = frame.as_slice()[channel.motion_index()]
                }
                studio_bvh::ChannelType::RotationX => {
                    new_rot.x += frame.as_slice()[channel.motion_index()]
                }
                studio_bvh::ChannelType::RotationY => {
                    new_rot.y += frame.as_slice()[channel.motion_index()]
                }
                studio_bvh::ChannelType::RotationZ => {
                    new_rot.z += frame.as_slice()[channel.motion_index()]
                }
            }
        }

        joint_entity.transform.translation = new_pos;
        joint_entity.transform.rotation = Quat::from_euler(
            EulerRot::ZXY,
            new_rot.z.to_radians(),
            new_rot.x.to_radians(),
            new_rot.y.to_radians(),
        );
    }

    if joint_entity.children.is_some() {
        let children = joint_entity
            .children
            .as_mut()
            .unwrap()
            .iter()
            .map(Entity::clone)
            .collect::<Vec<_>>();

        drop(joint_entity);

        for child in children {
            apply_frame(joint_query, child, frame, bvh_file);
        }
    }
}

//==============================================================================
//                        Bvh Anim Component
//==============================================================================

#[derive(Debug, Component, Reflect)]
pub struct BvhJoint(usize);

#[derive(Debug, WorldQuery)]
#[world_query(mutable)]
pub struct JointQuery {
    pub joint: Option<&'static BvhJoint>,
    pub children: Option<&'static mut Children>,
    pub transform: &'static mut Transform,
}

//==============================================================================
//                        Unloaded Scenes
//==============================================================================

#[derive(Resource, Default)]
pub struct PendingBVHAnims {
    pub items: Vec<(Entity, InstanceId, Handle<Bvh>, RetargetMap)>,
}

fn tick_pending_bvh_anims(
    mut commands : Commands,
    mut pending_bvh_anims: ResMut<PendingBVHAnims>,
    mut next_state : ResMut<NextState<SceneState>>,
    named_entities : Query<&Name>,
    scene_spawner: Res<SceneSpawner>,
    asset_server: Res<AssetServer>,
    bvh_assets: Res<Assets<Bvh>>,
) {
    let is_done_loading = |bvh_handle : &Handle<Bvh>, instance_id : &InstanceId| -> bool {
        asset_server.get_load_state(bvh_handle.id()) == Some(LoadState::Loaded) && scene_spawner.instance_is_ready(instance_id.clone())
    };
    
    let mut i = 0;
    while i < pending_bvh_anims.items.len() {
        let (parent, instance_id, bvh_handle, retarget_map) = pending_bvh_anims.items.get(i).unwrap();        
        if is_done_loading(bvh_handle, instance_id) {
            println!("Bvh anim is ready!");

            let bvh = bvh_assets.get(bvh_handle.id()).unwrap();
            
            for entity in scene_spawner.iter_instance_entities(*instance_id) {

                let Ok(potential_bone_name) = named_entities.get(entity) else {
                    continue;
                };
                let Some(mapped_bone_name) = retarget_map.map.get(potential_bone_name.as_str())
                else {
                    continue;
                };
                let Some(bone_data) = bvh.0.get_joint_by_name(&mapped_bone_name) else {
                    continue;
                };

                println!("Bone Index {}", bone_data.index());

                commands.entity(entity).insert(BvhJoint(bone_data.index()));
            }
            
            commands.entity(*parent)
                .insert(SpatialBundle::default())
                .insert(BvhAnim::new(bvh_handle.clone(), &bvh.0));

            commands.insert_resource(ActiveScene { instance: instance_id.clone() });
            
            pending_bvh_anims.items.remove(i);
        } else {
            i += 1;
        }
    }

    if pending_bvh_anims.items.is_empty() {
        println!("Moving to Active state.");
        next_state.set(SceneState::Active);
    }   
    // for (parent, instance_id, bvh_handle, retarget_map) in pending_bvh_anims.items.iter() {
    //     if let Some(LoadState::Loaded) = asset_server.get_load_state(bvh_handle) {
    //         if scene_spawner.instance_is_ready(*instance_idance_id) {
    //             println!("Bvh anim is ready!");

    //             let bvh = bvh_assets.get(bvh_handle.id()).unwrap();
    //             // commands.entity(parent).insert(BvhAnim{ bvh, should_play: todo!(), frame: todo!(), retarget: todo!(), timer: todo!(), last_frame: todo!() })
                
    //             for entity in scene_spawner.iter_instance_entities(*instance_id) {

    //                 let Ok(potential_bone_name) = named_entities.get(entity) else {
    //                     continue;
    //                 };
    //                 let Some(mapped_bone_name) = retarget_map.map.get(potential_bone_name.as_str())
    //                 else {
    //                     continue;
    //                 };
    //                 let Some(bone_data) = bvh.0.get_joint_by_name(&mapped_bone_name) else {
    //                     continue;
    //                 };

    //                 println!("Bone Index {}", bone_data.index());

    //                 commands.entity(entity).insert(BvhJoint(bone_data.index()));
    //             }
                
    //             commands.entity(*parent)
    //                 .insert(SpatialBundle::default())
    //                 .insert(BvhAnim::new(bvh_handle.clone(), &bvh.0));
    //         }
    //     }
    // }
}

//==============================================================================
//                        Commands Plugin
//==============================================================================

pub trait CommandsSpawnBvhExt {
    fn spawn_bvh(
        &mut self,
        asset: Handle<Bvh>,
        scene: Handle<Scene>,
        retarget_map: RetargetMap,
        transform: Transform,
    ) -> Entity;
}

impl<'w, 's> CommandsSpawnBvhExt for Commands<'w, 's> {
    fn spawn_bvh(
        &mut self,
        asset: Handle<Bvh>,
        scene: Handle<Scene>,
        retarget_map: RetargetMap,
        transform: Transform,
    ) -> Entity {
        let entity = self.spawn_empty().id();

        self.add(SpawnBvhCommand(
            entity,
            scene,
            asset,
            retarget_map,
            transform,
        ));
        entity
    }
}

pub struct SpawnBvhCommand(Entity, Handle<Scene>, Handle<Bvh>, RetargetMap, Transform);

impl Command for SpawnBvhCommand {
    //TODO: This loading process with have to be async eventually.
    fn apply(self, world: &mut World) {
        // let bvh = {
        //     let Some(bvh) = world.get_resource::<Assets<Bvh>>().unwrap().get(&self.1) else {
        //         panic!(
        //             "Could not create the BVH entity because the bvh file has not been loaded yet"
        //         )
        //     };

        //     bvh.clone()
        // };

        let mut scene_spanwer = world
            .get_resource_mut::<SceneSpawner>()
            .expect("The scene spawner has not been initialized.");
        let scene_id = scene_spanwer.spawn_as_child(self.1, self.0);

        drop(scene_spanwer);
        
        let mut pending_bvh_scenes = world.get_resource_mut::<PendingBVHAnims>().expect("Pending bvh Resource has not been initialized.");
        pending_bvh_scenes.items.push((self.0, scene_id, self.2, self.3));

        // let Some(root) = bvh.0.root_joint() else {
        //     panic!("Could not create the BVH entity because the bvh file has no root joint")
        // };
 
        // println!("Starting Scene Spawn");
        // let mut scene_spawner = SceneSpawner::from_world(world);
        
        // let id = match scene_spawner.spawn_sync(world, scene_handle.id()) {
        //     Ok(val) => val,
        //     Err(err) => {
        //         println!("There was an Error: {:?}", err);
        //         return;
        //     }
        // };

        // world.insert_resource(ActiveScene {
        //     instance: id,
        //     source: scene_handle,
        // });
    }
}

//==============================================================================
//                        Update BVH anim Event Functions
//==============================================================================

// #[derive(Event)]
// pub struct UpdateBVHAnim {
//     pub entity: Entity,
// }

// fn update_bvh_anim(
//     mut commands: Commands,
//     mut update_events: EventReader<UpdateBVHAnim>,
//     bvh_anims: Query<(Entity, &BvhAnim, Option<&Children>)>,
//     skinned_model_nodes: Query<(&Name, Option<&Children>)>,
//     bvh_assets: Res<Assets<Bvh>>,
// ) {
//     for event in update_events.read() {
//         // println!("Updating BVHs");
//         // println!("{:?} | {:?}", event.entity, bvh_anims.single().0);
//         // for anim in bvh_anims.iter() {
//         //     println!("Children {:?}", anim);
//         // };

//         if let Ok(bvh_anim) = bvh_anims.get(event.entity) {
//             let Some(bvh) = bvh_assets.get(bvh_anim.1.bvh.id()) else {
//                 continue;
//             };

//             println!("Children {:?}", bvh_anim.2);

//             if let Some(children) = bvh_anim.2 {
//                 for child in children {
//                     update_bvh_recursive(child, &mut commands, &bvh, &skinned_model_nodes);
//                 }
//             }
//         };
//     }
// }

// fn update_bvh_recursive(
//     current_node: &Entity,
//     commands: &mut Commands,
//     bvh_anim: &Bvh,
//     skinned_model_nodes: &Query<(&Name, Option<&Children>)>,
// ) {
//     println!("Called Function");

//     let Ok((current_name, current_children)) = skinned_model_nodes.get(*current_node) else {
//         return;
//     };

//     if let Some(children) = current_children {
//         for child in children.iter() {
//             update_bvh_recursive(child, commands, bvh_anim, skinned_model_nodes);
//         }
//     }
// }

//==============================================================================
//                        Utility Functions
//==============================================================================

fn spawn_bvh_recursive(world: &mut World, entity: &Entity, joint: studio_bvh::Joint) {
    let offset = joint.data().offset();
    let transform = Transform::from_xyz(offset[0], offset[1], offset[2]);

    // Save the name of the joint
    let name = String::from_utf8_lossy(joint.data().name()).to_string();

    world.entity_mut(*entity).insert((
        SpatialBundle::from_transform(transform),
        BvhJoint(joint.data().index()),
        Name::new(name),
    ));

    for child in joint.children() {
        let mut child_entity = world.spawn_empty().id();
        spawn_bvh_recursive(world, &child_entity, child);
        world.entity_mut(*entity).push_children(&[child_entity]);
    }
}

// fn locate_rig(world: &mut World, current_entity: Entity) -> Option<Entity> {
//     let Ok(query) = world
//         .query::<(Option<&Children>, Option<&Name>)>()
//         .get(world, current_entity)
//     else {
//         return None;
//     };

//     if let Some(name) = query.1 {
//         println!("Name: {}", name.as_str());
//         if name.as_str() == "Rig" {
//             return Some(current_entity);
//         }
//     }

//     let children = if let Some(children) = query.0 {
//         children.iter().map(|x| x.clone()).collect::<Vec<_>>()
//     } else {
//         return None;
//     };

//     for child in children.iter() {
//         println!("Child: {:?}", child);
//         if let Some(entity) = locate_rig(world, *child) {
//             return Some(entity);
//         }
//     }

//     None
// }

//==============================================================================
//                        RetargetMap
//==============================================================================

#[derive(Debug)]
pub struct RetargetMap {
    pub map: HashMap<String, String>,
}

impl RetargetMap {
    pub fn mocap_to_rigify() -> RetargetMap {
        RetargetMap {
            map: [
                ("DEF-spine", "LowerBack"),
                ("DEF-spine.002", "abdomen"),
                ("DEF-spine.003", "chest"),
                ("DEF-spine.004", "Neck"),
                ("DEF-spine.005", "Neck1"),
                ("DEF-spine.006", "Head"),
                ("DEF-shoulder.R", "rcollar"),
                ("DEF-upper_arm.R", "rshoulder"),
                ("DEF-forearm.R", "relbow"),
                ("DEF-hand.R", "rhand"),
                ("DEF-palm.01.R", "metacarpal1.r"),
                ("DEF-f_index.01.R", "finger2-1.r"),
                ("DEF-f_index.02.R", "finger2-2.r"),
                ("DEF-f_index.03.R", "finger2-3.r"),
                ("DEF-palm.02.R", "metacarpal2.r"),
                ("DEF-f_middle.01.R", "finger3-1.r"),
                ("DEF-f_middle.02.R", "finger3-2.r"),
                ("DEF-f_middle.03.R", "finger3-3.r"),
                ("DEF-palm.03.R", "metacarpal3.r"),
                ("DEF-f_ring.01.R", "finger4-1.r"),
                ("DEF-f_ring.02.R", "finger4-2.r"),
                ("DEF-f_ring.03.R", "finger4-3.r"),
                ("DEF-palm.04.R", "metacarpal4.r"),
                ("DEF-f_pinky.01.R", "finger5-1.r"),
                ("DEF-f_pinky.02.R", "finger5-2.r"),
                ("DEF-f_pinky.03.R", "finger5-3.r"),
                ("DEF-thumb.01.R", "rthumb"),
                ("DEF-thumb.02.R", "finger1-2.r"),
                ("DEF-thumb.03.R", "finger1-3.r"),
                ("DEF-thigh.R", "rhip"),
                ("DEF-shin.R", "rknee"),
                ("DEF-foot.R", "rfoot"),
                ("DEF-toe.R", "RightToeBase"),
                ("DEF-shoulder.L", "lcollar"),
                ("DEF-upper_arm.L", "lshoulder"),
                ("DEF-forearm.L", "lelbow"),
                ("DEF-hand.L", "lhand"),
                ("DEF-palm.01.L", "metacarpal1.l"),
                ("DEF-f_index.01.L", "finger2-1.l"),
                ("DEF-f_index.02.L", "finger2-2.l"),
                ("DEF-f_index.03.L", "finger2-3.l"),
                ("DEF-palm.02.L", "metacarpal2.l"),
                ("DEF-f_middle.01.L", "finger3-1.l"),
                ("DEF-f_middle.02.L", "finger3-2.l"),
                ("DEF-f_middle.03.L", "finger3-3.l"),
                ("DEF-palm.03.L", "metacarpal3.l"),
                ("DEF-f_ring.01.L", "finger4-1.l"),
                ("DEF-f_ring.02.L", "finger4-2.l"),
                ("DEF-f_ring.03.L", "finger4-3.l"),
                ("DEF-palm.04.L", "metacarpal4.l"),
                ("DEF-f_pinky.01.L", "finger5-1.l"),
                ("DEF-f_pinky.02.L", "finger5-2.l"),
                ("DEF-f_pinky.03.L", "finger5-3.l"),
                ("DEF-thumb.01.L", "lthumb"),
                ("DEF-thumb.02.L", "finger1-2.l"),
                ("DEF-thumb.03.L", "finger1-3.l"),
                ("DEF-thigh.L", "lhip"),
                ("DEF-shin.L", "lknee"),
                ("DEF-foot.L", "lfoot"),
                ("DEF-toe.L", "LeftToeBase"),
                ("DEF-pelvis.R", "RHipJoint"),
                ("DEF-pelvis.L", "LHipJoint"),
            ]
            .iter()
            .map(|v| (v.0.to_string(), v.1.to_string()))
            .collect(),
        }
    }

    pub fn is_match(&self, mocapnet_name: &str, rigify_name: &str) -> bool {
        if let Some(translated_name) = self.map.get(mocapnet_name) {
            if rigify_name == translated_name.as_str() {
                return true;
            }
        }

        false
    }
}
