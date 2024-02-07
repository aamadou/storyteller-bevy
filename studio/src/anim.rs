use bevy::{math::Affine3A, prelude::*};

#[derive(Component)]
pub struct SkeletalJoint(pub Color);

#[derive(Component)]
pub struct Skeleton;

#[derive(Component, Clone, Copy, Debug, Default, Deref, DerefMut)]
pub struct BindPoseLocal(pub Transform);

#[derive(Component, Clone, Copy, Debug, Default, Deref, DerefMut)]
pub struct BindPoseGlobal(pub Affine3A);

#[derive(Component, Clone, Copy, Debug, Default, Deref, DerefMut)]
pub struct InvBindPose(pub Affine3A);

pub trait EntityPathExtras {
    fn parse(path: &str) -> Self;
    fn new(name: Name) -> Self;
    fn append(&self, name: Name) -> Self;
    fn leaf(&self) -> &Name;
}

impl EntityPathExtras for EntityPath {
    fn parse(path: &str) -> Self {
        Self {
            parts: path
                .split(" > ")
                .map(|part| Name::new(part.to_string()))
                .collect(),
        }
    }

    fn new(name: Name) -> Self {
        Self { parts: vec![name] }
    }

    fn append(&self, name: Name) -> Self {
        let mut parts = self.parts.clone();
        parts.push(name);

        Self { parts }
    }

    fn leaf(&self) -> &Name {
        self.parts.last().unwrap()
    }
}
