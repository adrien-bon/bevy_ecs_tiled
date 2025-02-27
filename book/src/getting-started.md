
Add required dependencies to your `Cargo.toml` file:

```toml
[dependencies]
bevy = "0.15"
bevy_ecs_tiled = "0.5"
bevy_ecs_tilemap = "0.15"
```

Then add the plugin to your app and spawn a map.
Basically, all you have to do is to spawn a [`TiledMapHandle`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/struct.TiledMapHandle.html) with the map asset you want to load (the `map.tmx` file).
Note that this map asset should be in your local assets folder, as well as required dependencies (such as images or tilesets).
By default, this is the `./assets/` folder.

```rust,no_run
use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;

fn main() {
    App::new()
        // Add Bevy default plugins
        .add_plugins(DefaultPlugins)
        // Add bevy_ecs_tiled plugin: note that bevy_ecs_tilemap::TilemapPlugin
        // will be automatically added as well if it's not already done
        .add_plugins(TiledMapPlugin::default())
        // Add our startup function to the schedule and run the app
        .add_systems(Startup, startup)
        .run();
}

fn startup(
  mut commands: Commands,
  asset_server: Res<AssetServer>
) {
    // Spawn a Bevy 2D camera
    commands.spawn(Camera2d);

    // Load a map asset and retrieve the corresponding handle
    let map_handle: Handle<TiledMap> = asset_server.load("map.tmx");

    // Spawn a new entity with this handle
    commands.spawn(TiledMapHandle(map_handle));
}
```

This simplistic example will load a map using default settings.
You can tweak how to load the map by adding various components on the map entity, notably:

- [`TiledMapAnchor`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/components/enum.TiledMapAnchor.html)
- [`TiledMapLayerZOffset`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/components/struct.TiledMapLayerZOffset.html)
- [`TilemapRenderSettings`](https://docs.rs/bevy_ecs_tilemap/latest/bevy_ecs_tilemap/map/struct.TilemapRenderSettings.html)
- [`Transform`](https://docs.rs/bevy/latest/bevy/transform/components/struct.Transform.html)
- [`Visibility`](https://docs.rs/bevy/latest/bevy/render/view/visibility/enum.Visibility.html)

For instance, here's how you load a map but change its anchor point to be at center instead of bottom-left :

```rust,no_run
use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;

fn spawn_map(
  mut commands: Commands,
  asset_server: Res<AssetServer>
) {
    // You can also spawn your map and associated settings as a single bundle
    commands.spawn((
      TiledMapHandle(asset_server.load("map.tmx")),
      TiledMapAnchor::Center,
    ));
}
```

You can browse the [examples](https://github.com/adrien-bon/bevy_ecs_tiled/tree/main/examples/README.md) for more advanced use cases.
