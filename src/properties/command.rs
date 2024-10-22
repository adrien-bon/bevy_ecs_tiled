use super::load::DeserializedProperties;
use bevy::ecs::reflect::ReflectBundle;
use bevy::ecs::system::EntityCommands;
use bevy::ecs::world::Command;
use bevy::prelude::{AppTypeRegistry, Entity, ReflectComponent, ReflectResource, World};
use bevy::reflect::{Reflect, TypeRegistry};
use std::ops::Deref;

pub(crate) trait PropertiesCommandExt {
    fn insert_properties(&mut self, properties: DeserializedProperties) -> &mut Self;
}

impl PropertiesCommandExt for EntityCommands<'_> {
    fn insert_properties(&mut self, properties: DeserializedProperties) -> &mut Self {
        let entity = self.id();
        self.commands().add(InsertProperties { entity, properties });

        self
    }
}

pub(crate) struct InsertProperties {
    pub(crate) entity: Entity,
    pub(crate) properties: DeserializedProperties,
}

impl Command for InsertProperties {
    fn apply(self, world: &mut World) {
        let binding = world.get_resource::<AppTypeRegistry>().unwrap().clone();

        for property in self.properties.properties {
            insert_reflect(world, self.entity, binding.0.read().deref(), property);
        }
    }
}

/// Helper function to add a reflect component, bundle, or resource to a given entity
fn insert_reflect(
    world: &mut World,
    entity: Entity,
    type_registry: &TypeRegistry,
    property: Box<dyn Reflect>,
) {
    let type_info = property
        .get_represented_type_info()
        .expect("property should represent a type.");
    let type_path = type_info.type_path();
    let Some(type_registration) = type_registry.get_with_type_path(type_path) else {
        panic!("Could not get type registration (for property type {type_path}) because it doesn't exist in the TypeRegistry.");
    };

    if let Some(reflect_resource) = type_registration.data::<ReflectResource>() {
        reflect_resource.insert(world, &*property, type_registry);
        return;
    }

    let Some(mut entity) = world.get_entity_mut(entity) else {
        panic!("error[B0003]: Could not insert a reflected property (of type {type_path}) for entity {entity:?} because it doesn't exist in this World. See: https://bevyengine.org/learn/errors/#b0003");
    };

    if let Some(reflect_component) = type_registration.data::<ReflectComponent>() {
        reflect_component.insert(&mut entity, &*property, type_registry);
    } else if let Some(reflect_bundle) = type_registration.data::<ReflectBundle>() {
        reflect_bundle.insert(&mut entity, &*property, type_registry);
    } else {
        panic!("Could not get ReflectComponent data (for component type {type_path}) because it doesn't exist in this TypeRegistration.");
    }
}
