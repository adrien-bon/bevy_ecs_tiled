use crate::prelude::*;
use bevy::{ecs::system::EntityCommands, prelude::*, utils::HashMap};
use bevy_ecs_tilemap::prelude::*;

// Stores a list of tiled layers.
#[derive(Component, Default)]
pub struct TiledLayersStorage {
    pub storage: HashMap<u32, Entity>,
}

/// Marker component for a Tiled map.
#[derive(Component)]
pub struct TiledMapMarker;

/// Marker component for a Tiled map layer.
#[derive(Component)]
pub struct TiledMapLayer {
    // Store the map id so that we can delete layers for this map later.
    // We don't want to store the handle as a component because the parent
    // entity already has it and it complicates queries.
    pub map_handle_id: AssetId<TiledMap>,
}

/// Marker component for a Tiled map tile layer.
#[derive(Component)]
pub struct TiledMapTileLayer;

/// Marker component for a Tiled map tile layer for a given tileset.
#[derive(Component)]
pub struct TiledMapTileLayerForTileset;

/// Marker component for a Tiled map object layer.
#[derive(Component)]
pub struct TiledMapObjectLayer;

/// Marker component for a Tiled map group layer.
#[derive(Component)]
pub struct TiledMapGroupLayer;

/// Marker component for a Tiled map image layer.
#[derive(Component)]
pub struct TiledMapImageLayer;

/// Marker component for a Tiled map tile.
#[derive(Component)]
pub struct TiledMapTile;

/// Marker component for a Tiled map object.
#[derive(Component)]
pub struct TiledMapObject;

#[derive(Default, Clone)]
pub enum MapPositioning {
    #[default]
    /// Transforms TilemapBundle starting from the layer's offset.
    LayerOffset,
    /// Mimics Bevy's coordinate system so that (0, 0) is at the center of the map.
    Centered,
}

#[derive(Default, Bundle)]
pub struct TiledMapBundle {
    pub tiled_map: Handle<TiledMap>,
    pub storage: TiledLayersStorage,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub render_settings: TilemapRenderSettings,
    pub tiled_settings: TiledMapSettings,
}

pub type ColliderCallback = fn(&mut EntityCommands);

#[derive(Clone, Component)]
pub struct TiledMapSettings {
    /// Specify which object layers to add collision shapes for.
    ///
    /// All shapes in these object layers will be added as collision shapes.
    pub collision_layer_names: ObjectNames,
    /// Specify which tileset object names to add collision shapes for.
    pub collision_object_names: ObjectNames,
    /// By default, we add a single collider to the shape: you can use
    /// this callback to add additional components to the collider
    pub collider_callback: ColliderCallback,
    /// Specify which position transformation offset should be applied.
    ///
    /// By default, the layer's offset will be used.
    /// For Bevy's coordinate system use MapPositioning::Centered
    pub map_positioning: MapPositioning,
}

impl Default for TiledMapSettings {
    fn default() -> Self {
        Self {
            collider_callback: |_| {},
            collision_layer_names: ObjectNames::default(),
            collision_object_names: ObjectNames::default(),
            map_positioning: MapPositioning::default(),
        }
    }
}
