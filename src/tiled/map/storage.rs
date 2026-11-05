//! Storage structures for Tiled map data.
//!
//! This module defines data structures and utilities for storing and managing the contents of a Tiled map,
//! including layers, tiles, and associated metadata. It provides efficient access and organization of map data
//! for use by systems and plugins within the bevy_ecs_tiled framework.

use crate::prelude::*;
use bevy::{platform::collections::HashMap, prelude::*};

/// [`Component`] storing all the Tiled items composing this map.
///
/// Gives the [`TiledLayer`] and [`TiledObject`] entities of the map, and indexes its [`TiledTile`] entities
/// by [`TilesetId`] and [`tiled::TileId`], and by [`TilePos`].
///
/// Should not be manually inserted but can be accessed from the map [`Entity`].
///
/// It is kept up to date when layers, objects or tiles are added, modified or removed after the map
/// has been spawned, as long as these entities have a [`TiledMapReference`] to the map entity.
/// Note that new tiles are only registered during [`TiledPostUpdateSystems::SyncMapStorage`].
#[derive(Component, Default, Reflect, Clone, Debug)]
#[reflect(Component, Default, Debug)]
pub struct TiledMapStorage {
    /// [`TiledLayer`] entities, in the order they were registered
    pub(crate) layers: Vec<Entity>,

    /// [`TiledObject`] entities, in the order they were registered
    pub(crate) objects: Vec<Entity>,

    /// Mapping between a Tiled tileset ID + [`tiled::TileId`] with all corresponding [`TiledTile`] [`Entity`]s
    ///
    /// Note that we can have multiple entities (ie.several instances) of the same tile since
    /// it references the tile on the tileset and not the tile on the tilemap.
    pub(crate) tiles: HashMap<(TilesetId, tiled::TileId), Vec<Entity>>,

    /// Mapping between a [`TilePos`] and the [`TiledTile`] [`Entity`]s at this position, whatever their layer
    pub(crate) tiles_positions: HashMap<TilePos, Vec<Entity>>,
}

/// Where a [`TiledTile`] is registered in its map [`TiledMapStorage`], if any:
/// its tileset ID + [`tiled::TileId`] and its index in the corresponding list of entities.
///
/// Storing it on the tile itself allows removing a tile from the storage in constant time.
#[derive(Component, Default, Copy, Clone, Debug)]
pub(crate) struct TiledTileStorageIndex(pub(crate) Option<((TilesetId, tiled::TileId), usize)>);

impl TiledMapStorage {
    /// Clear the [`TiledMapStorage`], removing all children layers in the process
    ///
    /// To respawn a map, insert the [`RespawnTiledMap`] component on the map entity instead.
    pub(crate) fn clear(&mut self, commands: &mut Commands) {
        for layer_entity in self.layers.iter() {
            commands.entity(*layer_entity).try_despawn();
        }
        self.layers.clear();
        self.objects.clear();
        self.tiles.clear();
        self.tiles_positions.clear();
    }

    /// Register a [`TiledTile`] [`Entity`] at the given position
    pub(crate) fn insert_tile_position(&mut self, position: TilePos, entity: Entity) {
        self.tiles_positions
            .entry(position)
            .or_default()
            .push(entity);
    }

    /// Unregister a [`TiledTile`] [`Entity`] from the given position, if it is registered there
    pub(crate) fn remove_tile_position(&mut self, position: TilePos, entity: Entity) {
        let Some(entities) = self.tiles_positions.get_mut(&position) else {
            return;
        };
        // There is at most one tile per layer at a given position: the list is short
        if let Some(index) = entities.iter().position(|e| *e == entity) {
            entities.swap_remove(index);
        }
        if entities.is_empty() {
            self.tiles_positions.remove(&position);
        }
    }

    /// Index the next [`TiledTile`] [`Entity`] registered for this tileset ID + [`tiled::TileId`] will get
    pub(crate) fn next_tile_index(&self, key: (TilesetId, tiled::TileId)) -> usize {
        self.tiles.get(&key).map_or(0, Vec::len)
    }

