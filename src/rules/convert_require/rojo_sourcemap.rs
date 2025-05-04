use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::{utils, DarkluaError};

use super::InstancePath;

type NodeId = usize;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct RojoSourcemapNode {
    name: String,
    class_name: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    file_paths: Vec<PathBuf>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    children: Vec<RojoSourcemapNode>,
    #[serde(skip)]
    id: NodeId,
    #[serde(skip)]
    parent_id: NodeId,
}

impl RojoSourcemapNode {
    fn initialize(mut self, relative_to: &Path) -> Self {
        let mut queue = vec![&mut self];
        let mut index = 0;

        while let Some(node) = queue.pop() {
            node.id = index;
            for file_path in &mut node.file_paths {
                *file_path = utils::normalize_path(relative_to.join(&file_path));
            }
            for child in &mut node.children {
                child.parent_id = index;
                queue.push(child);
            }
            index += 1;
        }

        self
    }

    fn id(&self) -> NodeId {
        self.id
    }

    fn parent_id(&self) -> NodeId {
        self.parent_id
    }

    fn iter(&self) -> impl Iterator<Item = &Self> {
        RojoSourcemapNodeIterator::new(self)
    }
}

struct RojoSourcemapNodeIterator<'a> {
    queue: Vec<&'a RojoSourcemapNode>,
}

impl<'a> RojoSourcemapNodeIterator<'a> {
    fn new(root_node: &'a RojoSourcemapNode) -> Self {
        Self {
            queue: vec![root_node],
        }
    }
}

