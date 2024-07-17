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
}