    /// Register a [`TiledTile`] [`Entity`] for the given tileset ID + [`tiled::TileId`],
    /// returning its index in the corresponding list of entities
    pub(crate) fn insert_tile(&mut self, key: (TilesetId, tiled::TileId), entity: Entity) -> usize {
        let entities = self.tiles.entry(key).or_default();
        entities.push(entity);
        entities.len() - 1
    }

    /// Unregister the [`TiledTile`] [`Entity`] registered at the given tileset ID + [`tiled::TileId`] and index.
    ///
    /// Nothing is done if this entity is not registered there anymore (eg. the storage was cleared).
    /// Otherwise, returns the [`Entity`] which took its place, if any: its index must be updated.
    pub(crate) fn remove_tile(
        &mut self,
        key: (TilesetId, tiled::TileId),
        index: usize,
        entity: Entity,
    ) -> Option<Entity> {
        let entities = self.tiles.get_mut(&key)?;
        if entities.get(index) != Some(&entity) {
            return None;
        }
        entities.swap_remove(index);
        let moved = entities.get(index).copied();
        if entities.is_empty() {
            self.tiles.remove(&key);
        }
        moved
    }
}

impl TiledMapStorage {
    /// Returns an iterator over the [`TiledLayer`] entities of the map.
    ///
    /// Layers spawned along with the map are given in the order they appear in the map
    /// (a group layer comes before its children); layers added afterwards come last.
    pub fn layers(&self) -> impl Iterator<Item = Entity> + '_ {
        self.layers.iter().copied()
    }

    /// Returns an iterator over the [`TiledObject`] entities of the map.
    ///
    /// Objects spawned along with the map are given in the order they appear in the map;
    /// objects added afterwards come last.
    pub fn objects(&self) -> impl Iterator<Item = Entity> + '_ {
        self.objects.iter().copied()
    }

    /// Returns an iterator over the tileset ID + [`tiled::TileId`] and [`TiledTile`] [`Entity`]s associations
    pub fn tiles(&self) -> impl Iterator<Item = ((TilesetId, tiled::TileId), &[Entity])> + '_ {
        self.tiles
            .iter()
            .map(|(&id, entities)| (id, entities.as_slice()))
    }

    /// Retrieve the [`TiledTile`] [`Entity`] list associated with this tileset ID and [`tiled::TileId`]
    pub fn tile_entities(&self, tileset_id: TilesetId, tile_id: tiled::TileId) -> &[Entity] {
        self.tiles
            .get(&(tileset_id, tile_id))
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    /// Retrieve the [`TiledTile`] entities at the given [`TilePos`], whatever their layer.
    ///
    /// There is at most one tile per layer at a given position: to find the tile of a given layer,
    /// check the [`ChildOf`] of the returned entities.
    /// Note that a [`TilePos`] is a position in the map grid: if layers have different offsets
    /// (or parallax factors), tiles at the same [`TilePos`] are not drawn at the same place.
    ///
    /// Tiles added or moved at runtime are registered during [`TiledPostUpdateSystems::SyncMapStorage`].
    pub fn tiles_at(&self, position: TilePos) -> &[Entity] {
        self.tiles_positions
            .get(&position)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }
}

/// Remove an [`Entity`] from a list, keeping the order of the other ones
fn remove_entity(entities: &mut Vec<Entity>, entity: Entity) {
    if let Some(index) = entities.iter().position(|e| *e == entity) {
        entities.remove(index);
    }
}

pub(crate) fn plugin(app: &mut App) {
    app.register_type::<TiledMapStorage>();
    app.add_systems(
        PostUpdate,
        register_tiles.in_set(TiledPostUpdateSystems::SyncMapStorage),
    );
    app.add_observer(register_layer);
    app.add_observer(unregister_layer);
    app.add_observer(register_object);
    app.add_observer(unregister_object);
    app.add_observer(unregister_tile);
}

