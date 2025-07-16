//! Collider management for Tiled maps and worlds.
//!
//! This module defines marker components and events for colliders generated from Tiled maps and objects.
//! It provides types to distinguish between colliders created from tile layers and object layers,
//! as well as utilities for extracting tile data relevant to collider generation.

use crate::prelude::*;
use bevy::prelude::*;

/// Marker component for colliders
///
/// Helps to distinguish between colliders created from Tiled objects and those created from Tiled tile layers.
#[derive(Component, Reflect, Copy, PartialEq, Clone, Debug)]
#[reflect(Component, Debug, PartialEq)]
#[require(Transform)]
pub enum TiledCollider {
    /// Collider is created by a [`tiled::TileLayer`] (ie. a collection of [`Tile`])
    TilesLayer,
    /// Collider is created by an [`tiled::Object`]
    Object,
}

/// Event emitted when a collider is created from a Tiled map or world.
///
/// You can determine collider origin using the inner [`TiledCollider`].
/// See also [`TiledEvent`]
#[derive(Clone, Copy, PartialEq, Debug, Reflect)]
#[reflect(Clone, PartialEq)]
pub struct ColliderCreated(pub TiledCollider);

pub(crate) fn plugin(app: &mut App) {
    app.register_type::<TiledCollider>();
    app.add_event::<TiledEvent<ColliderCreated>>()
        .register_type::<TiledEvent<ColliderCreated>>();
}

impl<'a> TiledEvent<ColliderCreated> {
    /// Returns a vector containing [`Tile`]s in this layer as well as their relative position from their parent [`crate::tiled::tile::TiledTilemap`] [`Entity``].
    pub fn get_tiles(
        &self,
        assets: &'a Res<Assets<TiledMapAsset>>,
        anchor: &TilemapAnchor,
    ) -> Vec<(Vec2, Tile<'a>)> {
        let Some(map_asset) = self.get_map_asset(assets) else {
            return vec![];
        };
        self.get_layer(assets)
            .and_then(|layer| layer.as_tile_layer())
            .map(|layer| {
                let mut out = vec![];
                map_asset.for_each_tile(&layer, |layer_tile, _, tile_pos, _| {
                    if let Some(tile) = layer_tile.get_tile() {
                        let grid_size = grid_size_from_map(&map_asset.map);
                        let tile_size = tile_size_from_grid_size(&grid_size);
                        let map_type = tilemap_type_from_map(&map_asset.map);
                        let tile_coords = tile_pos.center_in_world(
                            &map_asset.tilemap_size,
                            &grid_size,
                            &tile_size,
                            &map_type,
                            anchor,
                        );
                        out.push((tile_coords, tile));
                    }
                });
                out
            })
            .unwrap_or_default()
    }
}
