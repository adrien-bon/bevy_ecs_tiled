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
    helpers::{tile_from_map, tileset_from_map},
    layer::TiledLayer,
    map::{TiledMap, asset::TiledMapAsset},
    object::TiledObject,
    tile::{TiledTile, TiledTilemap},
    world::{TiledWorld, asset::TiledWorldAsset},
};

/// Wrapper around Tiled events
///
/// Contains generic informations about origin of a particular Tiled event
#[derive(Message, EntityEvent, Clone, Copy, PartialEq, Debug, Reflect, Component)]
#[entity_event(auto_propagate, propagate = &'static ChildOf)]
#[reflect(Component, Debug, Clone)]
pub struct TiledEvent<E: Debug + Clone + Copy + Reflect> {
    /// The entity this event happened for
    pub entity: Entity,
    /// The original target of this event, before bubbling
    pub origin: Entity,
    /// The specific event that was triggered
    pub event: E,

    /// [`AssetId`] of the [`TiledWorldAsset`]
    world: Option<(Entity, AssetId<TiledWorldAsset>)>,
    /// [`AssetId`] of the [`TiledMapAsset`]
    map: Option<(Entity, AssetId<TiledMapAsset>)>,
    layer: Option<Entity>,
    tilemap: Option<(Entity, TilesetId)>,
    tile: Option<(Entity, TilePos, TilesetId, tiled::TileId)>,
    object: Option<Entity>,
}

