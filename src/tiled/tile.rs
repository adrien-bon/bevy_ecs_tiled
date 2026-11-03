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
use bevy::{platform::collections::HashMap, prelude::*, render::sync_world::SyncToRenderWorld};

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
#[derive(Component, Default, Reflect, Copy, Clone, Debug)]
#[reflect(Component, Default, Debug)]
#[require(
    TilePos,
    TilePosOld,
    TileFlip,
    TileVisible,
    TileColor,
    SyncToRenderWorld,
    TiledTileIndexedPosition,
    crate::tiled::map::storage::TiledTileStorageIndex
)]
pub struct TiledTile {
    /// Tileset of the tile, as its index in the map tilesets.
    ///
    /// You can retrieve the corresponding [`tiled::Tileset`] using the [`tileset_from_map`] helper function.
    pub tileset_id: u32,
    /// [`tiled::TileId`] of the tile in its tileset.
    ///
    /// You can retrieve the corresponding [`tiled::Tile`] using [`TiledMapAsset::tile`]
    /// or the [`tile_from_map`] helper function.
    pub tile_id: tiled::TileId,
}

impl TiledTile {
    /// Creates a new [`TiledTile`] from its tileset ID and [`tiled::TileId`]
    pub fn new(tileset_id: u32, tile_id: tiled::TileId) -> Self {
        Self {
            tileset_id,
            tile_id,
        }
    }
}

/// Which tile occupies each position of a [`TiledLayer::Tiles`].
///
/// Kept up to date when tiles are added, moved or removed.
#[derive(Component, Default, Debug)]
pub(crate) struct TiledLayerTiles(HashMap<TilePos, Entity>);

#[cfg(test)]
impl TiledLayerTiles {
    pub(crate) fn tiles(&self) -> &HashMap<TilePos, Entity> {
        &self.0
    }
}

/// Position under which a [`TiledTile`] is registered in its layer [`TiledLayerTiles`], if any.
#[derive(Component, Default, Copy, Clone, Debug)]
pub(crate) struct TiledTileIndexedPosition(Option<TilePos>);

/// Marker [`Component`] for a Tiled tilemap.
///
/// This component is used to group tiles together and render them as a single texture.
/// There is one tilemap for each layer and tileset combination: it is a sibling of the [`TiledTile`] entities
/// it renders, below the same [`TiledLayer::Tiles`] entity.
/// Entities with this component also have [`Visibility`] and [`Transform`] components,
/// as they control the rendering and positioning of the entire tilemap.
///
/// Tilemaps are rendering entities which are automatically created when needed and despawned
/// once they do not contain any tile anymore: you should not spawn them yourself.
///
/// See [`TilemapBundle`] for more information on available [`Component`]s.
#[derive(Component, Default, Reflect, Copy, Clone, Debug)]
#[reflect(Component, Default, Debug)]
#[require(Visibility, Transform, TiledName)]
pub struct TiledTilemap {
    /// Tileset of the tiles rendered by this tilemap, as its index in the map tilesets.
    ///
    /// You can retrieve the corresponding [`tiled::Tileset`] using the [`tileset_from_map`] helper function.
    pub tileset_id: u32,
}

/// Relationship [`Component`] between a [`TiledTilemap`] and its parent [`TiledLayer::Tiles`].
#[derive(Component, Reflect, Copy, Clone, Debug, Deref)]
#[reflect(Component, Debug)]
#[relationship(relationship_target = TiledTilemaps)]
pub struct TiledTilemapOf(pub Entity);

/// Relationship target [`Component`] pointing to all the [`TiledTilemap`]s of a [`TiledLayer::Tiles`].
#[derive(Component, Reflect, Debug, Deref)]
#[reflect(Component, Debug)]
#[relationship_target(relationship = TiledTilemapOf)]
pub struct TiledTilemaps(Vec<Entity>);

