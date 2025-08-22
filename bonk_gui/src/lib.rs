use component_registry::ComponentRegistry;
use engine_types::{EditorPlayMode, EditorState};
use yakui::{button, text};

pub type GuiFn = Box<dyn Fn(&yakui::dom::Dom, EditorState, &ComponentRegistry) + Send + Sync>;

pub fn gui(dom: &yakui::dom::Dom, state: EditorState, registry: &ComponentRegistry) {
    yakui::context::bind_dom(dom);
    let label = match state.play_mode {
        EditorPlayMode::Play => "Stop",
        EditorPlayMode::Stop => "Play",
    };

    if button(label).clicked {
        state.play_mode.flip();
    }

    yakui::context::unbind_dom();
}

#[unsafe(no_mangle)]
pub fn get_bonk_gui() -> GuiFn {
    Box::new(gui)
}
