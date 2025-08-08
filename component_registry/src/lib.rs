use std::collections::HashMap;

use hecs::EntityBuilder;

type DeserialiseFn = Box<dyn Fn(&mut EntityBuilder, serde_json::Value) + Send + Sync>;

#[derive(Default)]
pub struct ComponentRegistry {
    deserialisers: HashMap<String, DeserialiseFn>,
}

impl ComponentRegistry {
    pub fn register_component<Component: Send + Sync + serde::de::DeserializeOwned + 'static>(
        &mut self,
    ) {
        // Ha ha! Ha ha ha! Yes!
        let name = std::any::type_name::<Component>()
            .split(":")
            .last()
            .unwrap()
            .to_string();
        self.deserialisers.insert(
            name,
            Box::new(move |builder, value| {
                let component: Component = serde_json::from_value(value).expect("Bad JSON");
                builder.add(component);
            }),
        );
    }

    pub fn add_component_to_builder(
        &self,
        component_name: impl AsRef<str>,
        component: serde_json::Value,
        entity_builder: &mut EntityBuilder,
    ) {
        let deserialiser = self.deserialisers.get(component_name.as_ref()).unwrap();
        deserialiser(entity_builder, component);
    }
}

#[cfg(test)]
mod tests {
    use crate::ComponentRegistry;

    #[test]
    fn test_register() {
        #[derive(serde::Deserialize, serde::Serialize, Default, Clone, PartialEq, Debug)]
        struct MyComponent {
            a: usize,
            b: usize,
        }

        let mut registry = ComponentRegistry::default();
        registry.register_component::<MyComponent>();

        let mut entity_builder = hecs::EntityBuilder::new();
        let component = MyComponent { a: 42, b: 69 };
        registry.add_component_to_builder(
            "MyComponent",
            serde_json::to_value(component.clone()).unwrap(),
            &mut entity_builder,
        );

        let mut world = hecs::World::new();
        let entity = world.spawn(entity_builder.build());

        let spawned_component = world.get::<&MyComponent>(entity).unwrap();
        assert_eq!(*spawned_component, component);
    }
}
