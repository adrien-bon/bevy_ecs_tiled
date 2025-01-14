//! Events related to Tiled map loading
//!
//! These events will be fired after the whole map has loaded.
//! More informations in the [dedicated example](https://github.com/adrien-bon/bevy_ecs_tiled/blob/main/examples/map_events.rs)

use crate::prelude::*;
use bevy::{ecs::system::SystemParam, prelude::*};
use bevy_ecs_tilemap::prelude::*;
use tiled::{Layer, LayerTile, Map, Object};

#[derive(SystemParam)]
pub struct TiledMapEventWriters<'w> {
    pub map_event: EventWriter<'w, TiledMapCreated>,
    pub layer_event: EventWriter<'w, TiledLayerCreated>,
    pub object_event: EventWriter<'w, TiledObjectCreated>,
    pub tile_event: EventWriter<'w, TiledTileCreated>,
}

/// Event sent when a map is spawned
#[derive(Component, Clone, Debug, Copy)]
pub struct TiledMapCreated {
    /// Spawned map [Entity]
    pub entity: Entity,
    /// [AssetId] of the [TiledMap]
    pub asset_id: AssetId<TiledMap>,
}

impl Event for TiledMapCreated {
    type Traversal = &'static Parent;
    const AUTO_PROPAGATE: bool = true;
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
#[derive(Component, Clone, Debug, Copy)]
pub struct TiledLayerCreated {
    /// Creation event of the map this layer belongs to
    pub map: TiledMapCreated,
    /// Spawned layer [Entity]
    pub entity: Entity,
    /// ID of this layer in the [Map]
    pub id: usize,
}

impl Event for TiledLayerCreated {
    type Traversal = &'static Parent;
    const AUTO_PROPAGATE: bool = true;
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
#[derive(Component, Clone, Debug, Copy)]
pub struct TiledObjectCreated {
    /// Creation event of the layer this object belongs to
    pub layer: TiledLayerCreated,
    /// Spawned object [Entity]
    pub entity: Entity,
    /// ID of this object in the [tiled::ObjectLayer]
    pub id: usize,
}

impl Event for TiledObjectCreated {
    type Traversal = &'static Parent;
    const AUTO_PROPAGATE: bool = true;
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
    pub fn world_position(&self, map_asset: &'a Res<Assets<TiledMap>>) -> Option<Vec2> {
        self.layer.map.get_map(map_asset).and_then(|map| {
            self.get_object(map_asset).map(|object| {
                from_tiled_coords_to_bevy(
                    Vec2::new(object.x, object.y),
                    &get_map_type(map),
                    &get_map_size(map),
                    &get_grid_size(map),
                )
            })
        })
    }
}

/// Event sent when a tile has finished loading
///
/// This event is only sent for tiles which contain custom properties.
#[derive(Component, Clone, Debug, Copy)]
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

impl Event for TiledTileCreated {
    type Traversal = &'static Parent;
    const AUTO_PROPAGATE: bool = true;
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
    pub fn world_position(&self, map_asset: &'a Res<Assets<TiledMap>>) -> Option<Vec2> {
        self.layer.map.get_map(map_asset).map(|map| {
            self.position
                .center_in_world(&get_grid_size(map), &get_map_type(map))
        })
    }
}
