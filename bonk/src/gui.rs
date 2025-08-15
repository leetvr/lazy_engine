use std::{
    collections::HashMap,
    path::Path,
    sync::{
        LazyLock,
        atomic::{AtomicUsize, Ordering},
    },
};

use crate::AppState;
use engine_types::{
    InstanceID, InstanceNode, NodeID, Prefab, PrefabInstance, Scene, components::Transform,
};
use hecs::Entity;

pub fn draw_gui(state: &mut AppState, scene_path: &Path) {
    use yakui::{
        Constraints, CrossAxisAlignment, MainAxisAlignment, Vec2, button, constrained, row, spacer,
        text, widgets::List,
    };

    let scale_factor = state.window.scale_factor();
    let window_size = state.window.inner_size().to_logical(scale_factor);
    let yak = &mut state.render_state.yak;
    let loaded_prefabs = &mut state.loaded_prefabs;

    yak.start();
    let constraints = Constraints::tight(Vec2::new(window_size.width / 2.0, window_size.height));
    let mut scene_dirty = false;

    row(|| {
        constrained(constraints, || {
            let mut column = List::column();
            column.main_axis_alignment = MainAxisAlignment::Start;
            column.cross_axis_alignment = CrossAxisAlignment::Start;
            column.show(|| {
                text(40., "Prefabs in scene");
                for instantiated in &mut state.scene.instances {
                    let label = format!(
                        "[id: {}, prefab_name: {}]",
                        instantiated.instance_id, &instantiated.prefab
                    );
                    text(30., label);
                    text(30., "nodes:");
                    for (index, node) in &instantiated.nodes {
                        let entity_id = state.node_entity_map.get(&node.node_id).unwrap().id();
                        let label = format!(
                            "[index: {index}, node: {node_id}, entity: {entity_id}]",
                            node_id = node.node_id
                        );
                        text(20., label);
                    }
                    if button("nudge right").clicked {
                        nudge(
                            instantiated,
                            &mut state.render_state.world,
                            &mut state.node_entity_map,
                        );
                        scene_dirty = true;
                    }
                }

                text(40., "Available prefabs");
                for (name, prefab) in loaded_prefabs.iter_mut() {
                    if button(name.clone()).clicked {
                        spawn_prefab(
                            name,
                            prefab,
                            &mut state.scene,
                            &mut state.render_state.world,
                            &mut state.node_entity_map,
                        );
                        scene_dirty = true;
                    }
                }
            });
        });
        spacer(1);
    });

    yak.finish();
    yak.paint();

    if scene_dirty {
        std::fs::write(
            scene_path,
            serde_json::to_string_pretty(&state.scene).unwrap(),
        )
        .unwrap();
    }
}

fn nudge(
    prefab: &mut PrefabInstance,
    world: &mut hecs::World,
    node_entity_map: &HashMap<NodeID, Entity>,
) {
    // HAHAAHA! Hahaa! Ha! Yes!
    let position = prefab
        .nodes
        .get_mut(&0)
        .unwrap()
        .overrides
        .entry("Transform".to_string())
        .or_insert_with(|| serde_json::to_value(&Transform::default()).unwrap())
        .as_object_mut()
        .unwrap()
        .get_mut("position")
        .unwrap()
        .as_array_mut()
        .unwrap();

    // Get the current value
    let current_x = position[0].as_f64().unwrap();

    // Get the next value
    let next_x = current_x + 0.1;

    // Update it
    position[0] = serde_json::Value::from(next_x);

    let entity = node_entity_map
        .get(&prefab.nodes.get(&0).unwrap().node_id)
        .unwrap();
    world
        .entity(*entity)
        .unwrap()
        .get::<&mut Transform>()
        .unwrap()
        .position
        .x = next_x as f32;
}

fn spawn_prefab(
    name: &str,
    prefab: &mut Prefab,
    scene: &mut Scene,
    world: &mut hecs::World,
    node_entity_map: &mut HashMap<NodeID, Entity>,
) {
    let mut nodes = HashMap::new();
    for node in &mut prefab.nodes {
        let node_id = next_node_id();
        let entity = spawn_entity_for_node(world, node);
        node_entity_map.insert(node_id, entity);

        nodes.insert(
            node.index,
            InstanceNode {
                node_index: node.index,
                node_id,
                overrides: Default::default(),
            },
        );
    }

    let instance_id = NEXT_INSTANCE_ID.fetch_add(1, Ordering::Relaxed);

    scene.instances.push(PrefabInstance {
        instance_id: InstanceID::new(instance_id),
        prefab: name.to_string(),
        nodes,
    })
}

pub fn spawn_entity_for_node(
    world: &mut hecs::World,
    node: &mut engine_types::PrefabNode,
) -> Entity {
    let entity = world.spawn(node.builder.build());
    world.insert_one(entity, Transform::default()).unwrap();
    entity
}

fn next_node_id() -> NodeID {
    NodeID::new(NEXT_NODE_ID.fetch_add(1, Ordering::Relaxed))
}

static NEXT_INSTANCE_ID: LazyLock<AtomicUsize> = LazyLock::new(|| AtomicUsize::new(0));
static NEXT_NODE_ID: LazyLock<AtomicUsize> = LazyLock::new(|| AtomicUsize::new(0));
