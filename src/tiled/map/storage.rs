//! Storage structures for Tiled map data.
//!
//! This module defines data structures and utilities for storing and managing the contents of a Tiled map,
//! including layers, tiles, and associated metadata. It provides efficient access and organization of map data
//! for use by systems and plugins within the bevy_ecs_tiled framework.

use crate::prelude::*;
use bevy::{platform::collections::HashMap, prelude::*};

/// [`Component`] storing all the Tiled items composing this map.
/// Makes the association between Tiled ID and corresponding Bevy [`Entity`].
///
/// Should not be manually inserted but can be accessed from the map [`Entity`].
///
/// It is kept up to date when layers, objects or tiles are added, modified or removed after the map
/// has been spawned, as long as these entities have a [`TiledMapReference`] to the map entity.
/// Note that new tiles are only registered during [`TiledPostUpdateSystems::SyncMapStorage`].
#[derive(Component, Default, Reflect, Clone, Debug)]
#[reflect(Component, Default, Debug)]
pub struct TiledMapStorage {
    /// Mapping between a Tiled layer ID with corresponding [`TiledLayer`] [`Entity`]
    pub(crate) layers: HashMap<u32, Entity>,

    /// Mapping between a Tiled object ID with corresponding [`TiledObject`] [`Entity`]
    pub(crate) objects: HashMap<u32, Entity>,

    /// Mapping between a Tiled tileset ID + [`tiled::TileId`] with all corresponding [`TiledTile`] [`Entity`]s
    ///
    /// Note that we can have multiple entities (ie.several instances) of the same tile since
    /// it references the tile on the tileset and not the tile on the tilemap.
    pub(crate) tiles: HashMap<(u32, tiled::TileId), Vec<Entity>>,
}

/// Where a [`TiledTile`] is registered in its map [`TiledMapStorage`], if any:
/// its tileset ID + [`tiled::TileId`] and its index in the corresponding list of entities.
///
/// Storing it on the tile itself allows removing a tile from the storage in constant time.
#[derive(Component, Default, Copy, Clone, Debug)]
pub(crate) struct TiledTileStorageIndex(pub(crate) Option<((u32, tiled::TileId), usize)>);

impl TiledMapStorage {
    /// Clear the [`TiledMapStorage`], removing all children layers in the process
    ///
    /// To respawn a map, insert the [`RespawnTiledMap`] component on the map entity instead.
    pub(crate) fn clear(&mut self, commands: &mut Commands) {
        for layer_entity in self.layers.values() {
            commands.entity(*layer_entity).try_despawn();
        }
        self.layers.clear();
        self.objects.clear();
        self.tiles.clear();
    }

    /// Index the next [`TiledTile`] [`Entity`] registered for this tileset ID + [`tiled::TileId`] will get
    pub(crate) fn next_tile_index(&self, key: (u32, tiled::TileId)) -> usize {
        self.tiles.get(&key).map_or(0, Vec::len)
    }

    /// Register a [`TiledTile`] [`Entity`] for the given tileset ID + [`tiled::TileId`],
    /// returning its index in the corresponding list of entities
    pub(crate) fn insert_tile(&mut self, key: (u32, tiled::TileId), entity: Entity) -> usize {
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
        key: (u32, tiled::TileId),
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

impl<'a> TiledMapStorage {
    /// Returns an iterator over the layer ID and [`TiledLayer`] [`Entity`] associations
    pub fn layers(&self) -> impl Iterator<Item = (u32, Entity)> + '_ {
        self.layers.iter().map(|(&id, &entity)| (id, entity))
    }

    /// Retrieve the [`TiledLayer`] [`Entity`] associated with this layer ID
    pub fn layer_entity(&self, layer_id: u32) -> Option<Entity> {
        self.layers.get(&layer_id).cloned()
    }

    /// Retrieve the layer ID associated with this [`TiledLayer`] [`Entity`]
    pub fn layer_id(&self, entity: Entity) -> Option<u32> {
        self.layers
            .iter()
            .find(|(_, e)| **e == entity)
            .map(|(&id, _)| id)
    }

