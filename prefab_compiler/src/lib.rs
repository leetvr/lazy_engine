use component_registry::ComponentRegistry;
use engine_types::{Prefab, PrefabDefinition, PrefabNode};
use hecs::EntityBuilder;

pub fn compile(definition: &PrefabDefinition, component_registry: &ComponentRegistry) -> Prefab {
    let mut nodes = Vec::new();
    compile_node(definition, component_registry, &mut nodes, None);

    Prefab {
        name: definition.name.clone(),
        nodes,
    }
}

fn compile_node(
    definition: &PrefabDefinition,
    component_registry: &ComponentRegistry,
    nodes: &mut Vec<PrefabNode>,
    parent: Option<usize>,
) {
    let mut children = Vec::new();
    let my_index = nodes.len();

    let mut entity_builder = EntityBuilder::new();
    for (component_name, component) in &definition.components {
        component_registry.add_component_to_builder(
            component_name,
            component.clone(),
            &mut entity_builder,
        );
    }

    let node = PrefabNode {
        name: definition.name.clone(),
        index: my_index,
        builder: entity_builder,
        parent,
    };
    nodes.push(node);

    for child in &definition.children {
        let child_index = nodes.len();
        compile_node(child, component_registry, nodes, Some(my_index));
        children.push(child_index);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile() {
        #[derive(serde::Deserialize, serde::Serialize, Default, Clone, PartialEq, Debug)]
        struct FirstComponent {
            a: usize,
            b: usize,
        }

        #[derive(serde::Deserialize, serde::Serialize, Default, Clone, PartialEq, Debug)]
        struct NextComponent {
            an_array: Vec<String>,
        }

        let definition = serde_json::json!({
            "name":"root",
            "components": {
                "FirstComponent": {"a": 1, "b": 2},
                "NextComponent": {
                    "an_array": ["one", "two", "three"],
                },
            },
            "children": [{
                "name": "child",
                "components": {
                    "FirstComponent": {"a": 2, "b": 3},
                    "NextComponent": {
                        "an_array": ["four", "five", "six"],
                    },
                }
            }]
        });

        let definition = serde_json::from_value(definition).unwrap();
        let mut component_registry = ComponentRegistry::default();

        component_registry.register_component::<FirstComponent>();
        component_registry.register_component::<NextComponent>();

        let prefab = compile(&definition, &component_registry);
        assert_eq!(prefab.name, "root".to_string());
        assert_eq!(prefab.nodes[0].name, "root".to_string());
        assert_eq!(prefab.nodes[0].index, 0);
        assert_eq!(prefab.nodes[1].name, "child".to_string());
        assert_eq!(prefab.nodes[1].index, 1);
    }
}
