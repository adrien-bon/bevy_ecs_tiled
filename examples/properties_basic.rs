//! This example shows how to map custom tiles and objects properties from Tiled to Bevy Components.

use std::env;

use bevy::prelude::*;
use bevy_ecs_tiled::{prelude::*, TiledMapPluginConfig};
use bevy_ecs_tilemap::prelude::*;

mod helper;

fn main() {
    // Use a custom file path to export registered types in Tiled format
    let mut path = env::current_dir().unwrap();
    path.push("examples");
    path.push("my_tiled_export_file.json");

    App::new()
        // Bevy default plugins
        .add_plugins(DefaultPlugins)
        // Examples helper plugin (does not matter for this example)
        .add_plugins(helper::HelperPlugin)
        // bevy_ecs_tilemap and bevy_ecs_tiled main plugins
        .add_plugins(TilemapPlugin)
        .add_plugins(TiledMapPlugin(TiledMapPluginConfig {
            // Note: if you set this setting to `None`
            // properties will still be translated
            tiled_types_export_file: Some(path),
        }))
        // We need to register all the types we want to use
        .register_type::<BiomeInfos>()
        .register_type::<SpawnInfos>()
        // Add our systems and run the app!
        .add_systems(Startup, startup)
        .add_systems(Update, display_custom_tiles)
        .add_systems(Update, display_objects)
        .run();
}

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);
    commands.spawn(TiledMapHandle(
        asset_server.load("hex_map_pointy_top_odd.tmx"),
    ));
}

// You just have to define your Components and make sure they are properly registered and reflected.
// They will be exported in the Tiled .json file so they can be loaded then used inside Tiled.

#[derive(Component, Default, Debug, Reflect)]
#[reflect(Component, Default)]
struct BiomeInfos {
    ty: BiomeType,
    block_line_of_sight: bool,
}

#[derive(Component, Default, Debug, Reflect)]
#[reflect(Component, Default)]
struct SpawnInfos {
    ty: SpawnType,
}

// We can also define custom enums
#[derive(Default, Reflect, Debug)]
#[reflect(Default)]
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
#[reflect(Default)]
enum SpawnType {
    #[default]
    Unknown,
    Player,
    Enemy,
}

// Only print the first tile to avoid flooding the console
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
