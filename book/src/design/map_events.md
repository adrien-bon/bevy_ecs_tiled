# Map loading events

When loading a map or a world, you will receive events to both notifty you about the loading progress and allow you to customize how you map will be displayed.

There are five events :

- Four events are sent when loading a map or a world :
  - [`TiledMapCreated`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/map/events/struct.TiledMapCreated.html): called once the map has finished loading, contains information about the map. Called after the world it belongs to if you actually loaded a world.
  - [`TiledLayerCreated`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/map/events/struct.TiledLayerCreated.html): called once the map it belongs to has finished loading, contains informations about a specific layer.
  - [`TiledObjectCreated`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/map/events/struct.TiledObjectCreated.html): called once the map it belongs to has finished loading, contains informations about a specific object on a given layer.
  - [`TiledTileCreated`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/map/events/struct.TiledTileCreated.html): called once the map it belongs to has finished loading, contains informations about a specific tile on a given layer.
- A fifth one is sent only for worlds :
  - [`TiledWorldCreated`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/world/events/struct.TiledWorldCreated.html): called once the world has finished loading, contains informations about the world.

These events are both regular events and entity-scoped observers.

You can either use an `EventReader` to read them or a `Trigger` :

```rust,no_run
use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        // Spawn a map and attach an observer on it.
        .spawn(TiledMapHandle(
            asset_server.load("maps/orthogonal/finite.tmx"),
        ))
        // Add an "in-line" observer to detect when the map has finished loading
        .observe(|trigger: Trigger<TiledMapCreated>, map_query: Query<&Name, With<TiledMapMarker>>| {
            if let Ok(name) = map_query.get(trigger.event().entity) {
                info!("=> Observer TiledMapCreated was triggered for map '{}'", name);
            }
        });
}

// Just use a regular system which will receive TiledMapCreated events
fn handle_map_event(
    mut map_events: EventReader<TiledMapCreated>,
    map_query: Query<&Name, With<TiledMapMarker>>,
) {
    for e in map_events.read() {
        if let Ok(name) = map_query.get(e.entity) {
            info!("=> Received TiledMapCreated event for map '{}'", name);
        }
    }
}
```

All these events are sent **after** the map or world is actually loaded and their components have been inserted, including the ones coming from user properties.

A [dedicated example](https://github.com/adrien-bon/bevy_ecs_tiled/blob/main/examples/map_events.rs) is available to demonstrate how to use these.
