pub mod components;
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Default)]
pub struct Scene {
    pub instances: Vec<PrefabInstance>,
}

#[derive(Deserialize, Serialize, Clone, Default)]
pub struct PrefabInstance {
    pub instance_id: InstanceID,
    pub prefab: String,
    pub nodes: HashMap<usize, InstanceNode>,
}

#[derive(Deserialize, Serialize, Clone, Default)]
pub struct InstanceNode {
    pub node_index: usize,
    pub node_id: NodeID,
    pub overrides: HashMap<String, serde_json::Value>,
}

#[derive(Deserialize, Serialize, Clone, Default, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InstanceID(usize);

impl InstanceID {
    pub fn new(id: usize) -> Self {
        Self(id)
    }
}

impl std::fmt::Display for InstanceID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl InstanceID {
    pub fn as_raw(&self) -> usize {
        self.0
    }
}

#[derive(Deserialize, Serialize, Clone, Default, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeID(usize);

impl NodeID {
    pub fn new(id: usize) -> Self {
        Self(id)
    }
}

impl NodeID {
    pub fn as_raw(&self) -> usize {
        self.0
    }
}

impl std::fmt::Display for NodeID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

pub struct Prefab {
    pub name: String,
    pub nodes: Vec<PrefabNode>,
}

pub struct PrefabNode {
    pub name: String,
    pub index: usize,
    pub builder: hecs::BuiltEntityClone,
    pub parent: Option<usize>,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct PrefabDefinition {
    pub name: String,
    pub components: HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub children: Vec<PrefabDefinition>,
}
