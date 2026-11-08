//! Raw Tiled custom properties.
//!
//! This module defines the [`TiledProperties`] component, which gives access to the class and custom properties
//! of a Tiled map, layer or object as they are defined in Tiled, whether the `user_properties` feature is enabled or not.

use crate::prelude::*;
use bevy::prelude::*;

/// Class and custom properties of a Tiled map, layer or object, as they are defined in Tiled.
///
/// This component is inserted on [`TiledMap`], [`TiledLayer`] and [`TiledObject`] entities when the map is spawned,
/// even if they do not have any class nor custom property.
/// It is not inserted on tiles: their properties are defined in their tileset, which you can access using
/// [`TiledMapAsset::tile`] with the tileset ID and tile ID of their [`TiledTile`] component.
///
/// Unlike properties deserialized with the `user_properties` feature, these are raw [`tiled::Properties`]:
/// object references are Tiled object IDs, not Bevy [`Entity`]s.
///
/// Since [`tiled::PropertyValue`] does not implement [`Reflect`], the `properties` field is ignored by reflection:
/// it is not visible in inspectors and not saved in scenes.
#[derive(Component, Default, Reflect, Clone, Debug)]
#[reflect(Component, Default, Debug)]
pub struct TiledProperties {
    /// Class (also called user type) of the Tiled item, or an empty string if it does not have one
    pub class: String,
    /// Custom properties of the Tiled item
    #[reflect(ignore)]
    pub properties: tiled::Properties,
}

pub(crate) fn plugin(app: &mut App) {
    app.register_type::<TiledProperties>();
}
