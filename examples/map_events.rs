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
        .add_plugins(TiledPlugin::default())
        // Examples helper plugins, such as the logic to pan and zoom the camera
        // This should not be used directly in your game (but you can always have a look)
        .add_plugins(helper::HelperPlugin)
        // Add our systems and run the app!
        .add_systems(Startup, startup)
        .add_systems(Update, (evt_map_created, evt_object_created))
        .run();
}

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);
    commands
        // Spawn a map and attach some observers on it.
        // All events and observers will be fired _after_ the map has finished loading
        .spawn(TiledMap(asset_server.load("maps/orthogonal/finite.tmx")))
        // Add an "in-line" observer to detect when the map has finished loading
        .observe(
            |trigger: On<TiledEvent<MapCreated>>, map_query: Query<&Name, With<TiledMap>>| {
                let Ok(name) = map_query.get(trigger.event().origin) else {
                    return;
                };
                info!(
                    "=> Observer TiledMapCreated was triggered for map '{}'",
                    name
                );
            },
        )
        // And another one, with a dedicated function, to detect layer loading
        .observe(obs_layer_created);
}

// We fire both an observer and a regular event, so you can also use an [`MessageReader`]
fn evt_map_created(
    mut map_events: MessageReader<TiledEvent<MapCreated>>,
    map_query: Query<(&Name, &TiledMapStorage), With<TiledMap>>,
    assets: Res<Assets<TiledMapAsset>>,
) {
    for e in map_events.read() {
        // We can access the map components via a regular query
        let Ok((name, storage)) = map_query.get(e.origin) else {
            return;
        };

        // Or directly the underneath tiled Map data
        let Some(map) = e.get_map(&assets) else {
            return;
        };

        info!("=> Received TiledMapCreated event for map '{}'", name);
        info!("Loaded map: {:?}", map);

        // Additionally, we can access Tiled items using the TiledMapStorage
        // component from the map.
        // Using this component, we can retrieve Tiled items entity and access
        // their own components with another query (not shown here).
        // This can be useful if you want for instance to create a resource
        // based upon tiles or objects data but make it available only when
        // the map is actually spawned.
        for (id, entity) in storage.objects() {
            info!(
                "(map) Object ID {:?} was spawned as entity {:?}",
                id, entity
            );
        }
    }
}

// Callback for our observer, will be triggered for every layer of the map
fn obs_layer_created(
    trigger: On<TiledEvent<LayerCreated>>,
    layer_query: Query<&Name, With<TiledLayer>>,
    assets: Res<Assets<TiledMapAsset>>,
) {
    // We can either access the layer components via a regular query
    let Ok(name) = layer_query.get(trigger.event().origin) else {
        return;
    };

    // Or directly the underneath tiled Layer data
    let Some(layer) = trigger.event().get_layer(&assets) else {
        return;
    };

    info!(
        "=> Observer TiledLayerCreated was triggered for layer '{}'",
        name
    );
    info!("Loaded layer: {:?}", layer);

    // Moreover, we can retrieve the TiledMapCreated event data from here
    let _map = trigger.event().get_map(&assets);
}

// A typical usecase for regular events is to update components associated with tiles, objects or layers.
// Here, we will add a small offset on the Z axis to our objects to demonstrate how to use the
// `TiledObjectCreated` event.
fn evt_object_created(
    mut object_events: MessageReader<TiledEvent<ObjectCreated>>,
    mut object_query: Query<(&Name, &mut Transform), With<TiledObject>>,
    mut z_offset: Local<f32>,
) {
    for e in object_events.read() {
        let Ok((name, mut transform)) = object_query.get_mut(e.origin) else {
            return;
        };

        info!("=> Received TiledObjectCreated event for object '{}'", name);

        info!("Apply z-offset = {:?}", *z_offset);
        transform.translation.z += *z_offset;
        *z_offset += 0.01;
    }
}
