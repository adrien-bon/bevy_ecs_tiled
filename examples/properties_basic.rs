//! This example shows how to map custom tiles and objects properties from Tiled to Bevy Components.

use std::env;

use bevy::prelude::*;
use bevy_ecs_tiled::{prelude::*, TiledMapPluginConfig};

mod helper;

fn main() {
    // Use a custom file path to export registered types in Tiled format
    let mut path = env::current_dir().unwrap();
    path.push("examples");
    path.push("my_tiled_export_file.json");

    App::new()
        // Bevy default plugins: prevent blur effect by changing default sampling
        .add_plugins(DefaultPlugins.build().set(ImagePlugin::default_nearest()))
        // Add bevy_ecs_tiled plugin: bevy_ecs_tilemap::TilemapPlugin will
        // be automatically added as well if it's not already done
        // For demonstration purpose, provide a custom path where to export registered types
        .add_plugins(TiledMapPlugin(TiledMapPluginConfig {
            // Note: if you set this setting to `None`
            // properties won't be exported anymore but
            // you will still be able to load them from the map
            tiled_types_export_file: Some(path),
        }))
        // Examples helper plugins, such as the logic to pan and zoom the camera
        // This should not be used directly in your game (but you can always have a look)
        .add_plugins(helper::HelperPlugin)
        // We need to register all the custom types we want to use
        .register_type::<BiomeInfos>()
        .register_type::<SpawnType>()
        .register_type::<ResourceType>()
        // Add our systems and run the app!
        .add_systems(Startup, startup)
        .add_systems(Update, display_custom_tiles)
        .add_systems(Update, display_objects)
        .run();
}

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);
    commands.spawn(TiledMapHandle(
        asset_server.load("maps/hexagonal/finite_pointy_top_odd.tmx"),
    ));
}

// You just have to define your Components and make sure they are properly registered and reflected.
// They will be exported in the Tiled .json file so they can be imported then used from Tiled.
// Next time you load your map, they will be automatically added as components on tiles / objects / layers entities

#[derive(Component, Default, Debug, Reflect)]
#[reflect(Component, Default)]
struct BiomeInfos {
    ty: BiomeType,
    block_line_of_sight: bool,
}

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

#[derive(Component, Default, Debug, Reflect)]
#[reflect(Component, Default)]
enum SpawnType {
    #[default]
    Unknown,
    Player {
        color: Color,
        id: u32,
    },
    Enemy(Color),
    Friendly,
}

#[derive(Component, Default, Debug, Reflect)]
#[reflect(Component, Default)]
enum ResourceType {
    #[default]
    Unknown,
    Wheat,
    Strawberry,
    Wood,
    Copper,
    Gold,
}

// Marker component so we only print each entity once
#[derive(Component)]
struct DoNotPrint;

#[allow(clippy::type_complexity)]
fn display_custom_tiles(
    mut commands: Commands,
    q_tile: Query<
        (Entity, &TilePos, Option<&BiomeInfos>, Option<&ResourceType>),
        Without<DoNotPrint>,
    >,
) {
    for (entity, position, biome_infos, resource_type) in q_tile.iter() {
        if let Some(i) = biome_infos {
            // Only print the first tile to avoid flooding the console
            info_once!("Found BiomeInfos [{:?} @ {:?}]", i, position);
        }
        if let Some(i) = resource_type {
            // Only print the first tile to avoid flooding the console
            info_once!("Found ResourceType [{:?} @ {:?}]", i, position);
        }
        commands.entity(entity).insert(DoNotPrint);
    }
}

fn display_objects(
    mut commands: Commands,
    q_object: Query<(Entity, &Transform, &SpawnType), Without<DoNotPrint>>,
) {
    for (entity, transform, spawn_type) in q_object.iter() {
        info!(
            "Found SpawnType [{:?} @ {:?}]",
            spawn_type, transform.translation
        );
        commands.entity(entity).insert(DoNotPrint);
    }
}
