//! Events related to Tiled maps and worlds.
//!
//! This module defines a generic [`TiledEvent`] type that can be used to represent various
//! events related to Tiled maps and worlds.
//!
//! It also defines specific events such as [`WorldCreated`] or [`MapCreated`] that are used
//! to signal the creation of a Tiled world or map.
//!
//! The events in this module can be received using either Bevy's buffered events or entity observers.

use std::{
    fmt::{self, Debug},
    sync::Arc,
};

use crate::prelude::*;
use bevy::{ecs::system::SystemParam, prelude::*};

#[allow(unused_imports)]
use crate::tiled::{
    helpers::{get_layer_from_map, get_object_from_map, get_tile_from_map, get_tileset_from_map},
    layer::TiledLayer,
    map::{asset::TiledMapAsset, TiledMap},
    object::TiledObject,
    tile::{TiledTile, TiledTilemap},
    world::{asset::TiledWorldAsset, TiledWorld},
};

/// Wrapper around Tiled events
///
/// Contains generic informations about origin of a particular Tiled event
#[derive(Event, Clone, Copy, PartialEq, Debug, Reflect, Component)]
#[event(auto_propagate, traversal = &'static ChildOf)]
#[reflect(Component, Debug, Clone)]
pub struct TiledEvent<E: Debug + Clone + Copy + Reflect> {
    /// The original target of this event, before bubbling
    pub origin: Entity,
    /// The specific event that was triggered
    pub event: E,

    /// [`AssetId`] of the [`TiledWorldAsset`]
    world: Option<(Entity, AssetId<TiledWorldAsset>)>,
    /// [`AssetId`] of the [`TiledMapAsset`]
    map: Option<(Entity, AssetId<TiledMapAsset>)>,
    layer: Option<(Entity, u32)>,
    tilemap: Option<(Entity, u32)>,
    tile: Option<(Entity, TilePos, TileId)>,
    object: Option<(Entity, u32)>,
}

impl<E> TiledEvent<E>
where
    E: Debug + Clone + Copy + Reflect,
{
    /// Creates a new [`TiledEvent`]
    pub fn new(origin: Entity, event: E) -> Self {
        Self {
            origin,
            event,
            world: None,
            map: None,
            layer: None,
            tilemap: None,
            tile: None,
            object: None,
        }
    }

    /// Transmute a [`TiledEvent`] from one kind to another
    pub fn transmute<O>(&self, origin: Option<Entity>, event: O) -> TiledEvent<O>
    where
        O: Debug + Clone + Copy + Reflect,
    {
        TiledEvent::<O> {
            origin: origin.unwrap_or(self.origin),
            event,
            world: self.world,
            map: self.map,
            layer: self.layer,
            tilemap: self.tilemap,
            tile: self.tile,
            object: self.object,
        }
    }

    /// Update the world information for this [`TiledEvent`]
    pub fn with_world(&mut self, entity: Entity, asset_id: AssetId<TiledWorldAsset>) -> &mut Self {
        self.world = Some((entity, asset_id));
        self
    }

    /// Update the map information for this [`TiledEvent`]
    pub fn with_map(&mut self, entity: Entity, asset_id: AssetId<TiledMapAsset>) -> &mut Self {
        self.map = Some((entity, asset_id));
        self
    }

    /// Update the layer information for this [`TiledEvent`]
    pub fn with_layer(&mut self, entity: Entity, layer_id: u32) -> &mut Self {
        self.layer = Some((entity, layer_id));
        self
    }

    /// Update the object information for this [`TiledEvent`]
    pub fn with_object(&mut self, entity: Entity, object_id: u32) -> &mut Self {
        self.object = Some((entity, object_id));
        self
    }

    /// Update the tilemap information for this [`TiledEvent`]
    pub fn with_tilemap(&mut self, entity: Entity, tileset_id: u32) -> &mut Self {
        self.tilemap = Some((entity, tileset_id));
        self
    }

    /// Update the tile information for this [`TiledEvent`]
    pub fn with_tile(&mut self, entity: Entity, position: TilePos, tile_id: TileId) -> &mut Self {
        self.tile = Some((entity, position, tile_id));
        self
    }

    /// Trigger observer and write event for this [`TiledEvent`]
    pub fn send(&self, commands: &mut Commands, event_writer: &mut EventWriter<TiledEvent<E>>) {
        commands.trigger_targets(*self, self.origin);
        event_writer.write(*self);
    }
}

