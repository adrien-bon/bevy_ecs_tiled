//! This example shows how to use map loading events.

use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;

mod helper;

fn main() {
    App::new()
        // Bevy default plugins: prevent blur effect by changing default sampling
        .add_plugins(DefaultPlugins.build().set(ImagePlugin::default_nearest()))
        // Add bevy_ecs_tiled plugin: bevy_ecs_tilemap::TilemapPlugin will
        // be automatically added as well if it's not already done
        .add_plugins(TiledMapPlugin::default())
        // Examples helper plugins, such as the logic to pan and zoom the camera
        // This should not be used directly in your game (but you can always have a look)
        .add_plugins(helper::HelperPlugin)
        // Add our systems and run the app!
        .add_systems(Startup, startup)
        .run();
}

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);
    commands
        .spawn(TiledMapHandle(
            asset_server.load("maps/orthogonal/finite.tmx"),
        ))
        // Add observers for map loading events
        .observe(map_created)
        .observe(layer_created)
        .observe(object_created)
        .observe(tile_created);
}

fn map_created(
    trigger: Trigger<TiledMapCreated>,
    q_map: Query<&Name, With<TiledMapMarker>>,
    map_asset: Res<Assets<TiledMap>>,
) {
    // We can either access the map components
    if let Ok(name) = q_map.get(trigger.event().entity) {
        info!("Received TiledMapCreated event for map '{}'", name);
    }

    // Or directly the underneath tiled Map data
    let map = trigger.event().get_map(&map_asset);
    info!("Loaded map: {:?}", map);
}

fn layer_created(
    trigger: Trigger<TiledLayerCreated>,
    q_layer: Query<&Name, With<TiledMapLayer>>,
    map_asset: Res<Assets<TiledMap>>,
) {
    // We can either access the layer components
    if let Ok(name) = q_layer.get(trigger.event().entity) {
        info!("Received TiledLayerCreated event for layer '{}'", name);
    }

    // Or directly the underneath Map or Layer structures
    let _map = trigger.event().map.get_map(&map_asset);
    let layer = trigger.event().get_layer(&map_asset);
    info!("Loaded layer: {:?}", layer);
}

fn object_created(
    trigger: Trigger<TiledObjectCreated>,
    q_object: Query<&Name, With<TiledMapObject>>,
    map_asset: Res<Assets<TiledMap>>,
) {
    // We can either access the object components
    if let Ok(name) = q_object.get(trigger.event().entity) {
        info!("Received TiledObjectCreated event for object '{}'", name);
    }

    // Or directly the underneath Map, Layer or Object structures
    let _map = trigger.event().layer.map.get_map(&map_asset);
    let _layer = trigger.event().layer.get_layer(&map_asset);
    let object = trigger.event().get_object(&map_asset);
    info!("Loaded object: {:?}", object);
}

// Note that only tiles which have custom properties will trigger this event,
// even without the crate `user_properties` feature.
// The Debug implementation for tiled::LayerTile does not display the actual
// content of the properties field but it's there, go check the tileset if you
// don't believe me :)
fn tile_created(
    trigger: Trigger<TiledTileCreated>,
    q_tile: Query<&Name, With<TiledMapTile>>,
    map_asset: Res<Assets<TiledMap>>,
) {
    // We can either access the tile components
    if let Ok(name) = q_tile.get(trigger.event().entity) {
        info!("Received TiledTileCreated event for tile '{}'", name);
    }

    // Or directly the underneath Map and Layer structures
    let _map = trigger.event().layer.map.get_map(&map_asset);
    let _layer = trigger.event().layer.get_layer(&map_asset);
    let tile = trigger.event().get_tile(&map_asset);
    info!("Loaded tile: {:?}", tile);
}
