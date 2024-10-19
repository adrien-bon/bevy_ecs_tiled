# Map loading events

After loading a map, the plugin will send several events:

- [`TiledMapCreated`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/events/struct.TiledMapCreated.html): called once the map has finished loading.
- [`TiledLayerCreated`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/events/struct.TiledLayerCreated.html): called for all layers.
- [`TiledObjectCreated`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/events/struct.TiledObjectCreated.html): called for all objects.
- [`TiledSpecialTileCreated`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/events/struct.TiledSpecialTileCreated.html): only called for "special tiles" ie. tiles with either custom properties or colliders.

These events are a way to access directly raw `Tiled` data and easily extend the plugin capabilities.

For instance, you can access a `tiled::Object` from the corresponding event:

```rust,no_run
use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;

fn object_created(
    trigger: Trigger<TiledObjectCreated>,
    map_asset: Res<Assets<TiledMap>>,
) {
    // Access raw Tiled data
    let _map = trigger.event().map(&map_asset);
    let _layer = trigger.event().layer(&map_asset);
    let object = trigger.event().object(&map_asset);
    info!("Loaded object: {:?}", object);
}
```

A [dedicated example](https://github.com/adrien-bon/bevy_ecs_tiled/blob/main/examples/map_events.rs) is available to demonstrate how to use these.
