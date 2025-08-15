pub mod components;
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Default)]
pub struct Scene {
    pub prefabs: Vec<InstantiatedPrefab>,
}

#[derive(Deserialize, Serialize, Clone, Default)]
pub struct InstantiatedPrefab {
    pub name: String,
}

pub struct Prefab {
    pub name: String,
    pub nodes: Vec<PrefabNode>,
}

pub struct PrefabNode {
    pub name: String,
    pub index: usize,
    pub builder: hecs::EntityBuilder,
    pub parent: Option<usize>,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct PrefabDefinition {
    pub name: String,
    pub components: HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub children: Vec<PrefabDefinition>,
}