fn register_layer(
    added: On<Add<TiledLayer>>,
    layers_query: Query<&TiledMapReference>,
    mut maps_query: Query<&mut TiledMapStorage>,
) {
    let Ok(map_ref) = layers_query.get(added.entity) else {
        return;
    };
    if let Ok(mut storage) = maps_query.get_mut(map_ref.0) {
        storage.layers.push(added.entity);
    }
}

fn unregister_layer(
    removed: On<Remove<TiledLayer>>,
    layers_query: Query<&TiledMapReference>,
    mut maps_query: Query<&mut TiledMapStorage>,
) {
    let Ok(map_ref) = layers_query.get(removed.entity) else {
        return;
    };
    if let Ok(mut storage) = maps_query.get_mut(map_ref.0) {
        remove_entity(&mut storage.layers, removed.entity);
    }
}

fn register_object(
    added: On<Add<TiledObject>>,
    objects_query: Query<(Option<&TiledMapReference>, Option<&ChildOf>)>,
    layers_query: Query<&TiledMapReference, With<TiledLayer>>,
    mut maps_query: Query<&mut TiledMapStorage>,
) {
    // The TiledMapReference may not have been inserted yet: use the one of the parent layer
    let Some(map_ref) = objects_query
        .get(added.entity)
        .ok()
        .and_then(|(map_ref, child_of)| {
            super::resolve_map_reference(map_ref, child_of, &layers_query)
        })
    else {
        return;
    };
    if let Ok(mut storage) = maps_query.get_mut(map_ref.0) {
        storage.objects.push(added.entity);
    }
}

fn unregister_object(
    removed: On<Remove<TiledObject>>,
    objects_query: Query<&TiledMapReference>,
    mut maps_query: Query<&mut TiledMapStorage>,
) {
    let Ok(map_ref) = objects_query.get(removed.entity) else {
        return;
    };
    if let Ok(mut storage) = maps_query.get_mut(map_ref.0) {
        remove_entity(&mut storage.objects, removed.entity);
    }
}

/// Register new tiles (or tiles which changed of tileset ID or [`tiled::TileId`]) in their map storage.
fn register_tiles(
    tiles_query: Query<(Entity, &TiledTile, &TiledMapReference), Changed<TiledTile>>,
    mut indexes_query: Query<&mut TiledTileStorageIndex>,
    mut maps_query: Query<&mut TiledMapStorage>,
) {
    for (entity, tile, map_ref) in tiles_query.iter() {
        let key = (tile.tileset_id, tile.tile_id);
        let Ok(mut storage) = maps_query.get_mut(map_ref.0) else {
            continue;
        };
        let Ok(previous) = indexes_query.get(entity).map(|index| index.0) else {
            continue;
        };

        match previous {
            // Already registered with this key (eg. tiles registered when spawning the map)
            Some((previous_key, _)) if previous_key == key => continue,
            // Registered with another key: unregister it first
            Some((previous_key, index)) => {
                if let Some(moved) = storage.remove_tile(previous_key, index, entity)
                    && let Ok(mut moved_index) = indexes_query.get_mut(moved)
                {
                    moved_index.0 = Some((previous_key, index));
                }
            }
            None => {}
        }

        let index = storage.insert_tile(key, entity);
        if let Ok(mut tile_index) = indexes_query.get_mut(entity) {
            tile_index.0 = Some((key, index));
        }
    }
}

