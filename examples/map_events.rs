//! This example shows how to use map loading events.

use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;
use bevy_ecs_tilemap::prelude::*;

mod helper;

fn main() {
    App::new()
        // Bevy default plugins
        .add_plugins(DefaultPlugins)
        // Examples helper plugin (does not matter for this example)
        .add_plugins(helper::HelperPlugin)
        // bevy_ecs_tilemap and bevy_ecs_tiled main plugins
        .add_plugins(TilemapPlugin)
        .add_plugins(TiledMapPlugin::default())
        // Add observers for map loading events
        .observe(map_created)
        .observe(layer_created)
        .observe(object_created)
        .observe(special_tile_created)
        // Add our systems and run the app!
        .add_systems(Startup, startup)
        .run();
}

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());
    commands.spawn(TiledMapHandle(asset_server.load("finite.tmx")));
}

fn map_created(
    trigger: Trigger<TiledMapCreated>,
    q_map: Query<&Name, With<TiledMapMarker>>,
    map_asset: Res<Assets<TiledMap>>,
) {
    // We can either access the map components
    if let Ok(name) = q_map.get(trigger.event().map) {
        info!("Received TiledMapCreated event for map '{}'", name);
    }

    // Or directly the underneath Tiled Map structure
    let map = trigger.event().map(&map_asset);
    info!("Loaded map: {:?}", map);
}

fn layer_created(
    trigger: Trigger<TiledLayerCreated>,
    q_layer: Query<&Name, With<TiledMapLayer>>,
    map_asset: Res<Assets<TiledMap>>,
) {
    // We can either access the layer components
    if let Ok(name) = q_layer.get(trigger.event().layer) {
        info!("Received TiledLayerCreated event for layer '{}'", name);
    }

    // Or directly the underneath Map or Layer structures
    let _map = trigger.event().map(&map_asset);
    let layer = trigger.event().layer(&map_asset);
    info!("Loaded layer: {:?}", layer);
}

fn object_created(
    trigger: Trigger<TiledObjectCreated>,
    q_object: Query<&Name, With<TiledMapObject>>,
    map_asset: Res<Assets<TiledMap>>,
) {
    // We can either access the object components
    if let Ok(name) = q_object.get(trigger.event().object) {
        info!("Received TiledObjectCreated event for object '{}'", name);
    }

    // Or directly the underneath Map, Layer or Object structures
    let _map = trigger.event().map(&map_asset);
    let _layer = trigger.event().layer(&map_asset);
    let object = trigger.event().object(&map_asset);
    info!("Loaded object: {:?}", object);
}

fn special_tile_created(
    trigger: Trigger<TiledSpecialTileCreated>,
    q_tile: Query<&Name, With<TiledMapTile>>,
    map_asset: Res<Assets<TiledMap>>,
) {
    // We can either access the tile components
    if let Ok(name) = q_tile.get(trigger.event().tile) {
        info!("Received TiledSpecialTileCreated event for tile '{}'", name);
    }

    // Or directly the underneath Map and Layer structures
    let _map = trigger.event().map(&map_asset);
    let _layer = trigger.event().layer(&map_asset);
    let tile = trigger.event().tile(&map_asset);
    info!("Loaded tile: {:?}", tile);
}
