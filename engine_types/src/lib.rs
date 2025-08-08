use std::collections::HashMap;

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
