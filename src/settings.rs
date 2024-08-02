use bevy::prelude::*;

use crate::names::ObjectNames;

#[derive(Default, Clone, Component)]
pub struct TiledMapSettings {
    /// Specify which object layers to add collision shapes for.
    ///
    /// All shapes in these object layers will be added as collision shapes.
    pub collision_layer_names: ObjectNames,
    /// Specify which tileset object names to add collision shapes for.
    pub collision_object_names: ObjectNames,
    /// Specify which position transformation offset should be applied.
    ///
    /// By default, the layer's offset will be used.
    /// For Bevy's coordinate system use MapPositioning::Centered
    pub map_positioning: MapPositioning,
}

#[derive(Default, Clone)]
pub enum MapPositioning {
    #[default]
    /// Transforms TilemapBundle starting from the layer's offset.
    LayerOffset,
    /// Mimics Bevy's coordinate system so that (0, 0) is at the center of the map.
    Centered,
}
