use bevy::{
    prelude::*,
    utils::{EntityHashMap, HashMap},
};

use crate::anim::{EntityPathExtras, SkeletalJoint};

#[derive(Resource)]
pub(super) struct RetargetTree {
    pub root: Entity,
    pub nodes: EntityHashMap<Entity, RetargetNode>,
    pub path_map: HashMap<EntityPath, Entity>,
}

#[derive(Clone)]
pub(super) struct RetargetNode {
    pub id: Entity,
    pub path: EntityPath,
    pub children: Vec<Entity>,
    pub parent: Option<Entity>,
}

impl RetargetTree {
    pub fn build(
        root: Entity,
        q_names: &Query<&Name>,
        q_children: &Query<&Children>,
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
        };

        result.nodes.insert(root, root_node);
        result.path_map.insert(root_path, root);

        result.populate(root, q_names, q_children, q_joints);

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
        q_joints: &Query<&SkeletalJoint>,
    ) {
        let node = self.nodes[&node_id].clone();
        for &child_id in node.children.iter() {
            let name = q_names.get(child_id).unwrap();
            let path = node.path.append(name.clone());

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
            };

            self.nodes.insert(child_id, child);
            self.path_map.insert(path, child_id);

            self.populate(child_id, q_names, q_children, q_joints);
        }
    }
}
