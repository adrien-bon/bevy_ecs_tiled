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
use bevy::{prelude::*, render::sync_world::SyncToRenderWorld};

/// Marker [`Component`] for a Tiled map tile.
///
/// This component is attached to entities representing individual tiles in a Tiled map.
/// A tile entity is a child of its [`TiledLayer::Tiles`] entity.
///
/// Tile entities are the source of truth for the map content: you can freely spawn, despawn or modify tiles
/// ([`TiledTilesetId`], [`TiledTileId`], [`TilePos`], [`TileFlip`], ...) and the corresponding [`TiledTilemap`]
/// will be updated accordingly during [`TiledPostUpdateSystems::SyncRender`].
///
/// To spawn a new tile, you only need to insert this component along with a [`ChildOf`] pointing to
/// a [`TiledLayer::Tiles`] entity, a [`TiledTilesetId`], a [`TiledTileId`] and a [`TilePos`].
///
/// **Note:** Do not add [`Visibility`] or [`Transform`] to tile entities. Rendering is handled at the
/// [`TiledTilemap`] level via [`TilemapBundle`], and adding
/// these components to every tile entity can significantly degrade performance due to unnecessary
/// transform and visibility propagation.
///
/// See [`TileBundle`] for more information on available [`Component`]s.
#[derive(Component, Default, Reflect, Copy, Clone, Debug)]
#[reflect(Component, Default, Debug)]
#[require(
    TiledTileId,
    TiledTilesetId,
    TilePos,
    TilePosOld,
    TileFlip,
    TileVisible,
    TileColor,
    SyncToRenderWorld
)]
pub struct TiledTile;

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
#[require(Visibility, Transform, TiledName, TiledTilesetId)]
pub struct TiledTilemap;

/// Tiled tileset ID associated with this [`TiledTilemap`] or [`TiledTile`].
///
/// You can retrieve the corresponding [`tiled::Tileset`] using the [`tileset_from_map`] helper function.
#[derive(Component, Default, Reflect, Copy, Clone, Debug, PartialEq, Eq, Hash, Deref)]
#[reflect(Component, Default, Debug)]
pub struct TiledTilesetId(pub u32);

/// [`tiled::TileId`] associated with this [`TiledTile`].
///
/// You can retrieve the corresponding [`tiled::Tile`] using the [`tile_from_map`] helper function.
#[derive(Component, Default, Reflect, Copy, Clone, Debug, PartialEq, Eq, Hash, Deref)]
#[reflect(Component, Default, Debug)]
pub struct TiledTileId(pub tiled::TileId);

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
    app.register_type::<TiledTilesetId>();
    app.register_type::<TiledTileId>();
    app.register_type::<TiledTilemapOf>();
    app.register_type::<TiledTilemaps>();
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

#[cfg(feature = "render")]
mod render {
    use super::*;
    use crate::tiled::{
        event::TilemapCreated, helpers::tilemap_offset, map::asset::TiledMapTileset,
    };
    use bevy::platform::collections::HashMap;
    use bevy_ecs_tilemap::prelude::{TilemapBundle, TilemapSpacing};

    /// Where a [`TiledTile`] is currently registered for rendering.
    ///
    /// Used to remove the tile from its previous [`TileStorage`] when it is modified or removed.
    #[derive(Component, Copy, Clone, Debug)]
    pub(super) struct TiledTileRenderState {
        tilemap: Entity,
        position: TilePos,
    }

    /// A tilemap spawned during the current [`sync_tiles`] run
    struct NewTilemap {
        storage: TileStorage,
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

