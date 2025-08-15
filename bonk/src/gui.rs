use engine_types::{InstantiatedPrefab, Prefab, Scene};

use crate::AppState;

pub fn draw_gui(state: &mut AppState) {
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
    row(|| {
        constrained(constraints, || {
            let mut column = List::column();
            column.main_axis_alignment = MainAxisAlignment::Start;
            column.cross_axis_alignment = CrossAxisAlignment::Start;
            column.show(|| {
                text(40., "Prefabs in scene");
                for prefab in &state.scene.prefabs {
                    text(30., prefab.name.clone());
                }

                text(40., "Prefabs");
                for (name, prefab) in loaded_prefabs.iter_mut() {
                    if button(name.clone()).clicked {
                        spawn_prefab(
                            name,
                            prefab,
                            &mut state.scene,
                            &mut state.render_state.world,
                        );
                    }
                }
            });
        });
        spacer(1);
    });

    yak.finish();
    yak.paint();
}

fn spawn_prefab(name: &str, prefab: &mut Prefab, scene: &mut Scene, world: &mut hecs::World) {
    for node in &mut prefab.nodes {
        world.spawn(node.builder.build());
    }

    scene.prefabs.push(InstantiatedPrefab {
        name: name.to_string(),
    })
}
