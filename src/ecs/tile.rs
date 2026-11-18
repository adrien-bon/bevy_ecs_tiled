//! ECS components for Tiled tiles.
//!
//! This module defines Bevy components used to represent Tiled tiles and tilemaps within the ECS world.
//! The [`TiledTile`] component marks individual tile entities, while the [`TiledTilemap`] component
//! is used to group and render collections of tiles as a single texture.
//!
//! Tile entities only hold Tiled data: which tile (tileset ID and [`tiled::TileId`]) is displayed at which
//! [`TilePos`]. Tilemaps are rendering entities derived from these data: they are automatically created,
//! updated and removed when tiles are added, modified or removed.

use crate::prelude::*;
use bevy::prelude::*;

/// [`Component`] for a Tiled map tile, giving which tile of which tileset it displays.
///
/// This component is attached to entities representing individual tiles in a Tiled map.
/// A tile entity is a child of its [`TiledLayer::Tiles`] entity.
///
/// Tile entities are the source of truth for the map content: you can freely spawn, despawn or modify tiles
/// ([`TiledTile`], [`TilePos`], [`TileFlip`], ...) and the corresponding [`TiledTilemap`]
/// will be updated accordingly during [`TiledPostUpdateSystems::SyncRender`].
///
/// To spawn a new tile, you only need to insert this component along with a [`ChildOf`] pointing to
/// a [`TiledLayer::Tiles`] entity and a [`TilePos`]. Note that [`TiledTile::default()`] is the first tile
/// (ID `0`) of the first tileset (ID `0`) of the map.
///
/// A position of a layer can only hold a single tile: spawning a tile, or moving a tile (ie. changing its
/// [`TilePos`]), onto a position already used by another tile of the same layer despawns this other tile,
/// whatever its tileset.
///
/// Some limitations apply:
/// - tiles outside of the map bounds (see [`TiledMapAsset::tilemap_size`]) are neither rendered nor used to
///   compute physics colliders: layers cannot grow,
/// - moving a tile to another layer (ie. changing its [`ChildOf`]) is not supported: despawn it and spawn a new one instead.
///
/// Tiles spawned along with their map only have a [`Name`] when the `debug` feature is enabled.
///
/// **Note:** Do not add [`Visibility`] or [`Transform`] to tile entities. Rendering is handled at the
/// [`TiledTilemap`] level via [`TilemapBundle`], and adding
/// these components to every tile entity can significantly degrade performance due to unnecessary
/// transform and visibility propagation.
///
/// Tile entities hold two kinds of components:
/// - Tiled data, which you can freely modify: [`TiledTile`], [`TilePos`], [`TileFlip`],
///   [`TileVisible`] and [`TileColor`]. A [`TiledMapReference`] is automatically inserted from the parent layer
///   if the tile does not have one.
/// - rendering components derived from these data, which you should not modify since they are overwritten
///   when the tile changes: [`TilemapId`], [`TileTextureIndex`] and [`AnimatedTile`].
#[derive(Component, Default, Reflect, Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[reflect(Component, Default, Debug, PartialEq, Hash)]
#[require(
    TilePos,
    TilePosOld,
    TileFlip,
    TileVisible,
    TileColor,
    TiledTileIndexedPosition,
    crate::ecs::map::storage::TiledTileStorageIndex
)]
pub struct TiledTile {
    /// [`TilesetId`] of the tile.
    ///
    /// You can retrieve the corresponding [`tiled::Tileset`] using [`TiledMapAsset::tileset`].
    pub tileset_id: TilesetId,
    /// [`tiled::TileId`] of the tile in its tileset.
    ///
    /// You can retrieve the corresponding [`tiled::Tile`] using [`TiledMapAsset::tile`].
    pub tile_id: tiled::TileId,
}

