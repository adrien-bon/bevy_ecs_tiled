//! Events related to Tiled map loading
//!
//! These events will be fired after the whole map has loaded.
//! More informations in the [dedicated example](https://github.com/adrien-bon/bevy_ecs_tiled/blob/main/examples/map_events.rs)

use std::fmt;

use crate::prelude::*;
use bevy::{ecs::system::SystemParam, prelude::*};
use bevy_ecs_tilemap::prelude::*;
use tiled::{Layer, LayerTile, Map, Object};

/// All event writers used when loading a map
#[derive(SystemParam)]
pub struct TiledMapEventWriters<'w> {
    /// Map events writer
    pub map_event: EventWriter<'w, TiledMapCreated>,
    /// Layer events writer
    pub layer_event: EventWriter<'w, TiledLayerCreated>,
    /// Object events writer
    pub object_event: EventWriter<'w, TiledObjectCreated>,
    /// Tile events writer
    pub tile_event: EventWriter<'w, TiledTileCreated>,
}

impl fmt::Debug for TiledMapEventWriters<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("TiledMapEventWriters").finish()
    }
}

/// Event sent when a map is spawned
#[derive(Event, Component, Reflect, Clone, Debug, Copy)]
#[event(auto_propagate, traversal = &'static ChildOf)]
#[reflect(Component, Debug)]
pub struct TiledMapCreated {
    /// Spawned map [Entity]
    pub entity: Entity,
    /// [AssetId] of the [TiledMap]
    pub asset_id: AssetId<TiledMap>,
}

impl<'a> TiledMapCreated {
    /// Retrieve the [TiledMap] associated with this [TiledMapCreated] event.
    pub fn get_map_asset(&self, map_asset: &'a Res<Assets<TiledMap>>) -> Option<&'a TiledMap> {
        map_asset.get(self.asset_id)
    }

    /// Retrieve the [Map] associated with this [TiledMapCreated] event.
    pub fn get_map(&self, map_asset: &'a Res<Assets<TiledMap>>) -> Option<&'a Map> {
        map_asset.get(self.asset_id).map(|m| &m.map)
    }
}

/// Event sent when a layer is spawned
#[derive(Event, Component, Reflect, Clone, Debug, Copy)]
#[event(auto_propagate, traversal = &'static ChildOf)]
#[reflect(Component, Debug)]
pub struct TiledLayerCreated {
    /// Creation event of the map this layer belongs to
    pub map: TiledMapCreated,
    /// Spawned layer [Entity]
    pub entity: Entity,
    /// ID of this layer in the [Map]
    pub id: usize,
}

impl<'a> TiledLayerCreated {
    /// Retrieve the [Layer] associated with this [TiledLayerCreated] event.
    pub fn get_layer(&self, map_asset: &'a Res<Assets<TiledMap>>) -> Option<Layer<'a>> {
        self.map
            .get_map(map_asset)
            .and_then(|m| m.get_layer(self.id))
    }
}

/// Event sent when an object is spawned
#[derive(Event, Component, Reflect, Clone, Debug, Copy)]
#[event(auto_propagate, traversal = &'static ChildOf)]
#[reflect(Component, Debug)]
pub struct TiledObjectCreated {
    /// Creation event of the layer this object belongs to
    pub layer: TiledLayerCreated,
    /// Spawned object [Entity]
    pub entity: Entity,
    /// ID of this object in the [tiled::ObjectLayer]
    pub id: usize,
}

impl<'a> TiledObjectCreated {
    /// Retrieve the [Object] associated with this [TiledObjectCreated] event.
    pub fn get_object(&self, map_asset: &'a Res<Assets<TiledMap>>) -> Option<Object<'a>> {
        self.layer
            .get_layer(map_asset)
            .and_then(|l| l.as_object_layer())
            .and_then(|l| l.get_object(self.id))
    }

    /// Retrieve object world position (origin = top left) relative to its parent layer.
    pub fn world_position(
        &self,
        map_asset: &'a Res<Assets<TiledMap>>,
        anchor: &TilemapAnchor,
    ) -> Option<Vec2> {
        self.layer
            .map
            .get_map_asset(map_asset)
            .and_then(|tiled_map| {
                self.get_object(map_asset).map(|object| {
                    from_tiled_position_to_world_space(
                        tiled_map,
                        anchor,
                        Vec2::new(object.x, object.y),
                    )
                })
            })
    }
}

/// Event sent when a tile has finished loading
///
/// This event is only sent for tiles which contain custom properties.
#[derive(Event, Component, Reflect, Clone, Debug, Copy)]
#[event(auto_propagate, traversal = &'static ChildOf)]
#[reflect(Component, Debug)]
pub struct TiledTileCreated {
    /// Creation event of the layer this tile belongs to
    pub layer: TiledLayerCreated,
    /// Spawned layer for tileset [Entity]
    /// Note this is different from the layer entity
    pub parent: Entity,
    /// Spawned tile [Entity]
    pub entity: Entity,
    /// Tile index (Tiled referential)
    pub index: IVec2,
    /// Tile position (bevy_ecs_tilemap referential)
    pub position: TilePos,
}

impl<'a> TiledTileCreated {
    /// Retrieve the [LayerTile] associated with this [TiledTileCreated] event.
    pub fn get_tile(&self, map_asset: &'a Res<Assets<TiledMap>>) -> Option<LayerTile<'a>> {
        self.layer
            .get_layer(map_asset)
            .and_then(|l| l.as_tile_layer())
            .and_then(|l| l.get_tile(self.index.x, self.index.y))
    }

    /// Retrieve tile world position (origin = tile center) relative to its parent layer.
    pub fn world_position(
        &self,
        map_asset: &'a Res<Assets<TiledMap>>,
        anchor: &TilemapAnchor,
    ) -> Option<Vec2> {
        self.layer.map.get_map_asset(map_asset).map(|tiled_map| {
            let grid_size = get_grid_size(&tiled_map.map);
            let tile_size = tile_size_from_grid(&grid_size);
            self.position.center_in_world(
                &tiled_map.tilemap_size,
                &grid_size,
                &tile_size,
                &get_map_type(&tiled_map.map),
                anchor,
            )
        })
    }
}
