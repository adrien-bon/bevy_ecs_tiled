//! This module contains all [Component]s definition.

use crate::prelude::*;
use bevy::{prelude::*, utils::HashMap};
use tiled::TileId;

/// [Component] holding Tiled related settings.
///
/// Controls various settings related to the way we handle the Tiled map.
/// Must be added to the [Entity] holding the map.
#[derive(Component, Copy, Clone)]
pub struct TiledMapSettings {
    /// Specify which layer positioning strategy should be applied to the map.
    pub layer_positioning: LayerPositioning,
    /// Z-offset between two consecutives layers.
    pub layer_z_offset: f32,
}

impl Default for TiledMapSettings {
    fn default() -> Self {
        Self {
            layer_positioning: LayerPositioning::default(),
            layer_z_offset: 100.,
        }
    }
}

/// Controls layers positioning strategy.
///
/// Based upon this setting, you can determine where your layers (ie. your map) will be rendered.
#[derive(Default, Copy, Clone)]
pub enum LayerPositioning {
    #[default]
    /// Do not tweak layers position and keep original Tiled coordinate system so that Bevy (0, 0) is at the bottom-left of the map.
    TiledOffset,
    /// Update layers position and mimic Bevy's coordinate system so that Bevy (0, 0) is at the center of the map.
    Centered,
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
#[derive(Component)]
pub struct RespawnTiledMap;

/// [Component] storing maps to navigate from Tiled ID to Bevy [Entity].
///
/// Should not be manually inserted but can be accessed from the map [Entity].
#[derive(Component, Default)]
pub struct TiledIdStorage {
    /// Map of layers entities, using their Tiled ID as key
    pub layers: HashMap<u32, Entity>,
    /// Map of objects entities, using their Tiled ID as key
    pub objects: HashMap<u32, Entity>,
    /// Map of tiles entities, using the name of the tileset
    /// they belongs to + the tile ID in this tileset as key.
    /// Note that we can have multiple entities (several instances)
    /// of the same tile.
    pub tiles: HashMap<(String, TileId), Vec<Entity>>,
}

/// Marker [Component] for a Tiled map.
#[derive(Component)]
#[require(Visibility, Transform)]
pub struct TiledMapMarker;

/// Marker [Component] for a Tiled map layer.
#[derive(Component)]
#[require(Visibility, Transform)]
pub struct TiledMapLayer {
    // Store the map id so that we can delete layers for this map later.
    // We don't want to store the handle as a [Component] because the parent
    // entity already has it and it complicates queries.
    pub map_handle_id: AssetId<TiledMap>,
}

/// Marker [Component] for a Tiled map tile layer.
#[derive(Component)]
#[require(Visibility, Transform)]
pub struct TiledMapTileLayer;

/// Marker [Component] for a Tiled map tile layer for a given tileset.
#[derive(Component)]
#[require(Visibility, Transform)]
pub struct TiledMapTileLayerForTileset;

/// Marker [Component] for a Tiled map object layer.
#[derive(Component)]
#[require(Visibility, Transform)]
pub struct TiledMapObjectLayer;

/// Marker [Component] for a Tiled map group layer.
#[derive(Component)]
#[require(Visibility, Transform)]
pub struct TiledMapGroupLayer;

/// Marker [Component] for a Tiled map image layer.
#[derive(Component)]
#[require(Visibility, Transform)]
pub struct TiledMapImageLayer;

/// Marker [Component] for a Tiled map tile.
///
/// Note that this component does not require [Visibility] or [Transform]
/// It would be useless to add these components to tile entities:
/// - it will not do what you think: rendering is done at the [TiledMapTileLayerForTileset] level through `TilemapBundle` from `bevy_ecs_tilemap`
/// - it could impact performances pretty badly since it would mean to compute both [GlobalTransform] and [InheritedVisibility] for all tiles
#[derive(Component)]
pub struct TiledMapTile;

/// Marker [Component] for a Tiled map object.
#[derive(Component)]
#[require(Visibility, Transform)]
pub struct TiledMapObject;

/// Marker [Component] for a Tiled image
#[derive(Component)]
#[require(Visibility, Transform)]
pub struct TiledMapImage;
