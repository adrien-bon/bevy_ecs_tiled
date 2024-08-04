use bevy::{ecs::system::EntityCommands, prelude::*};

use crate::names::ObjectNames;

pub type ColliderCallback = fn(&mut EntityCommands);

#[derive(Clone, Component)]
pub struct TiledMapSettings {
    /// Specify which object layers to add collision shapes for.
    ///
    /// All shapes in these object layers will be added as collision shapes.
    pub collision_layer_names: ObjectNames,
    /// Specify which tileset object names to add collision shapes for.
    pub collision_object_names: ObjectNames,
    /// By default, we add a single collider to the shape: you can use
    /// this callback to add additional components to the collider
    pub collider_callback: ColliderCallback,
    /// Specify which position transformation offset should be applied.
    ///
    /// By default, the layer's offset will be used.
    /// For Bevy's coordinate system use MapPositioning::Centered
    pub map_positioning: MapPositioning,
}

impl Default for TiledMapSettings {
    fn default() -> Self {
        Self {
            collider_callback: |_| {},
            collision_layer_names: ObjectNames::default(),
            collision_object_names: ObjectNames::default(),
            map_positioning: MapPositioning::default(),
        }
    }
}

#[derive(Default, Clone)]
pub enum MapPositioning {
    #[default]
    /// Transforms TilemapBundle starting from the layer's offset.
    LayerOffset,
    /// Mimics Bevy's coordinate system so that (0, 0) is at the center of the map.
    Centered,
}
