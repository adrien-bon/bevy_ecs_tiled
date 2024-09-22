//! This example shows how to map custom tiles and objects properties from Tiled to Bevy Components.

use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;
use bevy_ecs_tilemap::prelude::*;

mod helper;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(TilemapPlugin)
        .add_plugins(TiledMapPlugin)
        .add_plugins(helper::HelperPlugin)
        .add_systems(Startup, startup)
        .add_systems(Update, display_custom_tiles)
        .add_systems(Update, display_objects)
        .register_type::<BiomeInfos>()
        .register_type::<SpawnInfos>()
        .observe(map_created)
        // .observe(layer_created)
        // .observe(object_created)
        // .observe(tile_created)
        .run();
}

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());
    let map_handle: Handle<TiledMap> = asset_server.load("hex_map_pointy_top_odd.tmx");
    commands.spawn(TiledMapBundle {
        tiled_map: map_handle,
        ..Default::default()
    });
}

fn map_created(
    trigger: Trigger<TiledMapCreated>,
    q_map: Query<&Name, With<TiledMapMarker>>,
) {
    if let Ok(name) = q_map.get(trigger.event().map) {
        info!("Received TiledMapCreated event for map '{}'", name);
    }
}

// Only print the first tiles to avoid flooding the console
fn display_custom_tiles(q_tile: Query<(&TilePos, Option<&BiomeInfos>)>) {
    for (position, biome_infos) in q_tile.iter() {
        if let Some(i) = biome_infos {
            info_once!("Found TileBundle [{:?} @ {:?}]", i, position);
        }
    }
}

// Only print the first object to avoid flooding the console
fn display_objects(q_object: Query<(&Transform, &SpawnInfos)>) {
    for (transform, spawn_infos) in q_object.iter() {
        info_once!("Found SpawnInfos [{:?} @ {:?}]", spawn_infos, transform);
    }
}

#[derive(Component, Default, Debug, Reflect)]
#[reflect(Component)]
struct BiomeInfos {
    ty: BiomeType,
    block_line_of_sight: bool,
}

#[derive(Component, Default, Debug, Reflect)]
#[reflect(Component)]
struct SpawnInfos {
    ty: SpawnType,
}

// We can also define custom enums
#[derive(Default, Reflect, Debug)]
enum BiomeType {
    #[default]
    Unknown,
    Plain,
    Desert,
    Forest,
    Mountain,
    Water,
}

#[derive(Default, Reflect, Debug)]
enum SpawnType {
    #[default]
    Unknown,
    Player,
    Enemy,
}
