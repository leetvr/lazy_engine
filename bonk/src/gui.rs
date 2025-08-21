use crate::{AppState, PlayState, spawn_prefab};
use engine_types::{NodeID, PrefabInstance, components::Transform};
use hecs::Entity;
use std::{collections::HashMap, path::Path};
use yakui::expanded;

pub fn draw_gui(state: &mut AppState, scene_path: &Path) {
    use yakui::{
        Constraints, CrossAxisAlignment, MainAxisAlignment, Vec2, button, constrained, image, row,
        text, widgets::List,
    };

    let world = &mut state.engine.world_mut();
    let scale_factor = state.window.scale_factor();
    let window_size = state.window.inner_size().to_logical(scale_factor);
    let yak = &mut state.yak;
    let loaded_prefabs = &mut state.loaded_prefabs;

    yak.start();
    let half_screen_size = Vec2::new(window_size.width / 2.0, window_size.height);
    let constraints = Constraints::tight(half_screen_size);
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
                        let entity = state.node_entity_map.get(&node.node_id).unwrap();
                        let entity_id = entity.id();
                        let label = format!(
                            "[index: {index}, node: {node_id}, entity: {entity_id}]",
                            node_id = node.node_id
                        );
                        text(20., label);

                        let entity_ref = world.entity(*entity).unwrap();
                        for component_type_id in entity_ref.component_types().into_iter() {
                            if let Some(name) = state.component_registry.get_name(component_type_id)
                            {
                                text(20., name.clone());
                            }

                            if let Some(gui) = state.component_registry.get_gui(component_type_id) {
                                gui(world, *entity)
                            }
                        }
                    }
                    if button("nudge right").clicked {
                        nudge(instantiated, world, &mut state.node_entity_map);
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
                            world,
                            &mut state.node_entity_map,
                        );
                        scene_dirty = true;
                    }
                }
                if match &state.play_state {
                    PlayState::Playing => button("Stop"),
                    PlayState::Stopped => button("Play"),
                }
                .clicked
                {
                    state.play_state.flip();
                }
            });
        });
        expanded(|| {
            image(state.engine_texture, half_screen_size);
        });
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
