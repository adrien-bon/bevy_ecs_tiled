//! [`TiledWorldReference`] and [`TiledWorldMaps`]: the map entities of a world, by [`WorldMapId`].

use crate::prelude::*;
use bevy::{
    ecs::{
        query::{IterQueryData, QueryData, QueryFilter, QueryItem, ROQueryItem},
        system::SystemParam,
    },
    prelude::*,
};

/// [`Component`] of the [`TiledMap`] entities spawned by a [`TiledWorld`].
///
/// Gives the world a map belongs to and the [`WorldMapId`] of this map in the world.
/// Should not be manually inserted: maps of a world are spawned by its [`TiledWorldStreaming`].
#[derive(Component, Reflect, Copy, Clone, Debug, PartialEq, Eq)]
#[reflect(Component, Debug, PartialEq)]
pub struct TiledWorldReference {
    /// The [`TiledWorld`] entity
    pub world: Entity,
    /// ID of the map in the world
    pub map_id: WorldMapId,
}

/// [`SystemParam`] querying the map entities of a given world, through their [`TiledWorldReference`].
///
/// It behaves like a [`Query`] whose iterators only yield the maps of the world passed as argument.
/// Only the maps currently spawned are given: with [`TiledWorldStreaming`], maps far from cameras are despawned.
///
/// Maps are given in no particular order.
///
/// Example:
/// ```rust,no_run
/// use bevy::prelude::*;
/// use bevy_ecs_tiled::prelude::*;
///
/// fn log_world_maps(worlds: Query<Entity, With<TiledWorld>>, maps: TiledWorldMaps<&TiledWorldReference>) {
///     for world in worlds.iter() {
///         for map in maps.iter(world) {
///             info!("Map {} of world {world} is spawned", map.map_id);
///         }
///         if let Some(first_map) = maps.map_entity(world, WorldMapId(0)) {
///             info!("First map of world {world} is {first_map}");
///         }
///     }
/// }
/// ```
#[derive(SystemParam)]
pub struct TiledWorldMaps<'w, 's, D: QueryData + 'static, F: QueryFilter + 'static = ()> {
    query: Query<'w, 's, (Entity, D, &'static TiledWorldReference), F>,
}

impl<D: QueryData, F: QueryFilter> TiledWorldMaps<'_, '_, D, F> {
    /// Returns an iterator over the query items of the maps of the given world.
    pub fn iter(&self, world: Entity) -> impl Iterator<Item = ROQueryItem<'_, '_, D>> {
        self.query
            .iter()
            .filter_map(move |(_, item, world_map)| (world_map.world == world).then_some(item))
    }

    /// Returns the map entity of the given [`WorldMapId`] in the given world, if it is spawned.
    pub fn map_entity(&self, world: Entity, map_id: WorldMapId) -> Option<Entity> {
        self.query
            .iter()
            .find(|(_, _, world_map)| world_map.world == world && world_map.map_id == map_id)
            .map(|(entity, _, _)| entity)
    }
}

impl<D: IterQueryData, F: QueryFilter> TiledWorldMaps<'_, '_, D, F> {
    /// Returns an iterator over the mutable query items of the maps of the given world.
    pub fn iter_mut(&mut self, world: Entity) -> impl Iterator<Item = QueryItem<'_, '_, D>> {
        self.query
            .iter_mut()
            .filter_map(move |(_, item, world_map)| (world_map.world == world).then_some(item))
    }
}

impl<D: QueryData, F: QueryFilter> std::fmt::Debug for TiledWorldMaps<'_, '_, D, F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TiledWorldMaps").finish_non_exhaustive()
    }
}

pub(crate) fn plugin(app: &mut App) {
    app.register_type::<TiledWorldReference>();
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;

    #[test]
    fn world_maps_only_yields_maps_of_the_world() {
        let mut world = World::new();
        let (world_a, world_b) = (world.spawn_empty().id(), world.spawn_empty().id());
        let map_a0 = world
            .spawn(TiledWorldReference {
                world: world_a,
                map_id: WorldMapId(0),
            })
            .id();
        let map_a1 = world
            .spawn(TiledWorldReference {
                world: world_a,
                map_id: WorldMapId(1),
            })
            .id();
        let map_b0 = world
            .spawn(TiledWorldReference {
                world: world_b,
                map_id: WorldMapId(0),
            })
            .id();

        world
            .run_system_once(move |maps: TiledWorldMaps<Entity>| {
                let mut maps_a = maps.iter(world_a).collect::<Vec<_>>();
                maps_a.sort();
                let mut expected = vec![map_a0, map_a1];
                expected.sort();
                assert_eq!(maps_a, expected);
                assert_eq!(maps.iter(world_b).collect::<Vec<_>>(), vec![map_b0]);
                assert_eq!(maps.map_entity(world_a, WorldMapId(1)), Some(map_a1));
                assert_eq!(maps.map_entity(world_b, WorldMapId(0)), Some(map_b0));
                assert_eq!(maps.map_entity(world_b, WorldMapId(1)), None);
            })
            .unwrap();
    }
}
