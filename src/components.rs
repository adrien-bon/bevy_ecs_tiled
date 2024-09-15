//! This module contains all `Component`s definition.

use crate::prelude::*;
use bevy::{ecs::system::EntityCommands, prelude::*, utils::HashMap};
use bevy_ecs_tilemap::prelude::*;

/// Marker `Component` for re-spawning the whole map
///
/// Example:
/// ```rust,no_run
/// fn handle_respawn(
///     mut commands: Commands,
///     map_query: Query<(Entity, &Handle<TiledMap>)>,
/// ) {
///     let (entity, _) = map_query.single();
///     commands.entity(entity).insert(RespawnTiledMap);
/// }
/// ```
#[derive(Component)]
pub struct RespawnTiledMap;

/// `Component` storing the list of current layers.
#[derive(Component, Default)]
pub struct TiledLayersStorage {
    /// Hashmap containing the list of loaded layers.
    pub storage: HashMap<u32, Entity>,
}

/// Marker `Component` for a Tiled map.
#[derive(Component)]
pub struct TiledMapMarker;

/// Marker `Component` for a Tiled map layer.
#[derive(Component)]
pub struct TiledMapLayer {
    // Store the map id so that we can delete layers for this map later.
    // We don't want to store the handle as a `Component` because the parent
    // entity already has it and it complicates queries.
    pub map_handle_id: AssetId<TiledMap>,
}

/// Marker `Component` for a Tiled map tile layer.
#[derive(Component)]
pub struct TiledMapTileLayer;

/// Marker `Component` for a Tiled map tile layer for a given tileset.
#[derive(Component)]
pub struct TiledMapTileLayerForTileset;

/// Marker `Component` for a Tiled map object layer.
#[derive(Component)]
pub struct TiledMapObjectLayer;

/// Marker `Component` for a Tiled map group layer.
#[derive(Component)]
pub struct TiledMapGroupLayer;

/// Marker `Component` for a Tiled map image layer.
#[derive(Component)]
pub struct TiledMapImageLayer;

/// Marker `Component` for a Tiled map tile.
#[derive(Component)]
pub struct TiledMapTile;

/// Marker `Component` for a Tiled map object.
#[derive(Component)]
pub struct TiledMapObject;

/// Controls position of the map in the world.
#[derive(Default, Clone)]
pub enum MapPositioning {
    #[default]
    /// Do not tweak layers position, only use raw position from Tiled
    LayerOffset,
    /// Update layers position and mimics Bevy's coordinate system so that (0, 0) is at the center of the map.
    Centered,
}

/// `Bundle` holding all the configuration needed to load a map with `bevy_ecs_tiled` plugin.
///
/// Only thing to do is to initialize this `Bundle` with a valid `Handle<TiledMap>`then spawn it.
///
/// Example:
/// ```rust,no_run
/// fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
///    commands.spawn(TiledMapBundle {
///        tiled_map: asset_server.load("map.tmx"),
///        ..default()
///    });
/// }
/// ```
#[derive(Default, Bundle)]
pub struct TiledMapBundle {
    /// Handle to the .tmx file to load reprenseting the map.
    /// It is the only mandatory field to actually spawn the map.
    pub tiled_map: Handle<TiledMap>,
    /// Hashmap holding all the layers of the map.
    /// Must be left as default when spawning the bundle.
    pub storage: TiledLayersStorage,
    /// Render settings from `bevy_ecs_tilemap`.
    pub render_settings: TilemapRenderSettings,
    /// Settings from `bevy_ecs_tiled`.
    pub tiled_settings: TiledMapSettings,
}

/// Callback for extending physics colliders.
///
/// Provided `EntityCommands` can be used to add additionnal `Component`s to the collider.
///
/// Example:
/// ```rust,no_run
/// // Just add a marker `Component`
/// settings = TiledMapSettings {
///     collider_callback: |entity_commands| {
///         entity_commands.insert(MyColliderMarker);
///     },
///     ..default()
/// };
/// ```
pub type ColliderCallback = fn(&mut EntityCommands);

/// `Component` holding Tiled related settings.
///
/// Controls various settings related to the way we handle the Tiled map.
#[derive(Clone, Component)]
pub struct TiledMapSettings {
    /// Specify which Tiled object layers to add colliders for using their name.
    ///
    /// Colliders will be automatically added for all objects whose containing layer name matches this filter.
    ///
    /// By default, we add colliders for all objects.
    pub collision_layer_names: ObjectNames,
    /// Specify which tiles collision object to add colliders for using their name.
    ///
    /// Colliders will be automatically added for all tiles collision objects whose name matches this filter.
    ///
    /// By default, we add colliders for all collision objects.
    pub collision_object_names: ObjectNames,
    /// Physics collider callback.
    ///
    /// Using this callback, we can add extra `Component`s to colliders which were automatically spawned.
    pub collider_callback: ColliderCallback,
    /// Physics backend to use.
    ///
    /// Specify which physics backend to use.
    pub physics_backend: PhysicsBackend,
    /// Specify which position transformation offset should be applied to the map.
    pub map_positioning: MapPositioning,
}

impl Default for TiledMapSettings {
    fn default() -> Self {
        Self {
            collider_callback: |_| {},
            collision_layer_names: ObjectNames::default(),
            collision_object_names: ObjectNames::default(),
            map_positioning: MapPositioning::default(),
            physics_backend: PhysicsBackend::default(),
        }
    }
}