impl<'a, E> TiledEvent<E>
where
    E: Debug + Clone + Copy + Reflect,
{
    /// Retrieve the [`TiledMap`] [`Entity`] associated with this [`TiledEvent`]
    pub fn get_world_entity(&self) -> Option<Entity> {
        self.world.map(|(e, _)| e)
    }

    /// Retrieve the [`TiledWorldAsset`] associated with this [`TiledEvent`]
    pub fn get_world_asset(
        &self,
        assets: &'a Res<Assets<TiledWorldAsset>>,
    ) -> Option<&'a TiledWorldAsset> {
        self.world.and_then(|(_, id)| assets.get(id))
    }

    /// Retrieve the [`World`] associated with this [`TiledEvent`]
    pub fn get_world(&self, assets: &'a Res<Assets<TiledWorldAsset>>) -> Option<&'a TiledRawWorld> {
        self.get_world_asset(assets).map(|w| &w.world)
    }

    /// Retrive the [`TiledWorld`] [`Entity`] associated with this [`TiledEvent`]
    pub fn get_map_entity(&self) -> Option<Entity> {
        self.map.map(|(e, _)| e)
    }

    /// Retrieve the [`TiledMapAsset`] associated with this [`TiledEvent`]
    pub fn get_map_asset(
        &self,
        assets: &'a Res<Assets<TiledMapAsset>>,
    ) -> Option<&'a TiledMapAsset> {
        self.map.and_then(|(_, id)| assets.get(id))
    }

    /// Retrieve the [`Map`] associated with this [`TiledEvent`]
    pub fn get_map(&self, assets: &'a Res<Assets<TiledMapAsset>>) -> Option<&'a Map> {
        self.get_map_asset(assets).map(|m| &m.map)
    }

    /// Retrieve the [`TiledLayer`] [`Entity`] associated with this [`TiledEvent`]
    pub fn get_layer_entity(&self) -> Option<Entity> {
        self.layer.map(|(e, _)| e)
    }

    /// Retrieve the layer ID associated with this [`TiledEvent`]
    pub fn get_layer_id(&self) -> Option<u32> {
        self.layer.map(|(_, id)| id)
    }

    /// Retrieve the [`Layer`] associated with this [`TiledEvent`]
    pub fn get_layer(&self, assets: &'a Res<Assets<TiledMapAsset>>) -> Option<Layer<'a>> {
        self.get_map(assets).and_then(|map| {
            self.get_layer_id()
                .and_then(|id| get_layer_from_map(map, id))
        })
    }

    /// Retrieve the [`TiledTilemap`] [`Entity`] associated with this [`TiledEvent`]
    pub fn get_tilemap_entity(&self) -> Option<Entity> {
        self.tilemap.map(|(e, _)| e)
    }

    /// Retrieve the tilemap tileset ID associated with this [`TiledEvent`]
    pub fn get_tilemap_tileset_id(&self) -> Option<u32> {
        self.tilemap.map(|(_, id)| id)
    }

    /// Retrieve the tilemap [`Tileset`] associated with this [`TiledEvent`]
    pub fn get_tilemap_tileset(
        &self,
        assets: &'a Res<Assets<TiledMapAsset>>,
    ) -> Option<&'a Arc<Tileset>> {
        self.get_map(assets).and_then(|map| {
            self.get_tilemap_tileset_id()
                .and_then(|id| get_tileset_from_map(map, id))
        })
    }

    /// Retrieve the [`TiledTile`] [`Entity`] associated with this [`TiledEvent`]
    pub fn get_tile_entity(&self) -> Option<Entity> {
        self.tile.map(|(e, _, _)| e)
    }

    /// Retrieve the [`TilePos`] associated with this [`TiledEvent`]
    pub fn get_tile_pos(&self) -> Option<TilePos> {
        self.tile.map(|(_, pos, _)| pos)
    }

    /// Retrieve the [`TileId`] associated with this [`TiledEvent`]
    pub fn get_tile_id(&self) -> Option<TileId> {
        self.tile.map(|(_, _, id)| id)
    }

    /// Retrieve the [`Tile`] associated with this [`TiledEvent`]
    pub fn get_tile(&self, assets: &'a Res<Assets<TiledMapAsset>>) -> Option<Tile<'a>> {
        self.get_map(assets).and_then(|map| {
            self.get_tilemap_tileset_id().and_then(|tileset_id| {
                self.get_tile_id()
                    .and_then(|id| get_tile_from_map(map, tileset_id, id))
            })
        })
    }

    /// Retrieve the [`TiledObject`] [`Entity`] associated with this [`TiledEvent`]
    pub fn get_object_entity(&self) -> Option<Entity> {
        self.object.map(|(e, _)| e)
    }

    /// Retrieve the object ID associated with this [`TiledEvent`]
    pub fn get_object_id(&self) -> Option<u32> {
        self.object.map(|(_, id)| id)
    }

    /// Retrieve the tilemap [`Tileset`] associated with this [`TiledEvent`]
    pub fn get_object(&self, assets: &'a Res<Assets<TiledMapAsset>>) -> Option<Object<'a>> {
        self.get_map(assets).and_then(|map| {
            self.get_object_id()
                .and_then(|id| get_object_from_map(map, id))
        })
    }
}

/// A [`TiledWorld`] was created
///
/// See also [`TiledEvent`]
#[derive(Clone, Copy, PartialEq, Debug, Reflect)]
#[reflect(Clone, PartialEq)]
pub struct WorldCreated;

/// A [`TiledMap`] was created
///
/// See also [`TiledEvent`]
#[derive(Clone, Copy, PartialEq, Debug, Reflect)]
#[reflect(Clone, PartialEq)]
pub struct MapCreated;

/// A [`TiledLayer`] was created
///
/// See also [`TiledEvent`]
#[derive(Clone, Copy, PartialEq, Debug, Reflect)]
#[reflect(Clone, PartialEq)]
pub struct LayerCreated;

/// A [`TiledTilemap`] was created
///
/// See also [`TiledEvent`]
#[derive(Clone, Copy, PartialEq, Debug, Reflect)]
#[reflect(Clone, PartialEq)]
pub struct TilemapCreated;

/// A [`TiledTile`] was created
///
/// See also [`TiledEvent`]
#[derive(Clone, Copy, PartialEq, Debug, Reflect)]
#[reflect(Clone, PartialEq)]
pub struct TileCreated;

/// A [`TiledObject`] was created
///
/// See also [`TiledEvent`]
#[derive(Clone, Copy, PartialEq, Debug, Reflect)]
#[reflect(Clone, PartialEq)]
pub struct ObjectCreated;

// /// All event writers used when loading a map
#[derive(SystemParam)]
pub(crate) struct TiledEventWriters<'w> {
    /// World events writer
    pub world_created: EventWriter<'w, TiledEvent<WorldCreated>>,
    /// Map events writer
    pub map_created: EventWriter<'w, TiledEvent<MapCreated>>,
    /// Layer events writer
    pub layer_created: EventWriter<'w, TiledEvent<LayerCreated>>,
    /// Tilemap events writer
    pub tilemap_created: EventWriter<'w, TiledEvent<TilemapCreated>>,
    /// Tile events writer
    pub tile_created: EventWriter<'w, TiledEvent<TileCreated>>,
    /// Object events writer
    pub object_created: EventWriter<'w, TiledEvent<ObjectCreated>>,
}

impl fmt::Debug for TiledEventWriters<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("TiledEventWriters").finish()
    }
}

pub(crate) fn plugin(app: &mut App) {
    app.add_event::<TiledEvent<WorldCreated>>()
        .register_type::<TiledEvent<WorldCreated>>();
    app.add_event::<TiledEvent<MapCreated>>()
        .register_type::<TiledEvent<MapCreated>>();
    app.add_event::<TiledEvent<LayerCreated>>()
        .register_type::<TiledEvent<LayerCreated>>();
    app.add_event::<TiledEvent<TilemapCreated>>()
        .register_type::<TiledEvent<TilemapCreated>>();
    app.add_event::<TiledEvent<TileCreated>>()
        .register_type::<TiledEvent<TileCreated>>();
    app.add_event::<TiledEvent<ObjectCreated>>()
        .register_type::<TiledEvent<ObjectCreated>>();
}