    /// Retrieve the [`tiled::Layer`] associated with this [`TiledLayer`] [`Entity`]
    pub fn layer(&self, map: &'a tiled::Map, entity: Entity) -> Option<tiled::Layer<'a>> {
        self.layer_id(entity).and_then(|id| layer_from_map(map, id))
    }

    /// Returns an iterator over the tileset ID + [`tiled::TileId`] and [`TiledTile`] [`Entity`]s associations
    pub fn tiles(&self) -> impl Iterator<Item = ((u32, tiled::TileId), &[Entity])> + '_ {
        self.tiles
            .iter()
            .map(|(&id, entities)| (id, entities.as_slice()))
    }

    /// Retrieve the [`TiledTile`] [`Entity`] list associated with this tileset ID and [`tiled::TileId`]
    pub fn tile_entities(&self, tileset_id: u32, tile_id: tiled::TileId) -> &[Entity] {
        self.tiles
            .get(&(tileset_id, tile_id))
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    /// Retrieve the tileset ID and [`tiled::TileId`] associated with this [`TiledTile`] [`Entity`]
    ///
    /// This function iterates over all the tiles of the map: prefer reading the [`TiledTilesetId`]
    /// and [`TiledTileId`] components of the tile entity instead.
    pub fn tile_id(&self, entity: Entity) -> Option<(u32, tiled::TileId)> {
        self.tiles
            .iter()
            .find(|(_, entities)| entities.contains(&entity))
            .map(|(&id, _)| id)
    }

    /// Retrieve the [`tiled::Tile`] associated with this [`TiledTile`] [`Entity`]
    pub fn tile(&self, map: &'a tiled::Map, entity: Entity) -> Option<tiled::Tile<'a>> {
        self.tile_id(entity)
            .and_then(|(tileset_id, tile_id)| tile_from_map(map, tileset_id, tile_id))
    }

    /// Returns an iterator over the object ID and [`TiledObject`] [`Entity`] associations
    pub fn objects(&self) -> impl Iterator<Item = (u32, Entity)> + '_ {
        self.objects.iter().map(|(&id, &entity)| (id, entity))
    }

    /// Retrieve the [`TiledObject`] [`Entity`] associated with this object ID
    pub fn object_entity(&self, object_id: u32) -> Option<Entity> {
        self.objects.get(&object_id).cloned()
    }

    /// Retrieve the object ID associated with this [`TiledObject`] [`Entity`]
    pub fn object_id(&self, entity: Entity) -> Option<u32> {
        self.objects
            .iter()
            .find(|(_, e)| **e == entity)
            .map(|(&id, _)| id)
    }

    /// Retrieve the [`tiled::Object`] associated with this [`TiledObject`] [`Entity`]
    pub fn object(&self, map: &'a tiled::Map, entity: Entity) -> Option<tiled::Object<'a>> {
        self.object_id(entity)
            .and_then(|id| object_from_map(map, id))
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
    layers_query: Query<(&TiledLayerId, &TiledMapReference)>,
    mut maps_query: Query<&mut TiledMapStorage>,
) {
    let Ok((layer_id, map_ref)) = layers_query.get(added.entity) else {
        return;
    };
    if let Ok(mut storage) = maps_query.get_mut(map_ref.0) {
        storage.layers.entry(layer_id.0).or_insert(added.entity);
    }
}

fn unregister_layer(
    removed: On<Remove<TiledLayer>>,
    layers_query: Query<(&TiledLayerId, &TiledMapReference)>,
    mut maps_query: Query<&mut TiledMapStorage>,
) {
    let Ok((layer_id, map_ref)) = layers_query.get(removed.entity) else {
        return;
    };
    if let Ok(mut storage) = maps_query.get_mut(map_ref.0)
        && storage.layers.get(&layer_id.0) == Some(&removed.entity)
    {
        storage.layers.remove(&layer_id.0);
    }
}

fn register_object(
    added: On<Add<TiledObject>>,
    objects_query: Query<(&TiledObjectId, Option<&TiledMapReference>, Option<&ChildOf>)>,
    layers_query: Query<&TiledMapReference, With<TiledLayer>>,
    mut maps_query: Query<&mut TiledMapStorage>,
) {
    // The TiledMapReference may not have been inserted yet: use the one of the parent layer
    let Some((object_id, map_ref)) =
        objects_query
            .get(added.entity)
            .ok()
            .and_then(|(id, map_ref, child_of)| {
                super::resolve_map_reference(map_ref, child_of, &layers_query).map(|r| (id, r))
            })
    else {
        return;
    };
    if let Ok(mut storage) = maps_query.get_mut(map_ref.0) {
        storage.objects.entry(object_id.0).or_insert(added.entity);
    }
}

fn unregister_object(
    removed: On<Remove<TiledObject>>,
    objects_query: Query<(&TiledObjectId, &TiledMapReference)>,
    mut maps_query: Query<&mut TiledMapStorage>,
) {
    let Ok((object_id, map_ref)) = objects_query.get(removed.entity) else {
        return;
    };
    if let Ok(mut storage) = maps_query.get_mut(map_ref.0)
        && storage.objects.get(&object_id.0) == Some(&removed.entity)
    {
        storage.objects.remove(&object_id.0);
    }
}

/// Register new tiles (or tiles which changed of tileset ID or [`tiled::TileId`]) in their map storage.
fn register_tiles(
    tiles_query: Query<
        (Entity, &TiledTilesetId, &TiledTileId, &TiledMapReference),
        (
            With<TiledTile>,
            Or<(
                Added<TiledTile>,
                Changed<TiledTilesetId>,
                Changed<TiledTileId>,
            )>,
        ),
    >,
    mut indexes_query: Query<&mut TiledTileStorageIndex>,
    mut maps_query: Query<&mut TiledMapStorage>,
) {
    for (entity, tileset_id, tile_id, map_ref) in tiles_query.iter() {
        let key = (tileset_id.0, tile_id.0);
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
        storage.layers.insert(1, a);
        storage.objects.insert(7, b);
        storage.insert_tile((0, 3), b);
        storage.insert_tile((0, 3), c);

        assert_eq!(storage.layers().collect::<Vec<_>>(), vec![(1, a)]);
        assert_eq!(storage.objects().collect::<Vec<_>>(), vec![(7, b)]);
        assert_eq!(
            storage.tiles().collect::<Vec<_>>(),
            vec![((0, 3), [b, c].as_slice())]
        );
        assert_eq!(storage.tile_entities(0, 3), &[b, c]);
        assert!(storage.tile_entities(0, 4).is_empty());
        assert_eq!(storage.tile_id(c), Some((0, 3)));
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
        assert_eq!(storage.next_tile_index((0, 1)), 0);
        assert_eq!(storage.insert_tile((0, 1), a), 0);
        assert_eq!(storage.insert_tile((0, 1), b), 1);
        assert_eq!(storage.next_tile_index((0, 1)), 2);
        assert_eq!(storage.insert_tile((0, 1), c), 2);
        assert_eq!(storage.tile_entities(0, 1), &[a, b, c]);

        // Removing an entity moves the last one at its place
        assert_eq!(storage.remove_tile((0, 1), 0, a), Some(c));
        assert_eq!(storage.tile_entities(0, 1), &[c, b]);
        assert_eq!(storage.tile_id(a), None);

        // Removing an entity which is not registered at this index does nothing
        assert_eq!(storage.remove_tile((0, 1), 0, a), None);
        assert_eq!(storage.remove_tile((0, 1), 1, c), None);
        assert_eq!(storage.tile_entities(0, 1), &[c, b]);

        // Removing the last entity does not move any other one
        assert_eq!(storage.remove_tile((0, 1), 1, b), None);
        assert_eq!(storage.tile_entities(0, 1), &[c]);

        // Removing the last entity for a key removes the key
        assert_eq!(storage.remove_tile((0, 1), 0, c), None);
        assert_eq!(storage.tiles().count(), 0);
    }
}
