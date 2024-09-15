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
        // We need to register custom tiles / objects we created in Tiled.
        // Any class which is not registered will not be added to the tile / object entity
        .register_tiled_custom_tile::<TileBundle>("TileBundle")
        .register_tiled_custom_tile::<TileComponent>("TileComponent")
        .register_tiled_object::<SpawnBundle>("SpawnBundle")
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

// Only print the first tiles to avoid flooding the console
fn display_custom_tiles(q_tile: Query<(&TilePos, Option<&BiomeInfos>, Option<&TileComponent>)>) {
    for (position, biome_infos, tile_component) in q_tile.iter() {
        if let Some(i) = biome_infos {
            info_once!("Found TileBundle [{:?} @ {:?}]", i, position);
        }
        if let Some(c) = tile_component {
            info_once!("Found TileComponent [{:?} @ {:?}]", c, position);
        }
    }
}

// Only print the first object to avoid flooding the console
fn display_objects(q_object: Query<(&Transform, &SpawnInfos)>) {
    for (transform, spawn_infos) in q_object.iter() {
        info_once!("Found SpawnInfos [{:?} @ {:?}]", spawn_infos, transform);
    }
}

// Here, we define our custom tiles:
// - `TileBundle` reference `BiomeInfos`, which is another custom type, so it needs to derive `Bundle`.
//    When checking the corresponding entity, we will only see the `BiomeInfos` component since
//    `TileBundle` is a `Bundle`.
// - `TileComponent` only reference "standard" types, so it needs to derive `Component`.
//    When checking the corresponding entity, we will see the `TileComponent` component.
// Note that `Debug` and `Reflect` traits are only here for convenience and are not required.
#[derive(TiledCustomTile, Bundle, Default, Debug, Reflect)]
struct TileBundle {
    // We expect this field to be named 'BiomeInfos' in Tiled custom properties
    // If a field is not defined in Tiled, we will use the field Rust's default value
    #[tiled_rename = "BiomeInfos"]
    infos: BiomeInfos,
}

// Here, we also declare an observer: when a new `TileComponent` is spawned, the
// observer will automatically be triggered with a `TiledCustomTileCreated` event (see below).
#[derive(TiledCustomTile, Component, Default, Debug, Reflect)]
#[tiled_observer(test_tile_observer)]
struct TileComponent {
    prefered_color: bevy::color::Color,
}


// Same thing as above, but for an object.
// Note that the observer will be triggered with a `TiledObjectCreatedEvent` (see below).
#[derive(TiledObject, Bundle, Default, Reflect, Debug)]
#[tiled_observer(test_object_observer)]
struct SpawnBundle {
    infos: SpawnInfos,
}

// Here, we define our "custom types" which are Bevy `Component`s, since they
// are part of `TileBundle` and `SpawnBundle` (which are `Bundle`s).
#[derive(TiledClass, Component, Default, Debug, Reflect)]
struct BiomeInfos {
    #[tiled_rename = "Type"]
    ty: BiomeType,
    #[tiled_rename = "BlockLineOfSight"]
    block_line_of_sight: bool,
}

#[derive(TiledClass, Component, Default, Debug, Reflect)]
struct SpawnInfos {
    #[tiled_rename = "Type"]
    ty: SpawnType,
}

// We can also define custom enums
#[derive(TiledEnum, Default, Reflect, Debug)]
enum BiomeType {
    #[default]
    Unknown,
    Plain,
    Desert,
    Forest,
    Mountain,
}

#[derive(TiledEnum, Default, Reflect, Debug)]
enum SpawnType {
    #[default]
    Unknown,
    Player,
}

// Finally, declare our observers for tiles and objects.
// They can use all arguments available to observers: like `Commands`, `Query`, `Res`, etc...
// The only restriction is to have as first arguement:
// - `Trigger<TiledCustomTileCreated>` for tiles observers
// - `Trigger<TiledObjectCreated>` for objects observers
// Note that trigger.event().entity and trigger.entity() are equivalent.
// WARNING: since observers are fired as we load the map, some entity could have been not yet spawned.
fn test_tile_observer(
    trigger: Trigger<TiledCustomTileCreated>,
    q_tile: Query<&TileComponent>,
    // following arguments are not used but illustrate what can be used in an observer
    mut _commands: Commands,
    _q_tilemap: Query<&TilemapGridSize>,
    _asset_server: Res<AssetServer>,
) {
    info!("OBSERVER for tile [{:?}]", trigger.entity());
    if let Ok(comp) = q_tile.get(trigger.entity()) {
        info!("(OBSERVER) Found TileComponent for tile: {:?}", comp);
    }
}

fn test_object_observer(trigger: Trigger<TiledObjectCreated>, q_object: Query<&SpawnInfos>) {
    info!("OBSERVER for object [{:?}]", trigger.entity());
    if let Ok(spawn_infos) = q_object.get(trigger.entity()) {
        info!("(OBSERVER) Found SpawnInfos for object: {:?}", spawn_infos);
    }
}
