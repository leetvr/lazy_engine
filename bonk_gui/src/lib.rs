use component_registry::ComponentRegistry;
use engine_types::{EditorPlayMode, EditorState};
use yakui::Direction;
use yakui::MainAxisSize;
use yakui_shadcn::SidebarItem;
use yakui_shadcn::button;
use yakui_shadcn::icons;
use yakui_shadcn::sidebar;

pub type GuiFn = Box<dyn Fn(&yakui::dom::Dom, EditorState, &ComponentRegistry) + Send + Sync>;

pub fn gui(dom: &yakui::dom::Dom, state: EditorState, _registry: &ComponentRegistry) {
    yakui::context::bind_dom(dom);

    gui_inner(state);

    yakui::context::unbind_dom();
}

fn gui_inner(state: EditorState) {
    let mut column = yakui::widgets::List::column();
    column.direction = Direction::Right;
    column.main_axis_size = MainAxisSize::Max;
    column.show(|| {
        sidebar(
            format!("{} Bonk", icons::hammer()),
            &[
                SidebarItem::Group {
                    title: "OK".into(),
                    icon: icons::activity(),
                    children: vec![
                        SidebarItem::Item {
                            label: "This".into(),
                        },
                        SidebarItem::Item {
                            label: "The".into(),
                        },
                        SidebarItem::Item {
                            label: "Other".into(),
                        },
                    ],
                },
                SidebarItem::Group {
                    title: "OK".into(),
                    icon: icons::activity(),
                    children: vec![
                        SidebarItem::Item {
                            label: "This".into(),
                        },
                        SidebarItem::Item {
                            label: "The".into(),
                        },
                        SidebarItem::Item {
                            label: "Other".into(),
                        },
                    ],
                },
            ],
        );
        let label = match state.play_mode {
            EditorPlayMode::Play => format!("{} Stahp", icons::stop()),
            EditorPlayMode::Stop => format!("{} Play", icons::play()),
        };

        if button(label).clicked {
            state.play_mode.flip();
        }
    });
}

#[unsafe(no_mangle)]
pub fn get_bonk_gui() -> GuiFn {
    Box::new(gui)
}
