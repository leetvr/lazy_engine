mod component_registry;
pub mod components;
pub use component_registry::ComponentRegistry;
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

#[derive(Deserialize, Serialize, Clone, Default, Debug)]
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

#[derive(
    Deserialize, Serialize, Clone, Default, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug,
)]
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

pub trait CanYak {
    fn get_paint_fn() -> PaintFn;
}

pub type PaintFn = Box<dyn Fn(&hecs::World, hecs::Entity) + Send + Sync>;

pub struct EditorState<'a> {
    pub play_mode: &'a mut EditorPlayMode,
    pub world: &'a hecs::World,
    pub scene: &'a mut Scene,
    pub node_entity_map: &'a HashMap<NodeID, hecs::Entity>,
    pub loaded_prefabs: &'a HashMap<String, Prefab>,
    pub prefab_definitions: &'a HashMap<String, PrefabDefinition>,
    pub component_registry: &'a ComponentRegistry,
    pub engine_texture: yakui::TextureId,
    pub screen_size: yakui::Vec2,
    pub scale: f32,
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum EditorPlayMode {
    Play,
    Stop,
}

impl EditorPlayMode {
    pub fn flip(&mut self) {
        *self = match *self {
            EditorPlayMode::Play => EditorPlayMode::Stop,
            EditorPlayMode::Stop => EditorPlayMode::Play,
        }
    }
}

pub type GuiFn = Box<dyn Fn(&yakui::dom::Dom, EditorState) + Send + Sync>;