impl<E> TiledEvent<E>
where
    E: Debug + Clone + Copy + Reflect,
{
    /// Creates a new [`TiledEvent`]
    pub fn new(origin: Entity, event: E) -> Self {
        Self {
            entity: origin,
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
            entity: self.entity,
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
    pub fn with_layer(&mut self, entity: Entity) -> &mut Self {
        self.layer = Some(entity);
        self
    }

    /// Update the object information for this [`TiledEvent`]
    pub fn with_object(&mut self, entity: Entity) -> &mut Self {
        self.object = Some(entity);
        self
    }

    /// Update the tilemap information for this [`TiledEvent`]
    pub fn with_tilemap(&mut self, entity: Entity, tileset_id: TilesetId) -> &mut Self {
        self.tilemap = Some((entity, tileset_id));
        self
    }

    /// Update the tile information for this [`TiledEvent`]
    pub fn with_tile(
        &mut self,
        entity: Entity,
        position: TilePos,
        tileset_id: TilesetId,
        tile_id: tiled::TileId,
    ) -> &mut Self {
        self.tile = Some((entity, position, tileset_id, tile_id));
        self
    }

    /// Trigger observer and write event for this [`TiledEvent`]
    pub fn send(&self, commands: &mut Commands, message_writer: &mut MessageWriter<TiledEvent<E>>) {
        commands.trigger(*self);
        message_writer.write(*self);
    }
}

impl<'a, E> TiledEvent<E>
where
    E: Debug + Clone + Copy + Reflect,
{
    /// Retrieve the [`TiledWorld`] [`Entity`] associated with this [`TiledEvent`]
    pub fn world_entity(&self) -> Option<Entity> {
        self.world.map(|(e, _)| e)
    }

    /// Retrieve the [`TiledWorldAsset`] associated with this [`TiledEvent`]
    pub fn world_asset(&self, assets: &'a Assets<TiledWorldAsset>) -> Option<&'a TiledWorldAsset> {
        self.world.and_then(|(_, id)| assets.get(id))
    }

    /// Retrieve the [`tiled::World`] associated with this [`TiledEvent`]
    pub fn world(&self, assets: &'a Assets<TiledWorldAsset>) -> Option<&'a tiled::World> {
        self.world_asset(assets).map(|w| &w.world)
    }

    /// Retrieve the [`TiledMap`] [`Entity`] associated with this [`TiledEvent`]
    pub fn map_entity(&self) -> Option<Entity> {
        self.map.map(|(e, _)| e)
    }

    /// Retrieve the [`TiledMapAsset`] associated with this [`TiledEvent`]
    pub fn map_asset(&self, assets: &'a Assets<TiledMapAsset>) -> Option<&'a TiledMapAsset> {
        self.map.and_then(|(_, id)| assets.get(id))
    }

    /// Retrieve the [`tiled::Map`] associated with this [`TiledEvent`]
    pub fn map(&self, assets: &'a Assets<TiledMapAsset>) -> Option<&'a tiled::Map> {
        self.map_asset(assets).map(|m| &m.map)
    }

    /// Retrieve the [`TiledLayer`] [`Entity`] associated with this [`TiledEvent`]
    pub fn layer_entity(&self) -> Option<Entity> {
        self.layer
    }

    /// Retrieve the [`TiledTilemap`] [`Entity`] associated with this [`TiledEvent`]
    pub fn tilemap_entity(&self) -> Option<Entity> {
        self.tilemap.map(|(e, _)| e)
    }

    /// Retrieve the tilemap [`TilesetId`] associated with this [`TiledEvent`]
    pub fn tilemap_tileset_id(&self) -> Option<TilesetId> {
        self.tilemap.map(|(_, id)| id)
    }

    /// Retrieve the tilemap [`tiled::Tileset`] associated with this [`TiledEvent`]
    pub fn tilemap_tileset(
        &self,
        assets: &'a Assets<TiledMapAsset>,
    ) -> Option<&'a Arc<tiled::Tileset>> {
        self.map(assets).and_then(|map| {
            self.tilemap_tileset_id()
                .and_then(|id| tileset_from_map(map, id))
        })
    }

    /// Retrieve the [`TiledTile`] [`Entity`] associated with this [`TiledEvent`]
    pub fn tile_entity(&self) -> Option<Entity> {
        self.tile.map(|(e, _, _, _)| e)
    }

    /// Retrieve the [`TilePos`] associated with this [`TiledEvent`]
    pub fn tile_pos(&self) -> Option<TilePos> {
        self.tile.map(|(_, pos, _, _)| pos)
    }

    /// Retrieve the [`TilesetId`] of the tile associated with this [`TiledEvent`]
    pub fn tile_tileset_id(&self) -> Option<TilesetId> {
        self.tile.map(|(_, _, tileset_id, _)| tileset_id)
    }

    /// Retrieve the [`tiled::TileId`] associated with this [`TiledEvent`]
    pub fn tile_id(&self) -> Option<tiled::TileId> {
        self.tile.map(|(_, _, _, id)| id)
    }

    /// Retrieve the [`tiled::Tile`] associated with this [`TiledEvent`]
    pub fn tile(&self, assets: &'a Assets<TiledMapAsset>) -> Option<tiled::Tile<'a>> {
        self.map(assets).and_then(|map| {
            self.tile_tileset_id().and_then(|tileset_id| {
                self.tile_id()
                    .and_then(|id| tile_from_map(map, tileset_id, id))
            })
        })
    }

    /// Retrieve the [`TiledObject`] [`Entity`] associated with this [`TiledEvent`]
    pub fn object_entity(&self) -> Option<Entity> {
        self.object
    }
}

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
/// Tilemaps are rendering entities: this event is sent when a tilemap is spawned during
/// [`TiledPostUpdateSystems::SyncRender`], either when the map is loaded or later on, when a tile
/// is added to a layer which does not have a tilemap for this tile tileset yet.
/// It is only sent when the `render` feature is enabled.
///
/// See also [`TiledEvent`]
#[derive(Clone, Copy, PartialEq, Debug, Reflect)]
#[reflect(Clone, PartialEq)]
pub struct TilemapCreated;

/// A [`TiledTile`] with custom properties was created
///
/// Only sent when a map is loaded, not for tiles spawned afterwards.
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
pub(crate) struct TiledMessageWriters<'w> {
    /// World events writer
    pub world_created: MessageWriter<'w, TiledEvent<WorldCreated>>,
    /// Map events writer
    pub map_created: MessageWriter<'w, TiledEvent<MapCreated>>,
    /// Layer events writer
    pub layer_created: MessageWriter<'w, TiledEvent<LayerCreated>>,
    /// Tile events writer
    pub tile_created: MessageWriter<'w, TiledEvent<TileCreated>>,
    /// Object events writer
    pub object_created: MessageWriter<'w, TiledEvent<ObjectCreated>>,
}

impl fmt::Debug for TiledMessageWriters<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("TiledMessageWriters").finish()
    }
}

pub(crate) fn plugin(app: &mut App) {
    app.add_message::<TiledEvent<WorldCreated>>()
        .register_type::<TiledEvent<WorldCreated>>();
    app.add_message::<TiledEvent<MapCreated>>()
        .register_type::<TiledEvent<MapCreated>>();
    app.add_message::<TiledEvent<LayerCreated>>()
        .register_type::<TiledEvent<LayerCreated>>();
    app.add_message::<TiledEvent<TilemapCreated>>()
        .register_type::<TiledEvent<TilemapCreated>>();
    app.add_message::<TiledEvent<TileCreated>>()
        .register_type::<TiledEvent<TileCreated>>();
    app.add_message::<TiledEvent<ObjectCreated>>()
        .register_type::<TiledEvent<ObjectCreated>>();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    #[test]
    fn accessors_can_be_used_with_assets_from_world() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()));
        app.add_plugins((
            crate::tiled::cache::plugin,
            crate::tiled::map::asset::plugin,
            crate::tiled::map::loader::plugin,
        ));

        let handle: Handle<TiledMapAsset> = app
            .world()
            .resource::<AssetServer>()
            .load("tests/chunking/empty_map.tmx");
        let deadline = Instant::now() + Duration::from_secs(10);
        while !app
            .world()
            .resource::<AssetServer>()
            .is_loaded_with_dependencies(&handle)
        {
            assert!(Instant::now() < deadline, "test map did not load in time");
            app.update();
            std::thread::sleep(Duration::from_millis(1));
        }

        let entity = app.world_mut().spawn_empty().id();
        let event = TiledEvent::new(entity, LayerCreated)
            .with_map(entity, handle.id())
            .with_layer(entity)
            .to_owned();

        // No system involved: assets are borrowed directly from the World
        let assets = app.world().resource::<Assets<TiledMapAsset>>();
        assert_eq!(
            event
                .map(assets)
                .and_then(|map| map.layers().next())
                .map(|layer| layer.name.clone()),
            Some("ground".to_string())
        );
        assert_eq!(event.layer_entity(), Some(entity));
        assert!(event.object_entity().is_none());
    }
}
