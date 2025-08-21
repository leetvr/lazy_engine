use std::{any::TypeId, collections::HashMap};

use engine_types::{
    CanYak, PaintFn,
    components::{GLTFAsset, Transform},
};
use hecs::EntityBuilderClone;

type DeserialiseFn = Box<dyn Fn(&mut EntityBuilderClone, serde_json::Value) + Send + Sync>;

pub struct ComponentRegistry {
    deserialisers: HashMap<String, DeserialiseFn>,
    gui: HashMap<TypeId, PaintFn>,
    names: HashMap<TypeId, String>,
}

impl Default for ComponentRegistry {
    fn default() -> Self {
        let mut registry = ComponentRegistry {
            deserialisers: Default::default(),
            gui: Default::default(),
            names: Default::default(),
        };

        registry.register_component::<GLTFAsset>();
        registry.register_component::<Transform>();

        registry
    }
}

impl ComponentRegistry {
    pub fn register_component<Component>(&mut self)
    where
        Component: Send + Sync + serde::de::DeserializeOwned + 'static + Clone + CanYak,
    {
        // Ha ha! Ha ha ha! Yes!
        let name = std::any::type_name::<Component>()
            .split(":")
            .last()
            .unwrap()
            .to_string();

        self.deserialisers.insert(
            name.clone(),
            Box::new(move |builder, value| {
                let component: Component = serde_json::from_value(value).expect("Bad JSON");
                builder.add(component);
            }),
        );

        let type_id = TypeId::of::<Component>();
        self.gui.insert(type_id, Component::get_paint_fn());
        self.names.insert(type_id, name.clone());
    }

    pub fn add_component_to_builder(
        &self,
        component_name: impl AsRef<str>,
        component: serde_json::Value,
        entity_builder: &mut EntityBuilderClone,
    ) {
        let deserialiser = self.deserialisers.get(component_name.as_ref()).unwrap();
        deserialiser(entity_builder, component);
    }

    pub fn get_gui(&self, component_type_id: std::any::TypeId) -> Option<&PaintFn> {
        self.gui.get(&component_type_id)
    }

    pub fn get_name(&self, component_type_id: TypeId) -> Option<&String> {
        self.names.get(&component_type_id)
    }
}

#[cfg(test)]
mod tests {
    use engine_types::CanYak;

    use crate::ComponentRegistry;

    #[test]
    fn test_register() {
        #[derive(serde::Deserialize, serde::Serialize, Default, Clone, PartialEq, Debug)]
        struct MyComponent {
            a: usize,
            b: usize,
        }

        impl CanYak for MyComponent {
            fn get_paint_fn() -> engine_types::PaintFn {
                Box::new(|_, _| {})
            }
        }

        let mut registry = ComponentRegistry::default();
        registry.register_component::<MyComponent>();

        let mut entity_builder = hecs::EntityBuilderClone::new();
        let component = MyComponent { a: 42, b: 69 };
        registry.add_component_to_builder(
            "MyComponent",
            serde_json::to_value(component.clone()).unwrap(),
            &mut entity_builder,
        );

        let mut world = hecs::World::new();
        let entity = world.spawn(&entity_builder.build());

        let spawned_component = world.get::<&MyComponent>(entity).unwrap();
        assert_eq!(*spawned_component, component);
    }
}
