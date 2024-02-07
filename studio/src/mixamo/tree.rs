use bevy::{
    ecs::query::QueryEntityError,
    math::{Affine3A, Mat3A},
    prelude::*,
    utils::{EntityHashMap, HashMap},
};

use crate::anim::{BindPoseGlobal, BindPoseLocal, EntityPathExtras, SkeletalJoint};

#[derive(Resource)]
pub(super) struct RetargetTree {
    // pub output: AnimationClip,
    pub root: Entity,
    pub nodes: EntityHashMap<Entity, RetargetNode>,
    pub path_map: HashMap<EntityPath, Entity>,
    // pub timestamps: Vec<f32>,
}

#[derive(Clone)]
pub(super) struct RetargetNode {
    pub id: Entity,
    pub path: EntityPath,
    // pub curves: Vec<VariableCurve>,
    pub children: Vec<Entity>,
    pub parent: Option<Entity>,
    // pub transform: Affine3A,
}

impl RetargetTree {
    pub fn build(
        root: Entity,
        q_names: &Query<&Name>,
        q_children: &Query<&Children>,
        q_bind_poses: &Query<&BindPoseLocal>,
        q_joints: &Query<&SkeletalJoint>,
    ) -> Self {
        let mut result = Self {
            root,
            nodes: EntityHashMap::default(),
            path_map: HashMap::default(),
        };

        let root_path = EntityPath::new(q_names.get(root).unwrap().clone());
        let root_children = q_children
            .get(root)
            .unwrap()
            .iter()
            .copied()
            .filter(|ent| q_joints.contains(*ent))
            .collect();

        let root_node = RetargetNode {
            id: root,
            path: root_path.clone(),
            children: root_children,
            parent: None,
            // transform: Affine3A::IDENTITY,
        };

        result.nodes.insert(root, root_node);
        result.path_map.insert(root_path, root);

        result.populate(
            root, q_names, q_children, // q_bind_poses,
            q_joints,
        );

        result
    }

    // TODO: Use an actual iterator
    // FIXME: Going to need mutable access to nodes, probably through interior mutability
    #[inline]
    pub fn for_each<F>(&self, mut f: F)
    where
        F: FnMut(&RetargetNode),
    {
        self.walk(&self.nodes[&self.root], &mut f);
    }

    fn walk<F>(&self, node: &RetargetNode, f: &mut F)
    where
        F: FnMut(&RetargetNode),
    {
        f(node);

        for id in node.children.iter() {
            self.walk(&self.nodes[id], f);
        }
    }

    fn populate(
        &mut self,
        node_id: Entity,
        q_names: &Query<&Name>,
        q_children: &Query<&Children>,
        // q_bind_poses: &Query<&BindPoseLocal>,
        q_joints: &Query<&SkeletalJoint>,
    ) {
        let node = self.nodes[&node_id].clone();
        for &child_id in node.children.iter() {
            let name = q_names.get(child_id).unwrap();
            let path = node.path.append(name.clone());

            // let transform = match q_bind_poses.get(child_id) {
            //     Ok(bind_pose) => **bind_pose,
            //     Err(_) => panic!("BindPoseLocal not found for '{name}'"),
            // };

            // let transform = Affine3A {
            //     matrix3: Mat3A::from_quat(transform.rotation),
            //     translation: transform.translation.into(),
            // };

            let children = q_children
                .get(child_id)
                .map(|children| &**children)
                .unwrap_or_default()
                .iter()
                .copied()
                .filter(|ent| q_joints.contains(*ent))
                .collect();

            let child = RetargetNode {
                id: child_id,
                path: path.clone(),
                children,
                parent: Some(node_id),
                // transform,
            };

            self.nodes.insert(child_id, child);
            self.path_map.insert(path, child_id);

            self.populate(
                child_id, q_names, q_children, // q_bind_poses,
                q_joints,
            );
        }
    }
}

impl RetargetNode {
    pub fn compute_global_transform<Q: TransformQuery>(
        &self,
        q_parents: &Query<&Parent>,
        q_xforms: &Q,
    ) -> GlobalTransform {
        let xform = q_xforms.get_by_value(self.id).unwrap_or_default();
        let mut result = GlobalTransform::from(xform);

        if let Some(parent) = self.parent {
            for ancestor in q_parents.iter_ancestors(parent) {
                let xform = q_xforms.get_by_value(ancestor).unwrap_or_default();
                result = xform * result;
            }
        }

        result
    }
}

pub trait TransformQuery {
    fn get_by_value(&self, entity: Entity) -> Result<Transform, QueryEntityError>;
}

impl TransformQuery for Query<'_, '_, &Transform> {
    fn get_by_value(&self, entity: Entity) -> Result<Transform, QueryEntityError> {
        self.get(entity).copied()
    }
}

impl TransformQuery for Query<'_, '_, &mut Transform> {
    fn get_by_value(&self, entity: Entity) -> Result<Transform, QueryEntityError> {
        self.get(entity).copied()
    }
}