pub(crate) fn plugin(app: &mut App) {
    app.register_type::<TiledTile>();
    app.register_type::<TiledTilemap>();
    app.register_type::<TiledTilemapOf>();
    app.register_type::<TiledTilemaps>();
    app.add_systems(
        PostUpdate,
        index_layers_tiles.in_set(TiledPostUpdateSystems::SyncMapStorage),
    );
    app.add_observer(unindex_tile);
    #[cfg(feature = "render")]
    {
        app.add_systems(
            PostUpdate,
            (render::sync_tiles, render::despawn_empty_tilemaps)
                .chain()
                .in_set(TiledPostUpdateSystems::SyncRender),
        );
        app.add_observer(render::on_tile_removed);
    }
}

/// Register new or moved tiles in their layer [`TiledLayerTiles`].
///
/// If another tile already occupies the position, it is despawned.
fn index_layers_tiles(
    mut commands: Commands,
    mut tiles_query: Query<
        (Entity, &TilePos, &ChildOf, &mut TiledTileIndexedPosition),
        (With<TiledTile>, Or<(Added<TiledTile>, Changed<TilePos>)>),
    >,
    positions_query: Query<&TilePos, With<TiledTile>>,
    mut layers_query: Query<&mut TiledLayerTiles>,
) {
    // Tiles despawned during this run, because another tile moved onto them
    let mut despawned = bevy::platform::collections::HashSet::<Entity>::default();
    for (entity, tile_pos, child_of, mut indexed) in tiles_query.iter_mut() {
        if despawned.contains(&entity) {
            continue;
        }
        let Ok(mut layer_tiles) = layers_query.get_mut(child_of.parent()) else {
            continue;
        };

        // Remove the tile from its previous position
        if let Some(previous) = indexed.0
            && layer_tiles.0.get(&previous) == Some(&entity)
        {
            layer_tiles.0.remove(&previous);
        }

        // Replace the tile which was at this position, unless it moved away during this
        // frame (it will then be handled later on, when iterating over moved tiles)
        if let Some(&other) = layer_tiles.0.get(tile_pos)
            && other != entity
            && positions_query.get(other).is_ok_and(|pos| pos == tile_pos)
        {
            commands.entity(other).despawn();
            despawned.insert(other);
        }

        layer_tiles.0.insert(*tile_pos, entity);
        indexed.0 = Some(*tile_pos);
    }
}

/// Remove a tile from its layer [`TiledLayerTiles`] when it is despawned or when its [`TiledTile`] component is removed.
fn unindex_tile(
    removed: On<Remove<TiledTile>>,
    mut tiles_query: Query<(&ChildOf, &mut TiledTileIndexedPosition)>,
    mut layers_query: Query<&mut TiledLayerTiles>,
) {
    let entity = removed.entity;
    let Ok((child_of, mut indexed)) = tiles_query.get_mut(entity) else {
        return;
    };
    if let Some(position) = indexed.0.take()
        && let Ok(mut layer_tiles) = layers_query.get_mut(child_of.parent())
        && layer_tiles.0.get(&position) == Some(&entity)
    {
        layer_tiles.0.remove(&position);
    }
}

#[cfg(feature = "render")]
pub(crate) mod render {
    use super::*;
    use crate::tiled::{
        event::TilemapCreated, helpers::tilemap_offset, map::asset::TiledMapTileset,
    };
    use bevy::platform::collections::HashMap;
    use bevy_ecs_tilemap::prelude::{TilemapBundle, TilemapSpacing};

    /// Where a [`TiledTile`] is currently registered for rendering.
    ///
    /// Used to remove the tile from its previous [`TileStorage`] when it is modified or removed.
    /// `tilemap` is `None` if the tile cannot be rendered at this position (eg. outside of map bounds).
    #[derive(Component, Copy, Clone, Debug)]
    pub(crate) struct TiledTileRenderState {
        pub(crate) tilemap: Option<Entity>,
        pub(crate) position: TilePos,
    }