impl TiledTile {
    /// Creates a new [`TiledTile`] from its [`TilesetId`] and [`tiled::TileId`]
    pub fn new(tileset_id: TilesetId, tile_id: tiled::TileId) -> Self {
        Self {
            tileset_id,
            tile_id,
        }
    }
}

/// [`Message`] sent when a position of a tiles layer is vacated: its tile moved away or was removed.
///
/// Used to update what is derived from the tiles of a layer by position, such as physics colliders.
#[derive(Message, Clone, Copy, Debug)]
#[cfg_attr(not(feature = "physics"), allow(dead_code))]
pub(crate) struct TiledTilePositionVacated {
    /// The [`TiledLayer::Tiles`] entity
    pub(crate) layer: Entity,
    /// The position which does not hold the tile anymore
    pub(crate) position: TilePos,
}

/// Position under which a [`TiledTile`] is registered in the [`TiledTileLayerStorage`] of its layer, if any.
#[derive(Component, Default, Copy, Clone, Debug)]
pub(crate) struct TiledTileIndexedPosition(pub(crate) Option<TilePos>);

pub(crate) fn plugin(app: &mut App) {
    app.register_type::<TiledTile>();
    app.add_message::<TiledTilePositionVacated>();
    app.add_systems(
        PostUpdate,
        index_layers_tiles.in_set(TiledPostUpdateSystems::SyncMapStorage),
    );
    app.add_observer(unindex_tile);
}

/// Register new or moved tiles in the [`TiledTileLayerStorage`] of their layer.
///
/// If another tile already occupies the position, it is despawned.
fn index_layers_tiles(
    mut commands: Commands,
    mut tiles_query: Query<
        (Entity, &TilePos, &ChildOf, &mut TiledTileIndexedPosition),
        (With<TiledTile>, Or<(Added<TiledTile>, Changed<TilePos>)>),
    >,
    positions_query: Query<&TilePos, With<TiledTile>>,
    mut layers_query: Query<&mut TiledTileLayerStorage>,
    mut vacated: MessageWriter<TiledTilePositionVacated>,
) {
    for (entity, tile_pos, child_of, mut indexed) in tiles_query.iter_mut() {
        // Nothing to do if the tile is already indexed at its position
        // (eg. tiles indexed when spawning the map)
        if indexed.0 == Some(*tile_pos) {
            continue;
        }
        let Ok(mut storage) = layers_query.get_mut(child_of.parent()) else {
            continue;
        };

        // Remove the tile from its previous position
        if let Some(previous) = indexed.0 {
            storage.remove(previous, entity);
            vacated.write(TiledTilePositionVacated {
                layer: child_of.parent(),
                position: previous,
            });
        }

        // Replace the tile which was at this position, unless it moved away during this
        // frame (it will then be handled later on, when iterating over moved tiles)
        if let Some(other) = storage.set(*tile_pos, entity)
            && other != entity
            && positions_query.get(other).is_ok_and(|pos| pos == tile_pos)
        {
            commands.entity(other).try_despawn();
        }
        indexed.0 = Some(*tile_pos);
    }
}

/// Remove a tile from the [`TiledTileLayerStorage`] of its layer
/// when it is despawned or when its [`TiledTile`] component is removed.
fn unindex_tile(
    removed: On<Remove<TiledTile>>,
    mut tiles_query: Query<(Option<&ChildOf>, &mut TiledTileIndexedPosition)>,
    mut layers_query: Query<&mut TiledTileLayerStorage>,
    mut vacated: MessageWriter<TiledTilePositionVacated>,
) {
    let entity = removed.entity;
    let Ok((child_of, mut indexed)) = tiles_query.get_mut(entity) else {
        return;
    };
    let Some(position) = indexed.0.take() else {
        return;
    };
    if let Some(child_of) = child_of
        && let Ok(mut storage) = layers_query.get_mut(child_of.parent())
    {
        storage.remove(position, entity);
        vacated.write(TiledTilePositionVacated {
            layer: child_of.parent(),
            position,
        });
    }
}
