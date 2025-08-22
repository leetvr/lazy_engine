use crate::{AppState, GuiFn};
use engine_types::{EditorState, NodeID, PrefabInstance, components::Transform};
use hecs::Entity;
use std::{collections::HashMap, path::Path};
use yakui::expanded;

pub fn draw_gui(state: &mut AppState, scene_path: &Path) {
    use yakui::{
        Constraints, CrossAxisAlignment, MainAxisAlignment, Vec2, constrained, image, row,
        widgets::List,
    };

    let scale_factor = state.window.scale_factor();
    let window_size = state.window.inner_size().to_logical(scale_factor);

    state.yak.start();
    let half_screen_size = Vec2::new(window_size.width / 2.0, window_size.height);
    let constraints = Constraints::tight(half_screen_size);
    let mut scene_dirty = false;

    if unsafe { state.gui.check_and_reload(None) }.unwrap() {
        let get_gui: system_loader::Symbol<unsafe extern "C" fn() -> GuiFn> =
            unsafe { state.gui.lib.get(b"get_bonk_gui\0") }.unwrap();

        state.gui_fn = unsafe { get_gui() };
    }

    row(|| {
        constrained(constraints, || {
            let mut column = List::column();
            column.main_axis_alignment = MainAxisAlignment::Start;
            column.cross_axis_alignment = CrossAxisAlignment::Start;
            column.show(|| scene_dirty = gui_inner(state));
        });
        image(state.engine_texture, half_screen_size);
    });

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
    (state.gui_fn)(
        &state.yak.dom(),
        EditorState {
            play_mode: &mut state.play_state,
            world: &state.engine.world_mut(),
            scene: &mut state.scene,
            node_entity_map: &state.node_entity_map,
            loaded_prefabs: &state.loaded_prefabs,
        },
        &state.component_registry,
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