    /// Number of tiles in the [`TileStorage`] of a [`TiledTilemap`].
    ///
    /// Used to despawn the tilemap once it is empty without iterating over its whole storage.
    #[derive(Component, Default, Copy, Clone, Debug)]
    pub(crate) struct TiledTilemapTileCount(pub(crate) u32);

    /// Set a tile in a tilemap [`TileStorage`], updating the tilemap tile count
    fn storage_set(storage: &mut TileStorage, count: &mut u32, position: &TilePos, entity: Entity) {
        if storage.checked_get(position).is_none() {
            *count += 1;
        }
        storage.set(position, entity);
    }

    /// Remove a tile from a tilemap [`TileStorage`], updating the tilemap tile count
    fn storage_remove(storage: &mut TileStorage, count: &mut u32, position: &TilePos) {
        if storage.checked_remove(position).is_some() {
            *count -= 1;
        }
    }

    /// A tilemap spawned during the current [`sync_tiles`] run
    struct NewTilemap {
        storage: TileStorage,
        count: u32,
        bundle: TilemapBundle,
        event: TiledEvent<TilemapCreated>,
    }

    /// Detach a tile from its tilemap: it will not be rendered anymore
    fn detach_tile(commands: &mut Commands, entity: Entity, position: TilePos) {
        commands
            .entity(entity)
            .try_remove::<(
                TilemapId,
                TileTextureIndex,
                AnimatedTile,
                TiledTileRenderState,
            )>()
            // Removing then re-inserting the TilePos component removes
            // the tile from bevy_ecs_tilemap render world
            .try_remove::<TilePos>()
            .try_insert(position);
    }

    /// Mark a tile as not rendered at the given position, detaching it from its tilemap if needed.
    ///
    /// Keeping a render state for this position prevents [`sync_tiles`] from processing the tile
    /// again (and logging again why it cannot be rendered) until it is actually modified.
    fn mark_not_rendered(
        commands: &mut Commands,
        entity: Entity,
        position: TilePos,
        render_state: Option<&TiledTileRenderState>,
    ) {
        if render_state.is_some_and(|state| state.tilemap.is_some()) {
            detach_tile(commands, entity, position);
        }
        commands.entity(entity).insert(TiledTileRenderState {
            tilemap: None,
            position,
        });
    }

    /// Tile components read by [`sync_tiles`]
    type SyncedTile = (
        Entity,
        Ref<'static, TiledTile>,
        &'static TilePos,
        Option<&'static ChildOf>,
        Option<&'static TiledTileRenderState>,
        Has<AnimatedTile>,
    );