fn unregister_tile(
    removed: On<Remove<TiledTile>>,
    mut tiles_query: Query<(Option<&TiledMapReference>, &mut TiledTileStorageIndex)>,
    mut maps_query: Query<&mut TiledMapStorage>,
) {
    let Ok((map_ref, mut tile_index)) = tiles_query.get_mut(removed.entity) else {
        return;
    };
    let (Some(map_ref), Some((key, index))) = (map_ref.copied(), tile_index.0.take()) else {
        return;
    };
    if let Ok(mut storage) = maps_query.get_mut(map_ref.0)
        && let Some(moved) = storage.remove_tile(key, index, removed.entity)
        && let Ok((_, mut moved_index)) = tiles_query.get_mut(moved)
    {
        moved_index.0 = Some((key, index));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn storage_accessors() {
        let mut world = World::new();
        let (a, b, c) = (
            world.spawn_empty().id(),
            world.spawn_empty().id(),
            world.spawn_empty().id(),
        );

        let mut storage = TiledMapStorage::default();
        storage.layers.extend([a, c]);
        storage.objects.extend([b, a, c]);
        storage.insert_tile((TilesetId(0), 3), b);
        storage.insert_tile((TilesetId(0), 3), c);

        assert_eq!(storage.layers().collect::<Vec<_>>(), vec![a, c]);
        assert_eq!(storage.objects().collect::<Vec<_>>(), vec![b, a, c]);
        assert_eq!(
            storage.tiles().collect::<Vec<_>>(),
            vec![((TilesetId(0), 3), [b, c].as_slice())]
        );
        assert_eq!(storage.tile_entities(TilesetId(0), 3), &[b, c]);
        assert!(storage.tile_entities(TilesetId(0), 4).is_empty());

        // Removing an entity keeps the order of the other ones
        remove_entity(&mut storage.objects, a);
        assert_eq!(storage.objects().collect::<Vec<_>>(), vec![b, c]);
        remove_entity(&mut storage.objects, a);
        assert_eq!(storage.objects().collect::<Vec<_>>(), vec![b, c]);
    }

    #[test]
    fn storage_tiles_insert_and_remove() {
        let mut world = World::new();
        let (a, b, c) = (
            world.spawn_empty().id(),
            world.spawn_empty().id(),
            world.spawn_empty().id(),
        );

        let mut storage = TiledMapStorage::default();
        assert_eq!(storage.next_tile_index((TilesetId(0), 1)), 0);
        assert_eq!(storage.insert_tile((TilesetId(0), 1), a), 0);
        assert_eq!(storage.insert_tile((TilesetId(0), 1), b), 1);
        assert_eq!(storage.next_tile_index((TilesetId(0), 1)), 2);
        assert_eq!(storage.insert_tile((TilesetId(0), 1), c), 2);
        assert_eq!(storage.tile_entities(TilesetId(0), 1), &[a, b, c]);

        // Removing an entity moves the last one at its place
        assert_eq!(storage.remove_tile((TilesetId(0), 1), 0, a), Some(c));
        assert_eq!(storage.tile_entities(TilesetId(0), 1), &[c, b]);

        // Removing an entity which is not registered at this index does nothing
        assert_eq!(storage.remove_tile((TilesetId(0), 1), 0, a), None);
        assert_eq!(storage.remove_tile((TilesetId(0), 1), 1, c), None);
        assert_eq!(storage.tile_entities(TilesetId(0), 1), &[c, b]);

        // Removing the last entity does not move any other one
        assert_eq!(storage.remove_tile((TilesetId(0), 1), 1, b), None);
        assert_eq!(storage.tile_entities(TilesetId(0), 1), &[c]);

        // Removing the last entity for a key removes the key
        assert_eq!(storage.remove_tile((TilesetId(0), 1), 0, c), None);
        assert_eq!(storage.tiles().count(), 0);
    }

    #[test]
    fn storage_tiles_positions() {
        let mut world = World::new();
        let (a, b) = (world.spawn_empty().id(), world.spawn_empty().id());
        let (p1, p2) = (TilePos::new(1, 2), TilePos::new(3, 4));

        let mut storage = TiledMapStorage::default();
        storage.insert_tile_position(p1, a);
        storage.insert_tile_position(p1, b);
        assert_eq!(storage.tiles_at(p1), &[a, b]);
        assert!(storage.tiles_at(p2).is_empty());

        // Removing an entity which is not at this position does nothing
        storage.remove_tile_position(p2, a);
        storage.remove_tile_position(p1, a);
        storage.remove_tile_position(p1, a);
        assert_eq!(storage.tiles_at(p1), &[b]);

        // Removing the last entity of a position removes the position
        storage.remove_tile_position(p1, b);
        assert!(storage.tiles_positions.is_empty());
    }
}
