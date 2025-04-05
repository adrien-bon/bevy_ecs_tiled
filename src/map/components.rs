//! This module contains all map [Component]s definition.

use bevy::{platform_support::collections::HashMap, prelude::*};
use tiled::TileId;

/// Specificy the Z offset between two consecutives Tiled layers.
///
/// Must be added to the [Entity] holding the map.
#[derive(Component, Reflect, Copy, Clone, Debug)]
#[reflect(Component, Default, Debug)]
pub struct TiledMapLayerZOffset(pub f32);

impl Default for TiledMapLayerZOffset {
    fn default() -> Self {
        Self(100.)
    }
}

/// Marker [Component] to trigger a map respawn.
///
/// Must be added to the [Entity] holding the map.
///
/// Example:
/// ```rust,no_run
/// use bevy::prelude::*;
/// use bevy_ecs_tiled::prelude::*;
///
/// fn handle_respawn(
///     mut commands: Commands,
///     map_query: Query<(Entity, &TiledMapHandle)>,
/// ) {
///     let (entity, _) = map_query.single();
///     commands.entity(entity).insert(RespawnTiledMap);
/// }
/// ```
#[derive(Component, Default, Reflect, Copy, Clone, Debug)]
#[reflect(Component, Default, Debug)]
pub struct RespawnTiledMap;

/// [Component] storing all the Tiled items composing this map.
/// Makes the association between Tiled ID and corresponding Bevy [Entity].
///
/// Should not be manually inserted but can be accessed from the map [Entity].
#[derive(Component, Default, Reflect, Clone, Debug)]
#[reflect(Component, Default, Debug)]
pub struct TiledMapStorage {
    /// Map of layers entities, using their Tiled ID as key
    pub layers: HashMap<u32, Entity>,
    /// Map of objects entities, using their Tiled ID as key
    pub objects: HashMap<u32, Entity>,
    /// Map of tiles entities, using the name of the tileset
    /// they belongs to + the tile ID in this tileset as key.
    /// Note that we can have multiple entities (several instances)
    /// of the same tile since it references the tile on the tileset
    /// and not the tile on the tilemap.
    pub tiles: HashMap<(String, TileId), Vec<Entity>>,
}

/// Marker [Component] for a Tiled map.
#[derive(Component, Default, Reflect, Copy, Clone, Debug)]
#[reflect(Component, Default, Debug)]
#[require(Visibility, Transform)]
pub struct TiledMapMarker;

/// Marker [Component] for a Tiled map layer.
#[derive(Component, Default, Reflect, Copy, Clone, Debug)]
#[reflect(Component, Default, Debug)]
#[require(Visibility, Transform)]
pub struct TiledMapLayer;

/// Marker [Component] for a Tiled map tile layer.
#[derive(Component, Default, Reflect, Copy, Clone, Debug)]
#[reflect(Component, Default, Debug)]
#[require(Visibility, Transform)]
pub struct TiledMapTileLayer;

/// Marker [Component] for a Tiled map tile layer for a given tileset.
#[derive(Component, Default, Reflect, Copy, Clone, Debug)]
#[reflect(Component, Default, Debug)]
#[require(Visibility, Transform)]
pub struct TiledMapTileLayerForTileset;

/// Marker [Component] for a Tiled map object layer.
#[derive(Component, Default, Reflect, Copy, Clone, Debug)]
#[reflect(Component, Default, Debug)]
#[require(Visibility, Transform)]
pub struct TiledMapObjectLayer;

/// Marker [Component] for a Tiled map group layer.
#[derive(Component, Default, Reflect, Copy, Clone, Debug)]
#[reflect(Component, Default, Debug)]
#[require(Visibility, Transform)]
pub struct TiledMapGroupLayer;

/// Marker [Component] for a Tiled map image layer.
#[derive(Component, Default, Reflect, Copy, Clone, Debug)]
#[reflect(Component, Default, Debug)]
#[require(Visibility, Transform)]
pub struct TiledMapImageLayer;

/// Marker [Component] for a Tiled map tile.
///
/// Note that this component does not require [Visibility] or [Transform]
/// It would be useless to add these components to tile entities:
/// - it will not do what you think: rendering is done at the [TiledMapTileLayerForTileset] level through `TilemapBundle` from `bevy_ecs_tilemap`
/// - it could impact performances pretty badly since it would mean to compute both [GlobalTransform] and [InheritedVisibility] for all tiles
#[derive(Component, Default, Reflect, Copy, Clone, Debug)]
#[reflect(Component, Default, Debug)]
pub struct TiledMapTile;

/// Marker [Component] for a Tiled map object.
#[derive(Component, Default, Reflect, Copy, Clone, Debug)]
#[reflect(Component, Default, Debug)]
#[require(Visibility, Transform)]
pub struct TiledMapObject;

/// Marker [Component] for the [Sprite] attached to an image layer.
#[derive(Component, Default, Reflect, Copy, Clone, Debug)]
#[reflect(Component, Default, Debug)]
#[require(Visibility, Transform)]
pub struct TiledMapImage;

/// This [Component] is used for animated objects.
/// We will automatically update the Sprite index every time the timer fires.
#[derive(Component, Default, Reflect, Clone, Debug)]
#[reflect(Component, Default, Debug)]
#[require(Visibility, Transform, Sprite)]
pub struct TiledAnimation {
    /// First index of the animation
    pub start: usize,
    /// First index after the animation
    pub end: usize,
    /// Timer firing every time we should update the frame
    pub timer: Timer,
}
