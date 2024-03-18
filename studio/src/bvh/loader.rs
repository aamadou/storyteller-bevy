use bevy::{
    animation::{AnimationClip, EntityPath, Keyframes, VariableCurve}, asset::{io::Reader, AssetLoader, AsyncReadExt, LoadContext}, core::Name, math::{Quat, Vec3}, utils::BoxedFuture
};
use studio_bvh::{joint, Joint};

use crate::anim::RetargetMap;

use super::{
    error::{Anyhow, Error},
    Bvh,
};

#[derive(Default)]
pub struct BvhLoader;

impl AssetLoader for BvhLoader {
    type Asset = AnimationClip;
    type Settings = ();
    type Error = Error;

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _settings: &'a Self::Settings,
        _cx: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut buf = vec![];
            reader.read_to_end(&mut buf).await.anyhow()?;
            let bvh = studio_bvh::from_bytes(&buf).anyhow()?;
            
            let mut anim_clip = AnimationClip::default();
            
            for (index, joint) in bvh.joints().enumerate() {
                let path = get_entity_path(&joint, &RetargetMap::default());
                // println!("{} - {:?}", index, path);
                let mut timestamps = Vec::new();
                let mut motions = Vec::new();
                let mut rotations = Vec::new();
                for (index, frame) in bvh.frames().enumerate() {
                    timestamps.push(index as f32 * bvh.frame_time().as_secs_f32());
                    let mut frame_pos = Vec3::ZERO;
                    let mut frame_rot = Vec3::ZERO;
                    for channel in joint.channels() {
                        let target = match channel.channel_type() {
                            studio_bvh::ChannelType::PositionX => &mut frame_pos.x,
                            studio_bvh::ChannelType::PositionY => &mut frame_pos.y,
                            studio_bvh::ChannelType::PositionZ => &mut frame_pos.z,
                            studio_bvh::ChannelType::RotationX => &mut frame_rot.x,
                            studio_bvh::ChannelType::RotationY => &mut frame_rot.y,
                            studio_bvh::ChannelType::RotationZ => &mut frame_rot.z,
                        };
                        
                        *target = frame[channel.motion_index()];
                    }
                    
                    motions.push(frame_pos);
                    rotations.push(Quat::from_euler(bevy::math::EulerRot::ZXY, frame_rot.x.to_radians(), frame_rot.y.to_radians(), frame_rot.z.to_radians()));
                }
                
                // if joint.is_root() {
                //     let motion_curve = VariableCurve {
                //         keyframe_timestamps: timestamps.clone(),
                //         keyframes: Keyframes::Translation(motions),
                //     };
                //     anim_clip.add_curve_to_path(path.clone(), motion_curve);
                // }
                
                let rotation_curve = VariableCurve {
                    keyframe_timestamps: timestamps,
                    keyframes: Keyframes::Rotation(rotations),
                };
                anim_clip.add_curve_to_path(path, rotation_curve);
            }

            Ok(anim_clip)
        })
    }

    fn extensions(&self) -> &[&str] {
        &["bvh"]
    }
}

pub fn get_entity_path(current_joint : &Joint, retarget_map : &RetargetMap) -> EntityPath {
     let name = String::from_utf8_lossy(current_joint.name()).to_string();
     let name = retarget_map.src_to_tgt.get(&name).unwrap_or(&name).clone();
     
    if let Some(parent) = current_joint.parent() {
        let mut path = get_entity_path(&parent, retarget_map);
        path.parts.push(Name::new(name));
        path
    } else {
        let mut path = EntityPath::default();
        path.parts.push("Mannequin_gamerig".into());
        path.parts.push(Name::new(name));
        path
    }
}