impl<'a> Iterator for RojoSourcemapNodeIterator<'a> {
    type Item = &'a RojoSourcemapNode;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(next_node) = self.queue.pop() {
            for child in &next_node.children {
                self.queue.push(child);
            }
            Some(next_node)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FlatNode {
    id: NodeId,
    parent_id: NodeId,
    name: String,
    children_ids: Vec<NodeId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RojoSourcemap {
    nodes: Vec<FlatNode>,
    file_map: HashMap<PathBuf, NodeId>,
    is_datamodel: bool,
}

impl RojoSourcemap {
    pub(crate) fn parse(
        content: &str,
        relative_to: impl AsRef<Path>,
    ) -> Result<Self, DarkluaError> {
        let root_node =
            serde_json::from_str::<RojoSourcemapNode>(content)?.initialize(relative_to.as_ref());

        let is_datamodel = root_node.class_name == "DataModel";

        let mut file_map: HashMap<PathBuf, NodeId> = HashMap::new();
        let mut nodes_flat: Vec<FlatNode> = root_node
            .iter()
            .map(|node| {
                let id = node.id();
                for file_path in &node.file_paths {
                    file_map.insert(file_path.clone(), id);
                }
                FlatNode {
                    id,
                    parent_id: node.parent_id(),
                    name: node.name.clone(),
                    children_ids: node.children.iter().map(|child| child.id()).collect(),
                }
            })
            .collect();
        nodes_flat.sort_unstable_by_key(|n| n.id);

        Ok(Self {
            nodes: nodes_flat,
            file_map,
            is_datamodel,
        })
    }

    pub(crate) fn get_instance_path(
        &self,
        from_file: impl AsRef<Path>,
        target_file: impl AsRef<Path>,
    ) -> Option<InstancePath> {
        let from = *self.file_map.get(from_file.as_ref())?;
        let target = *self.file_map.get(target_file.as_ref())?;

        let from_ancestors = self.hierarchy(from);
        let target_ancestors = self.hierarchy(target);

        let (parents, descendants, _) = from_ancestors
            .iter()
            .enumerate()
            .find_map(|(i, &ancestor_id)| {
                target_ancestors
                    .iter()
                    .enumerate()
                    .find(|&(_, &id)| id == ancestor_id)
                    .map(|(j, &id)| (i, j, id))
            })
            .map(|(i, j, common)| {
                (
                    from_ancestors.split_at(i).0,
                    target_ancestors.split_at(j).0,
                    common,
                )
            })?;

        let relative_path_length = parents.len().saturating_add(descendants.len());

        if !self.is_datamodel || relative_path_length <= target_ancestors.len() {
            let mut instance_path = InstancePath::from_script();
            for _ in 0..parents.len() {
                instance_path.parent();
            }
            self.index_descendants(instance_path, descendants.iter().rev())
        } else {
            let instance_path = InstancePath::from_root();
            self.index_descendants(instance_path, target_ancestors.iter().rev().skip(1))
        }
    }

    fn hierarchy(&self, mut node_id: NodeId) -> Vec<NodeId> {
        let mut ids = vec![node_id];
        while node_id != self.nodes[node_id].parent_id {
            node_id = self.nodes[node_id].parent_id;
            ids.push(node_id);
        }
        ids
    }

    fn index_descendants<'a>(
        &self,
        mut instance_path: InstancePath,
        descendants: impl Iterator<Item = &'a NodeId>,
    ) -> Option<InstancePath> {
        for &descendant_id in descendants {
            let node = &self.nodes[descendant_id];
            instance_path.child(&node.name);
        }
        Some(instance_path)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn new_sourcemap(content: &str) -> RojoSourcemap {
        RojoSourcemap::parse(content, "").expect("unable to parse sourcemap")
    }

    mod instance_paths {
        use super::*;

        fn script_path(components: &[&'static str]) -> InstancePath {
            components
                .iter()
                .fold(InstancePath::from_script(), |mut path, component| {
                    match *component {
                        "parent" => {
                            path.parent();
                        }
                        child_name => {
                            path.child(child_name);
                        }
                    }
                    path
                })
        }

        #[test]
        fn from_init_to_sibling_module() {
            let sourcemap = new_sourcemap(
                r#"{
                "name": "Project",
                "className": "ModuleScript",
                "filePaths": ["src/init.lua", "default.project.json"],
                "children": [
                    {
                        "name": "value",
                        "className": "ModuleScript",
                        "filePaths": ["src/value.lua"]
                    }
                ]
            }"#,
            );
            pretty_assertions::assert_eq!(
                sourcemap
                    .get_instance_path("src/init.lua", "src/value.lua")
                    .unwrap(),
                script_path(&["value"])
            );
        }

        #[test]
        fn from_sibling_to_sibling_module() {
            let sourcemap = new_sourcemap(
                r#"{
                "name": "Project",
                "className": "ModuleScript",
                "filePaths": ["src/init.lua", "default.project.json"],
                "children": [
                    {
                        "name": "main",
                        "className": "ModuleScript",
                        "filePaths": ["src/main.lua"]
                    },
                    {
                        "name": "value",
                        "className": "ModuleScript",
                        "filePaths": ["src/value.lua"]
                    }
                ]
            }"#,
            );
            pretty_assertions::assert_eq!(
                sourcemap
                    .get_instance_path("src/main.lua", "src/value.lua")
                    .unwrap(),
                script_path(&["parent", "value"])
            );
        }

        #[test]
        fn from_sibling_to_nested_sibling_module() {
            let sourcemap = new_sourcemap(
                r#"{
                "name": "Project",
                "className": "ModuleScript",
                "filePaths": ["src/init.lua", "default.project.json"],
                "children": [
                    {
                        "name": "main",
                        "className": "ModuleScript",
                        "filePaths": ["src/main.lua"]
                    },
                    {
                        "name": "Lib",
                        "className": "Folder",
                        "children": [
                            {
                                "name": "format",
                                "className": "ModuleScript",
                                "filePaths": ["src/Lib/format.lua"]
                            }
                        ]
                    }
                ]
            }"#,
            );
            pretty_assertions::assert_eq!(
                sourcemap
                    .get_instance_path("src/main.lua", "src/Lib/format.lua")
                    .unwrap(),
                script_path(&["parent", "Lib", "format"])
            );
        }

        #[test]
        fn from_child_require_parent() {
            let sourcemap = new_sourcemap(
                r#"{
                "name": "Project",
                "className": "ModuleScript",
                "filePaths": ["src/init.lua", "default.project.json"],
                "children": [
                    {
                        "name": "main",
                        "className": "ModuleScript",
                        "filePaths": ["src/main.lua"]
                    }
                ]
            }"#,
            );
            pretty_assertions::assert_eq!(
                sourcemap
                    .get_instance_path("src/main.lua", "src/init.lua")
                    .unwrap(),
                script_path(&["parent"])
            );
        }

        #[test]
        fn from_child_require_parent_nested() {
            let sourcemap = new_sourcemap(
                r#"{
                "name": "Project",
                "className": "ModuleScript",
                "filePaths": ["src/init.lua", "default.project.json"],
                "children": [
                    {
                        "name": "Sub",
                        "className": "ModuleScript",
                        "filePaths": ["src/Sub/init.lua"],
                        "children": [
                            {
                                "name": "test",
                                "className": "ModuleScript",
                                "filePaths": ["src/Sub/test.lua"]
                            }
                        ]
                    }
                ]
            }"#,
            );
            pretty_assertions::assert_eq!(
                sourcemap
                    .get_instance_path("src/Sub/test.lua", "src/Sub/init.lua")
                    .unwrap(),
                script_path(&["parent"])
            );
        }
    }
}
