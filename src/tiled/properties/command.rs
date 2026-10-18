use std::ops::Deref;

use crate::tiled::properties::load::DeserializedProperties;
use bevy::{
    ecs::{reflect::ReflectBundle, system::EntityCommands},
    prelude::*,
    reflect::{PartialReflect, TypeRegistry},
};

pub(crate) trait PropertiesCommandExt {
    fn insert_properties(&mut self, properties: DeserializedProperties) -> &mut Self;
}

impl PropertiesCommandExt for EntityCommands<'_> {
    fn insert_properties(&mut self, properties: DeserializedProperties) -> &mut Self {
        let entity = self.id();
        self.commands()
            .queue(InsertProperties { entity, properties });

        self
    }
}

pub(crate) struct InsertProperties {
    pub(crate) entity: Entity,
    pub(crate) properties: DeserializedProperties,
}

impl Command for InsertProperties {
    type Out = ();

    fn apply(self, world: &mut World) -> Self::Out {
        let Some(binding) = world.get_resource::<AppTypeRegistry>().cloned() else {
            error!("Could not insert properties: AppTypeRegistry resource is missing");
            return;
        };

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
    property: Box<dyn PartialReflect>,
) {
    let Some(type_info) = property.get_represented_type_info() else {
        error!("Could not insert a property for entity {entity:?}: it does not represent any type");
        return;
    };
    let type_path = type_info.type_path();
    let Some(type_registration) = type_registry.get_with_type_path(type_path) else {
        error!("Could not get type registration (for property type {type_path}) because it doesn't exist in the TypeRegistry.");
        return;
    };

    let Ok(mut entity) = world.get_entity_mut(entity) else {
        // Entity may have been despawned before the command was applied (eg. map despawned right after spawn)
        warn!("Could not insert a reflected property (of type {type_path}) for entity {entity:?} because it doesn't exist in this World.");
        return;
    };

    if let Some(reflect_component) = type_registration.data::<ReflectComponent>() {
        reflect_component.insert(&mut entity, property.as_partial_reflect(), type_registry);
    } else if let Some(reflect_bundle) = type_registration.data::<ReflectBundle>() {
        reflect_bundle.insert(&mut entity, property.as_partial_reflect(), type_registry);
    } else {
        error!("Could not get ReflectComponent data (for component type {type_path}) because it doesn't exist in this TypeRegistration.");
    }
}
