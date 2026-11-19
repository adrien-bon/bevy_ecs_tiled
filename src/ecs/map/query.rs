//! [`TiledMapQuery`]: query the layers, tiles or objects entities of a given map.

use crate::prelude::*;
use bevy::{
    ecs::{
        query::{IterQueryData, QueryData, QueryFilter, QueryItem, ROQueryItem},
        system::SystemParam,
    },
    prelude::*,
};

/// [`SystemParam`] querying the entities of a given map, through their [`TiledMapReference`].
///
/// It behaves like a [`Query`] whose iterators only yield the entities of the map passed as argument.
/// Use the [`TiledMapLayers`], [`TiledMapTiles`] and [`TiledMapObjects`] aliases to only query
/// layers, tiles or objects.
///
/// Entities are given in no particular order. Iterating over the entities of a map goes through the
/// matching entities of all the maps: this is fine for a few maps, but prefer a plain [`Query`] to process
/// the entities of all the maps at once.
///
/// To retrieve the tile at a given [`TilePos`], use the [`TiledTileLayerStorage`] of its layer instead.
///
/// Example:
/// ```rust,no_run
/// use bevy::prelude::*;
/// use bevy_ecs_tiled::prelude::*;
///
/// fn log_map_content(
///     map_created: On<TiledEvent<MapCreated>>,
///     layers: TiledMapLayers<&Name>,
///     tiles: TiledMapTiles<(&TilePos, &TiledTile)>,
///     objects: TiledMapObjects<Entity>,
/// ) {
///     let map = map_created.origin;
///     for name in layers.iter(map) {
///         info!("Layer: {name}");
///     }
///     for (position, tile) in tiles.iter(map) {
///         info!("Tile {:?} of tileset {:?} at {position:?}", tile.tile_id, tile.tileset_id);
///     }
///     info!("{} objects", objects.iter(map).count());
/// }
/// ```
#[derive(SystemParam)]
pub struct TiledMapQuery<'w, 's, D: QueryData + 'static, F: QueryFilter + 'static = ()> {
    query: Query<'w, 's, (D, &'static TiledMapReference), F>,
}

/// [`TiledMapQuery`] over the [`TiledLayer`] entities of a map.
pub type TiledMapLayers<'w, 's, D, F = ()> = TiledMapQuery<'w, 's, D, (With<TiledLayer>, F)>;

/// [`TiledMapQuery`] over the [`TiledTile`] entities of a map.
pub type TiledMapTiles<'w, 's, D, F = ()> = TiledMapQuery<'w, 's, D, (With<TiledTile>, F)>;

/// [`TiledMapQuery`] over the [`TiledObject`] entities of a map.
pub type TiledMapObjects<'w, 's, D, F = ()> = TiledMapQuery<'w, 's, D, (With<TiledObject>, F)>;

impl<D: QueryData, F: QueryFilter> TiledMapQuery<'_, '_, D, F> {
    /// Returns an iterator over the query items of the entities of the given map.
    pub fn iter(&self, map: Entity) -> impl Iterator<Item = ROQueryItem<'_, '_, D>> {
        self.query
            .iter()
            .filter_map(move |(item, map_ref)| (map_ref.0 == map).then_some(item))
    }
}

impl<D: IterQueryData, F: QueryFilter> TiledMapQuery<'_, '_, D, F> {
    /// Returns an iterator over the mutable query items of the entities of the given map.
    pub fn iter_mut(&mut self, map: Entity) -> impl Iterator<Item = QueryItem<'_, '_, D>> {
        self.query
            .iter_mut()
            .filter_map(move |(item, map_ref)| (map_ref.0 == map).then_some(item))
    }
}

impl<D: QueryData, F: QueryFilter> std::fmt::Debug for TiledMapQuery<'_, '_, D, F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TiledMapQuery").finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;

    #[test]
    fn map_query_only_yields_entities_of_the_map() {
        let mut world = World::new();
        let (map_a, map_b) = (world.spawn_empty().id(), world.spawn_empty().id());
        let layer_a = world
            .spawn((TiledLayer::Tiles, TiledMapReference(map_a)))
            .id();
        let tile_a = world
            .spawn((TiledTile::new(TilesetId(0), 1), TiledMapReference(map_a)))
            .id();
        let tile_b = world
            .spawn((TiledTile::new(TilesetId(0), 1), TiledMapReference(map_b)))
            .id();
        // Not a tile: must not be yielded by TiledMapTiles
        world.spawn(TiledMapReference(map_a));

        world
            .run_system_once(
                move |layers: TiledMapLayers<Entity>, tiles: TiledMapTiles<Entity>| {
                    assert_eq!(layers.iter(map_a).collect::<Vec<_>>(), vec![layer_a]);
                    assert_eq!(layers.iter(map_b).count(), 0);
                    assert_eq!(tiles.iter(map_a).collect::<Vec<_>>(), vec![tile_a]);
                    assert_eq!(tiles.iter(map_b).collect::<Vec<_>>(), vec![tile_b]);
                },
            )
            .unwrap();

        world
            .run_system_once(move |mut tiles: TiledMapTiles<&mut TiledTile>| {
                for mut tile in tiles.iter_mut(map_a) {
                    tile.tile_id = 2;
                }
            })
            .unwrap();
        assert_eq!(world.get::<TiledTile>(tile_a).unwrap().tile_id, 2);
        assert_eq!(world.get::<TiledTile>(tile_b).unwrap().tile_id, 1);
    }
}
