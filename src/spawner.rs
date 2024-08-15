use bevy::{
    ecs::{system::RunSystemOnce, world::Command},
    prelude::*,
};
use bevy_ecs_tilemap::prelude::*;

use crate::{
    loader::{load_map_by_asset_id, TiledMap},
    prelude::*,
};

pub struct SpawnTiledMap {
    pub bundle: TiledMapBundle,
}

impl Command for SpawnTiledMap {
    fn apply(self, world: &mut World) {
        let map_handle = self.bundle.tiled_map.clone_weak();
        world.run_system_once_with(self, spawn_tiled_map);
        world.run_system_once_with(map_handle.id(), finalize_tiled_map)
    }
}

fn spawn_tiled_map(spawn_tiled_map: In<SpawnTiledMap>, mut commands: Commands) {
    commands.spawn(spawn_tiled_map.0.bundle);
}

fn finalize_tiled_map(
    id: In<AssetId<TiledMap>>,
    mut commands: Commands,
    mut maps: ResMut<Assets<TiledMap>>,
    tile_storage_query: Query<(Entity, &TileStorage)>,
    mut map_query: Query<(
        Entity,
        &Handle<TiledMap>,
        &mut TiledLayersStorage,
        &TilemapRenderSettings,
        &TiledMapSettings,
    )>,
    #[cfg(feature = "user_properties")] objects_registry: NonSend<TiledObjectRegistry>,
    #[cfg(feature = "user_properties")] custom_tiles_registry: NonSend<TiledCustomTileRegistry>,
) {
    load_map_by_asset_id(
        &mut commands,
        &mut maps,
        &tile_storage_query,
        &mut map_query,
        &id,
        #[cfg(feature = "user_properties")]
        &objects_registry,
        #[cfg(feature = "user_properties")]
        &custom_tiles_registry,
    )
}