    /// Add new or modified tiles to the [`TileStorage`] of the corresponding [`TiledTilemap`],
    /// spawning this tilemap if needed.
    pub(super) fn sync_tiles(
        mut commands: Commands,
        tiles_query: Query<
            (
                Entity,
                Ref<TiledTilesetId>,
                Ref<TiledTileId>,
                &TilePos,
                Option<&ChildOf>,
                Option<&TiledTileRenderState>,
            ),
            (
                With<TiledTile>,
                Or<(
                    Added<TiledTile>,
                    Changed<TiledTilesetId>,
                    Changed<TiledTileId>,
                    Changed<TilePos>,
                )>,
            ),
        >,
        layers_query: Query<
            (
                &TiledMapReference,
                &TiledLayerId,
                &TiledName,
                Option<&TiledTilemaps>,
            ),
            With<TiledLayer>,
        >,
        mut tilemaps_query: Query<(&TiledTilesetId, &mut TileStorage), With<TiledTilemap>>,
        maps_query: Query<(&TiledMap, &TilemapRenderSettings, &TilemapAnchor)>,
        assets: Res<Assets<TiledMapAsset>>,
        mut message_writer: MessageWriter<TiledEvent<TilemapCreated>>,
    ) {
        // Tilemaps found or spawned during this run, by (layer, tileset ID)
        let mut tilemaps_cache = HashMap::<(Entity, u32), Entity>::default();
        let mut new_tilemaps = HashMap::<Entity, NewTilemap>::default();

        for (entity, tileset_id, tile_id, tile_pos, child_of, render_state) in tiles_query.iter() {
            // Nothing to do if only the TilePos was re-inserted (see below)
            if render_state.is_some_and(|state| state.position == *tile_pos)
                && !tileset_id.is_changed()
                && !tile_id.is_changed()
            {
                continue;
            }

            // First, remove the tile from its previous tilemap
            if let Some(state) = render_state
                && let Ok((_, mut storage)) = tilemaps_query.get_mut(state.tilemap)
                && storage.checked_get(&state.position) == Some(entity)
            {
                storage.remove(&state.position);
            }

            let Some((layer_entity, (map_ref, layer_id, layer_name, layer_tilemaps))) =
                child_of.and_then(|c| layers_query.get(c.parent()).ok().map(|l| (c.parent(), l)))
            else {
                warn!("Tile {entity} is not a child of a tiles layer, it will not be rendered");
                if render_state.is_some() {
                    detach_tile(&mut commands, entity, *tile_pos);
                }
                continue;
            };

            let Some((tiled_map, map_asset, render_settings, anchor)) = maps_query
                .get(map_ref.0)
                .ok()
                .and_then(|(m, r, a)| assets.get(&m.0).map(|asset| (m, asset, r, a)))
            else {
                // Map is not loaded (yet): nothing we can do
                continue;
            };

            let tileset = match map_asset.tilesets.get(&tileset_id.0) {
                Some(tileset) if tileset.usable_for_tiles_layer => tileset,
                _ => {
                    warn_once!(
                        "Tileset {} cannot be used in a tiles layer (missing tileset or images with different sizes), some tiles will not be rendered",
                        tileset_id.0
                    );
                    if render_state.is_some() {
                        detach_tile(&mut commands, entity, *tile_pos);
                    }
                    continue;
                }
            };

            if !tile_pos.within_map_bounds(&map_asset.tilemap_size) {
                warn!(
                    "Tile {entity} position {tile_pos:?} is outside of map bounds {:?}, it will not be rendered",
                    map_asset.tilemap_size
                );
                if render_state.is_some() {
                    detach_tile(&mut commands, entity, *tile_pos);
                }
                continue;
            }

            // Find the tilemap for this layer and tileset, or spawn a new one
            let tilemap_entity = *tilemaps_cache
                .entry((layer_entity, tileset_id.0))
                .or_insert_with(|| {
                    layer_tilemaps
                        .into_iter()
                        .flat_map(|tilemaps| tilemaps.iter())
                        .find(|e| {
                            tilemaps_query
                                .get(*e)
                                .is_ok_and(|(id, _)| id.0 == tileset_id.0)
                        })
                        .unwrap_or_else(|| {
                            let tilemap_entity = commands.spawn_empty().id();
                            let event = TiledEvent::new(tilemap_entity, TilemapCreated)
                                .with_map(map_ref.0, tiled_map.0.id())
                                .with_layer(layer_entity, layer_id.0)
                                .with_tilemap(tilemap_entity, tileset_id.0)
                                .to_owned();
                            commands.entity(tilemap_entity).insert((
                                Name::new(format!(
                                    "TiledTilemap({}, {})",
                                    layer_name.0, tileset.tileset.name
                                )),
                                *map_ref,
                                TiledTilemap,
                                *tileset_id,
                                TiledName(tileset.tileset.name.clone()),
                                ChildOf(layer_entity),
                                TiledTilemapOf(layer_entity),
                            ));
                            new_tilemaps.insert(
                                tilemap_entity,
                                NewTilemap {
                                    storage: TileStorage::empty(map_asset.tilemap_size),
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
                Some(new_tilemap) => new_tilemap.storage.set(tile_pos, entity),
                None => {
                    if let Ok((_, mut storage)) = tilemaps_query.get_mut(tilemap_entity) {
                        storage.set(tile_pos, entity);
                    }
                }
            }

            // Then update the tile rendering components
            let mut entity_commands = commands.entity(entity);
            if render_state
                .is_some_and(|state| state.tilemap != tilemap_entity || state.position != *tile_pos)
            {
                // The tile changed of tilemap or position: removing then re-inserting its TilePos
                // makes sure it is removed from its previous location in bevy_ecs_tilemap render
                // world before being added to the new one. Relying on bevy_ecs_tilemap TilePosOld
                // is not enough: it does not handle a tile changing of tilemap, nor several tiles
                // moving at the same time onto each other's previous position.
                entity_commands.remove::<TilePos>().insert(*tile_pos);
            }
            entity_commands.insert((
                TilemapId(tilemap_entity),
                tile_texture_index(tileset, tile_id.0),
                TiledTileRenderState {
                    tilemap: tilemap_entity,
                    position: *tile_pos,
                },
            ));
            match map_asset
                .tile(tileset_id.0, tile_id.0)
                .and_then(|tile| animated_tile(&tile))
            {
                Some(animated_tile) => entity_commands.insert(animated_tile),
                None => entity_commands.remove::<AnimatedTile>(),
            };
        }

        // Finally, insert the actual tilemap bundle for new tilemaps and send the associated events
        for (tilemap_entity, new_tilemap) in new_tilemaps {
            let NewTilemap {
                storage,
                mut bundle,
                event,
            } = new_tilemap;
            bundle.storage = storage;
            commands.entity(tilemap_entity).insert(bundle);
            event.send(&mut commands, &mut message_writer);
        }
    }

    /// Remove a tile from its [`TileStorage`] when it is despawned or when its [`TiledTile`] component is removed.
    pub(super) fn on_tile_removed(
        removed: On<Remove<TiledTile>>,
        mut commands: Commands,
        tiles_query: Query<(&TiledTileRenderState, &TilePos)>,
        mut tilemaps_query: Query<&mut TileStorage>,
    ) {
        let entity = removed.entity;
        let Ok((state, tile_pos)) = tiles_query.get(entity) else {
            return;
        };
        if let Ok(mut storage) = tilemaps_query.get_mut(state.tilemap)
            && storage.checked_get(&state.position) == Some(entity)
        {
            storage.remove(&state.position);
        }
        // If the entity is not despawned, it should not be rendered anymore
        detach_tile(&mut commands, entity, *tile_pos);
    }

    /// Despawn tilemaps which do not contain any tile anymore
    pub(super) fn despawn_empty_tilemaps(
        mut commands: Commands,
        tilemaps_query: Query<(Entity, &TileStorage), (With<TiledTilemap>, Changed<TileStorage>)>,
    ) {
        for (entity, storage) in tilemaps_query.iter() {
            if storage.iter().all(Option::is_none) {
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
