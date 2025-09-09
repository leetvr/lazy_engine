use crate::{AppState, GuiFn};
use engine_types::{EditorState, NodeID, PrefabInstance, components::Transform};
use hecs::Entity;
use std::{collections::HashMap, path::Path};

pub fn draw_gui(state: &mut AppState, scene_path: &Path) {
    state.yak.start();

    if unsafe { state.gui.check_and_reload(None) }.unwrap() {
        let get_gui: system_loader::Symbol<unsafe extern "C" fn() -> GuiFn> =
            unsafe { state.gui.lib.get(b"get_bonk_gui\0") }.unwrap();

        state.gui_fn = unsafe { get_gui() };
    }

    let scene_dirty = gui_inner(state);

    let yak = &mut state.yak;
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

fn gui_inner(state: &mut AppState) -> bool {
    let screen_size = state.window.inner_size();
    let screen_size = [screen_size.width as f32, screen_size.height as f32];
    (state.gui_fn)(
        &state.yak.dom(),
        EditorState {
            play_mode: &mut state.play_state,
            world: &state.engine.world_mut(),
            scene: &mut state.scene,
            node_entity_map: &state.node_entity_map,
            loaded_prefabs: &state.loaded_prefabs,
            prefab_definitions: &state.prefab_definitions,
            component_registry: &state.component_registry,
            engine_texture: state.engine_texture,
            screen_size: screen_size.into(),
            scale: state.window.scale_factor() as _,
        },
    );
    false
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
