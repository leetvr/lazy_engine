use engine_types::{ComponentRegistry, EditorState};
use yakui::Direction;
use yakui::MainAxisSize;
use yakui::image;
use yakui_shadcn::SidebarItem;
use yakui_shadcn::icons;
use yakui_shadcn::sidebar;

pub type GuiFn = Box<dyn Fn(&yakui::dom::Dom, EditorState) + Send + Sync>;

pub fn gui(dom: &yakui::dom::Dom, state: EditorState) {
    yakui::context::bind_dom(dom);

    gui_inner(state);

    yakui::context::unbind_dom();
}

fn gui_inner(state: EditorState) {
    let mut row = yakui::widgets::List::column();
    row.direction = Direction::Right;
    row.main_axis_size = MainAxisSize::Max;
    let mut sidebar_items = Vec::new();
    for instance in &state.scene.instances {
        let mut children = Vec::new();
        let prefab = state.prefab_definitions.get(&instance.prefab).unwrap();

        let mut components = Vec::new();
        for (name, value) in &prefab.components {
            let mut members = Vec::new();
            for (name, value) in value.as_object().unwrap() {
                members.push(SidebarItem::Item {
                    label: format!("{name}: {value}"),
                })
            }

            components.push(SidebarItem::Group {
                title: name.clone(),
                icon: "".into(),
                children: members,
            })
        }

        children.push(SidebarItem::Group {
            title: "Components".into(),
            icon: "".into(),
            children: components,
        });

        let mut root_children = Vec::new();
        root_children.push(SidebarItem::Group {
            title: format!("Prefab"),
            icon: icons::hammer(),
            children,
        });

        let instance_node = instance.nodes.get(&0).unwrap();
        let mut components = Vec::new();
        for (name, value) in &instance_node.overrides {
            let mut members = Vec::new();
            for (name, value) in value.as_object().unwrap() {
                members.push(SidebarItem::Item {
                    label: format!("{name}: {value}"),
                })
            }

            components.push(SidebarItem::Group {
                title: name.clone(),
                icon: "".into(),
                children: members,
            })
        }

        let mut children = Vec::new();
        children.push(SidebarItem::Group {
            title: "Overrides".into(),
            icon: "".into(),
            children: components,
        });

        root_children.push(SidebarItem::Group {
            title: format!("InstanceNode (#{})", instance_node.node_id),
            icon: icons::play(),
            children,
        });

        let mut children = Vec::new();
        let mut components = Vec::new();
        let entity = state.node_entity_map.get(&instance_node.node_id).unwrap();

        for name in prefab
            .components
            .keys()
            .chain(instance_node.overrides.keys())
        {
            let component =
                state
                    .component_registry
                    .get_component_as_value(name, state.world, *entity);

            let mut members = Vec::new();
            for (name, value) in component.as_object().unwrap() {
                members.push(SidebarItem::Item {
                    label: format!("{name}: {value}"),
                })
            }

            components.push(SidebarItem::Group {
                title: name.clone(),
                icon: "".into(),
                children: members,
            })
        }

        children.push(SidebarItem::Group {
            title: "Components".into(),
            icon: "".into(),
            children: components,
        });

        root_children.push(SidebarItem::Group {
            title: format!("Entity (#{})", entity.id()),
            icon: icons::play(),
            children,
        });

        sidebar_items.push(SidebarItem::Group {
            title: format!("{}#{}", prefab.name, instance.instance_id),
            icon: icons::hexagon(),
            children: root_children,
        });
    }

    row.show(|| {
        sidebar(format!("{} Bonk", icons::hammer()), &sidebar_items);
        image(
            state.engine_texture,
            [
                (state.screen_size.x / state.scale) - 256.0,
                state.screen_size.y / state.scale,
            ],
        );
    });
}

#[unsafe(no_mangle)]
pub fn get_bonk_gui() -> GuiFn {
    Box::new(gui)
}