    /// Add new or modified tiles to the [`TileStorage`] of the corresponding [`TiledTilemap`],
    /// spawning this tilemap if needed.
    ///
    /// Tiles whose map asset is not available yet are processed again on the next runs.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn sync_tiles(
        mut commands: Commands,
        changed_tiles_query: Query<
            SyncedTile,
            (With<TiledTile>, Or<(Changed<TiledTile>, Changed<TilePos>)>),
        >,
        tiles_query: Query<SyncedTile, With<TiledTile>>,
        mut pending_tiles: Local<bevy::ecs::entity::EntityHashSet>,
        layers_query: Query<
            (&TiledMapReference, &TiledName, Option<&TiledTilemaps>),
            With<TiledLayer>,
        >,
        mut tilemaps_query: Query<
            (&TiledTilemap, &mut TileStorage, &mut TiledTilemapTileCount),
            With<TiledTilemap>,
        >,
        maps_query: Query<(&TiledMap, &TilemapRenderSettings, &TilemapAnchor)>,
        assets: Res<Assets<TiledMapAsset>>,
        mut message_writer: MessageWriter<TiledEvent<TilemapCreated>>,
        mut unusable_tilesets: Local<
            bevy::platform::collections::HashSet<(AssetId<TiledMapAsset>, u32)>,
        >,
    ) {
        // Tilemaps found or spawned during this run, by (layer, tileset ID)
        let mut tilemaps_cache = HashMap::<(Entity, u32), Entity>::default();
        let mut new_tilemaps = HashMap::<Entity, NewTilemap>::default();
        // Rendering components of tiles processed during this run, by (map, tileset ID, tile ID):
        // many tiles usually share the same tile, especially when spawning a map
        let mut tiles_cache = HashMap::<
            (Entity, u32, tiled::TileId),
            (TileTextureIndex, Option<AnimatedTile>),
        >::default();

        // Tiles which could not be processed during previous runs, and which were not modified since then
        let pending = std::mem::take(&mut *pending_tiles);
        let pending = pending
            .into_iter()
            .filter(|entity| !changed_tiles_query.contains(*entity))
            .filter_map(|entity| tiles_query.get(entity).ok());

        for (entity, tile, tile_pos, child_of, render_state, has_animation) in
            changed_tiles_query.iter().chain(pending)
        {
            // Nothing to do if only the TilePos was re-inserted (see below)
            if render_state.is_some_and(|state| state.position == *tile_pos) && !tile.is_changed() {
                continue;
            }

            // First, remove the tile from its previous tilemap
            if let Some(state) = render_state
                && let Some(tilemap) = state.tilemap
                && let Ok((_, mut storage, mut count)) = tilemaps_query.get_mut(tilemap)
                && storage.checked_get(&state.position) == Some(entity)
            {
                storage_remove(&mut storage, &mut count.0, &state.position);
            }

            let Some((layer_entity, (map_ref, layer_name, layer_tilemaps))) =
                child_of.and_then(|c| layers_query.get(c.parent()).ok().map(|l| (c.parent(), l)))
            else {
                warn!("Tile {entity} is not a child of a tiles layer, it will not be rendered");
                mark_not_rendered(&mut commands, entity, *tile_pos, render_state);
                continue;
            };

            let Some((tiled_map, map_asset, render_settings, anchor)) = maps_query
                .get(map_ref.0)
                .ok()
                .and_then(|(m, r, a)| assets.get(&m.0).map(|asset| (m, asset, r, a)))
            else {
                // Map asset is not available (yet): try again later
                pending_tiles.insert(entity);
                continue;
            };

            let tileset = match map_asset.tilesets.get(&tile.tileset_id) {
                Some(tileset) if tileset.usable_for_tiles_layer => tileset,
                _ => {
                    // Only warn once for each tileset of each map
                    if unusable_tilesets.insert((tiled_map.0.id(), tile.tileset_id)) {
                        warn!(
                            "Tileset {} of map {:?} cannot be used in a tiles layer (missing tileset or images with different sizes), its tiles will not be rendered",
                            tile.tileset_id, map_asset.map.source
                        );
                    }
                    mark_not_rendered(&mut commands, entity, *tile_pos, render_state);
                    continue;
                }
            };

            if !tile_pos.within_map_bounds(&map_asset.tilemap_size) {
                warn!(
                    "Tile {entity} position {tile_pos:?} is outside of map bounds {:?}, it will not be rendered",
                    map_asset.tilemap_size
                );
                mark_not_rendered(&mut commands, entity, *tile_pos, render_state);
                continue;
            }

            // Find the tilemap for this layer and tileset, or spawn a new one
            let tilemap_entity = *tilemaps_cache
                .entry((layer_entity, tile.tileset_id))
                .or_insert_with(|| {
                    layer_tilemaps
                        .into_iter()
                        .flat_map(|tilemaps| tilemaps.iter())
                        .find(|e| {
                            tilemaps_query
                                .get(*e)
                                .is_ok_and(|(tilemap, _, _)| tilemap.tileset_id == tile.tileset_id)
                        })
                        .unwrap_or_else(|| {
                            let tilemap_entity = commands.spawn_empty().id();
                            let event = TiledEvent::new(tilemap_entity, TilemapCreated)
                                .with_map(map_ref.0, tiled_map.0.id())
                                .with_layer(layer_entity)
                                .with_tilemap(tilemap_entity, tile.tileset_id)
                                .to_owned();
                            commands.entity(tilemap_entity).insert((
                                Name::new(format!(
                                    "TiledTilemap({}, {})",
                                    layer_name.0, tileset.tileset.name
                                )),
                                *map_ref,
                                TiledTilemap {
                                    tileset_id: tile.tileset_id,
                                },
                                TiledName(tileset.tileset.name.clone()),
                                ChildOf(layer_entity),
                                TiledTilemapOf(layer_entity),
                            ));
                            new_tilemaps.insert(
                                tilemap_entity,
                                NewTilemap {
                                    storage: TileStorage::empty(map_asset.tilemap_size),
                                    count: 0,
                                    bundle: tilemap_bundle(
                                        map_asset,
                                        tileset,
                                        render_settings,
                                        anchor,
                                    ),
                                    event,
                                },
                            );
                            tilemap_entity
                        })
                });

            // Add the tile to the tilemap storage
            match new_tilemaps.get_mut(&tilemap_entity) {
                Some(new_tilemap) => storage_set(
                    &mut new_tilemap.storage,
                    &mut new_tilemap.count,
                    tile_pos,
                    entity,
                ),
                None => {
                    if let Ok((_, mut storage, mut count)) = tilemaps_query.get_mut(tilemap_entity)
                    {
                        storage_set(&mut storage, &mut count.0, tile_pos, entity);
                    }
                }
            }

            // Then update the tile rendering components
            let mut entity_commands = commands.entity(entity);
            if render_state.is_some_and(|state| {
                state.tilemap != Some(tilemap_entity) || state.position != *tile_pos
            }) {
                // The tile changed of tilemap or position: removing then re-inserting its TilePos
                // makes sure it is removed from its previous location in bevy_ecs_tilemap render
                // world before being added to the new one. Relying on bevy_ecs_tilemap TilePosOld
                // is not enough: it does not handle a tile changing of tilemap, nor several tiles
                // moving at the same time onto each other's previous position.
                // This relies on bevy_ecs_tilemap internals and cannot be tested without a renderer:
                // see the "Manual Checks" section of the contributing guide.
                entity_commands.remove::<TilePos>().insert(*tile_pos);
            }
            let (texture_index, animation) = *tiles_cache
                .entry((map_ref.0, tile.tileset_id, tile.tile_id))
                .or_insert_with(|| {
                    (
                        tile_texture_index(tileset, tile.tile_id),
                        map_asset
                            .tile(tile.tileset_id, tile.tile_id)
                            .and_then(|tile| animated_tile(&tile)),
                    )
                });
            entity_commands.insert((
                TilemapId(tilemap_entity),
                texture_index,
                TiledTileRenderState {
                    tilemap: Some(tilemap_entity),
                    position: *tile_pos,
                },
            ));
            match animation {
                Some(animation) => {
                    entity_commands.insert(animation);
                }
                None if has_animation => {
                    entity_commands.remove::<AnimatedTile>();
                }
                None => {}
            }
        }

        // Finally, insert the actual tilemap bundle for new tilemaps and send the associated events
        for (tilemap_entity, new_tilemap) in new_tilemaps {
            let NewTilemap {
                storage,
                count,
                mut bundle,
                event,
            } = new_tilemap;
            bundle.storage = storage;
            commands
                .entity(tilemap_entity)
                .insert((bundle, TiledTilemapTileCount(count)));
            event.send(&mut commands, &mut message_writer);
        }
    }

    /// Remove a tile from its [`TileStorage`] when it is despawned or when its [`TiledTile`] component is removed.
    pub(super) fn on_tile_removed(
        removed: On<Remove<TiledTile>>,
        mut commands: Commands,
        tiles_query: Query<(&TiledTileRenderState, &TilePos)>,
        mut tilemaps_query: Query<(&mut TileStorage, &mut TiledTilemapTileCount)>,
    ) {
        let entity = removed.entity;
        let Ok((state, tile_pos)) = tiles_query.get(entity) else {
            return;
        };
        if let Some(tilemap) = state.tilemap
            && let Ok((mut storage, mut count)) = tilemaps_query.get_mut(tilemap)
            && storage.checked_get(&state.position) == Some(entity)
        {
            storage_remove(&mut storage, &mut count.0, &state.position);
        }
        // If the entity is not despawned, it should not be rendered anymore
        detach_tile(&mut commands, entity, *tile_pos);
    }

    /// Despawn tilemaps which do not contain any tile anymore
    pub(super) fn despawn_empty_tilemaps(
        mut commands: Commands,
        tilemaps_query: Query<
            (Entity, &TiledTilemapTileCount),
            (With<TiledTilemap>, Changed<TiledTilemapTileCount>),
        >,
    ) {
        for (entity, count) in tilemaps_query.iter() {
            if count.0 == 0 {
                commands.entity(entity).despawn();
            }
        }
    }

    fn tilemap_bundle(
        map_asset: &TiledMapAsset,
        tileset: &TiledMapTileset,
        render_settings: &TilemapRenderSettings,
        anchor: &TilemapAnchor,
    ) -> TilemapBundle {
        let tiled_tileset = &tileset.tileset;
        TilemapBundle {
            grid_size: grid_size_from_map(&map_asset.map),
            size: map_asset.tilemap_size,
            texture: tileset.tilemap_texture.clone(),
            tile_size: TilemapTileSize {
                x: tiled_tileset.tile_width as f32,
                y: tiled_tileset.tile_height as f32,
            },
            spacing: TilemapSpacing {
                x: tiled_tileset.spacing as f32,
                y: tiled_tileset.spacing as f32,
            },
            transform: Transform::from_translation(
                tilemap_offset(&map_asset.map, tiled_tileset).extend(0.0),
            ),
            map_type: tilemap_type_from_map(&map_asset.map),
            render_settings: *render_settings,
            anchor: *anchor,
            ..default()
        }
    }

    fn tile_texture_index(tileset: &TiledMapTileset, tile_id: tiled::TileId) -> TileTextureIndex {
        TileTextureIndex(match &tileset.tilemap_texture {
            #[cfg(not(feature = "atlas"))]
            TilemapTexture::Vector(_) => tileset
                .tile_image_offsets
                .get(&tile_id)
                .copied()
                .unwrap_or_else(|| {
                    warn!(
                        "Missing image for tile {tile_id} in tileset '{}'",
                        tileset.tileset.name
                    );
                    0
                }),
            _ => tile_id,
        })
    }

    /// Tile-layer animations go through bevy_ecs_tilemap's `AnimatedTile`, which requires
    /// consecutive frame IDs and a uniform duration. Object-layer animations use `TiledAnimation`
    /// instead, which stores per-frame (index, duration) pairs and has no such constraints.
    pub(super) fn animated_tile(tile: &tiled::Tile) -> Option<AnimatedTile> {
        let animation_data = tile.animation.as_ref()?;
        let mut previous_tile_id = None;
        let first_tile = animation_data.iter().next()?;
        let last_tile = animation_data.iter().last()?;

        // Sanity checks: current limitations from bevy_ecs_tilemap
        for frame in animation_data {
            if frame.duration != first_tile.duration {
                log::warn!(
                    "Animated tile with non constant frame duration is currently not supported"
                );
                return None;
            }
            if let Some(id) = previous_tile_id
                && frame.tile_id != id + 1
            {
                log::warn!("Animated tile with non-aligned frame tiles is currently not supported");
                return None;
            }
            previous_tile_id = Some(frame.tile_id);
        }

        // duration is in ms and we want a 'frames per second' speed
        Some(AnimatedTile {
            start: first_tile.tile_id,
            end: last_tile.tile_id + 1,
            speed: 1000.
                / (first_tile.duration * (last_tile.tile_id - first_tile.tile_id + 1)) as f32,
        })
    }
}
