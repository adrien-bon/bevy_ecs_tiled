# Map Loading Events

When loading a map or a world with `bevy_ecs_tiled`, you receive events that both notify you about the loading progress and allow you to customize how your map or world is handled in your game.

These events are extremely useful for:

- Running custom logic when a map, layer, object, or tile is loaded
- Accessing or modifying entities as soon as they are available
- Integrating with other systems (e.g., spawning a player, setting up triggers, etc.)

---

## `TiledEvent<E>`

All `bevy_ecs_tiled` events are encapsulated in the [`TiledEvent<E>`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/tiled/event/struct.TiledEvent.html) structure.
This provides more context for each event, including the origin of each particular event `E` and helper methods to access Tiled data.

For example, an event related to an object will contain not only information to identify the object itself, but also the layer and the map containing it.

```rust,no_run
use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;

fn handle_event(
    mut object_events: MessageReader<TiledEvent<ObjectCreated>>,
) {
    // Even though we receive an event for a Tiled object,
    // we can retrieve information about the Tiled map
    for e in object_events.read() {
        let object_entity = e.origin;
        if let Some(map_asset) = e.get_map_asset() {
            info!("Received TiledEvent<ObjectCreated> for: '{:?}'", map_asset);
        }
    }
}
```

---

## List of Map and World Events

There are six main events:

- **For maps and worlds:**
  - [`TiledEvent<MapCreated>`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/tiled/event/struct.MapCreated.html):  
    Emitted once a map has finished loading. Contains information about the map entity.  
    (If loading a world, this is called after the world event.)
  - [`TiledEvent<LayerCreated>`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/tiled/event/struct.LayerCreated.html):  
    Emitted for each layer after the map is loaded. Contains information about the layer entity.
  - [`TiledEvent<TilemapCreated>`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/tiled/event/struct.TilemapCreated.html):  
    Emitted for each tilemap, for every tile layer, after the map is loaded. Contains information about the tilemap entity.
  - [`TiledEvent<TileCreated>`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/tiled/event/struct.TileCreated.html):  
    Emitted for each tile after the map is loaded. Contains information about the tile entity. Note this event is only fired for tiles using custom properties.
  - [`TiledEvent<ObjectCreated>`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/tiled/event/struct.ObjectCreated.html):  
    Emitted for each object after the map is loaded. Contains information about the object entity.
- **For worlds only:**
  - [`TiledEvent<WorldCreated>`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/tiled/event/struct.WorldCreated.html):  
    Emitted once the world has finished loading. Contains information about the world entity.

All these events are sent **after** the map or world is fully loaded and all components (including those from user properties) have been inserted.

---

## How to Use Map Events

You can listen to these events in two ways:

### 1. Using Buffered Events

```rust,no_run
use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;

fn handle_map_event(
    mut map_events: MessageReader<TiledEvent<MapCreated>>,
    map_query: Query<&Name, With<TiledMap>>,
) {
    for e in map_events.read() {
        if let Some(map_entity) = e.get_map_entity() {
            if let Ok(name) = map_query.get(map_entity) {
                info!("=> Received TiledMapCreated event for map '{}'", name);
            }
        }
    }
}
```

### 2. Using Observers

Observers can be attached directly to map or world entities.  
This is useful for "inline" logic, such as spawning a player when a map is ready:

```rust,no_run
use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        // Spawn a map and attach an observer on it.
        .spawn(TiledMap(
            asset_server.load("maps/orthogonal/finite.tmx"),
        ))
        // Add an "in-line" observer to detect when the map has finished loading
        .observe(|trigger: Trigger<TiledEvent<MapCreated>>, map_query: Query<&Name, With<TiledMap>>| {
            if let Some(map_entity) = e.get_map_entity() {
                if let Ok(name) = map_query.get(map_entity) {
                    info!("=> Observer TiledMapCreated was triggered for map '{}'", name);
                }
            }
        });
}
```

---

## When Are Events Sent?

All these events are sent **after** the map or world is actually loaded and their components have been inserted, including those coming from user properties.  
This means you can safely query for components and child entities in your event handlers.

---

## Dedicated Example

A [dedicated example](https://github.com/adrien-bon/bevy_ecs_tiled/blob/main/examples/map_events.rs) is available to demonstrate how to use these events in practice.
