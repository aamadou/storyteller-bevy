use bevy::{math::Affine3A, prelude::*};

#[derive(Component)]
pub struct SkeletalJoint(pub Color);

#[derive(Component)]
pub struct Skeleton;

#[derive(Component, Clone, Copy, Debug, Deref, DerefMut)]
pub struct BindPoseLocal(pub Transform);

#[derive(Component, Clone, Copy, Debug, Deref, DerefMut)]
pub struct BindPoseGlobal(pub Affine3A);

#[derive(Component, Clone, Copy, Debug, Deref, DerefMut)]
pub struct InvBindPose(pub Affine3A);
