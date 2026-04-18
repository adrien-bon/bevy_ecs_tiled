//! ECS components for Tiled tiles.
//!
//! This module defines Bevy components used to represent Tiled tiles and tilemaps within the ECS world.
//! The [`TiledTile`] component marks individual tile entities, while the [`TiledTilemap`] component
//! is used to group and render collections of tiles as a single texture.

use crate::prelude::*;
use bevy::prelude::*;

/// Marker [`Component`] for a Tiled map tile.
///
/// This component is attached to entities representing individual tiles in a Tiled map.
/// **Note:** Do not add [`Visibility`] or [`Transform`] to tile entities. Rendering is handled at the
/// [`TiledTilemap`] level via [`TilemapBundle`], and adding
/// these components to every tile entity can significantly degrade performance due to unnecessary
/// transform and visibility propagation.
///
/// See [`TileBundle`] for more information on available [`Component`]s.
#[derive(Component, Default, Reflect, Copy, Clone, Debug)]
#[reflect(Component, Default, Debug)]
#[require(TiledTileId, TiledTilesetId)]
pub struct TiledTile;

/// Marker [`Component`] for a Tiled tilemap.
///
/// This component is used to group tiles together and render them as a single texture.
/// It is the parent of all [`TiledTile`] entities for a given layer and tileset combination.
/// Entities with this component also have [`Visibility`] and [`Transform`] components,
/// as they control the rendering and positioning of the entire tilemap.
///
/// See [`TilemapBundle`] for more information on available [`Component`]s.
#[derive(Component, Default, Reflect, Copy, Clone, Debug)]
#[reflect(Component, Default, Debug)]
#[require(Visibility, Transform, TiledName, TiledTilesetId)]
pub struct TiledTilemap;

/// Tiled tileset ID associated with this [`TiledTilemap`] or [`TiledTile`].
///
/// You can retrieve the corresponding [`tiled::Tileset`] using the [`get_tileset_from_map`] helper function.
#[derive(Component, Default, Reflect, Copy, Clone, Debug, Deref)]
#[reflect(Component, Default, Debug)]
pub struct TiledTilesetId(pub u32);

/// [`tiled::TileId`] associated with this [`TiledTile`].
///
/// You can retrieve the corresponding [`tiled::Tile`] using the [`get_tile_from_map`] helper function.
#[derive(Component, Default, Reflect, Copy, Clone, Debug, Deref)]
#[reflect(Component, Default, Debug)]
pub struct TiledTileId(pub tiled::TileId);

pub(crate) fn plugin(app: &mut App) {
    app.register_type::<TiledTile>();
    app.register_type::<TiledTilemap>();
    app.register_type::<TiledTilesetId>();
    app.register_type::<TiledTileId>();
}
