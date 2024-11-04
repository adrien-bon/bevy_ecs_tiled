
Add dependencies to your `Cargo.toml` file:

```toml
[dependencies]
bevy = "0.15"
bevy_ecs_tiled = "0.4"
bevy_ecs_tilemap = "0.15"
```

Then add the plugin to your app and spawn a map:

```rust,no_run
use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;
use bevy_ecs_tilemap::prelude::*;

fn main() {
    App::new()
        // Add Bevy default plugins
        .add_plugins(DefaultPlugins)
        // Add bevy_ecs_tilemap plugin
        .add_plugins(TilemapPlugin)
        // Add bevy_ecs_tiled plugin
        .add_plugins(TiledMapPlugin::default())
        // Add our startup function to the schedule and run the app
        .add_systems(Startup, startup)
        .run();
}

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Spawn a 2D camera
    commands.spawn(Camera2d);

    // Load the map: ensure any tile / tileset paths are relative to assets/ folder
    let map_handle: Handle<TiledMap> = asset_server.load("map.tmx");

    // Spawn the map with default options
    commands.spawn(TiledMapHandle(map_handle));
}
```

Basically, all you have to do is to spawn a [`TiledMapHandle`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/struct.TiledMapHandle.html) with the map asset you want to load (the `map.tmx` file).
Note that this map asset should be in your local `assets/` folder, as well as required dependencies (for instance, associated tilesets).

You can tweak how to load the map by adding various components on the map entity, notably:

- [`TiledMapSettings`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/components/struct.TiledMapSettings.html)
- [`TilemapRenderSettings`](https://docs.rs/bevy_ecs_tilemap/latest/bevy_ecs_tilemap/map/struct.TilemapRenderSettings.html)
- [`Transform`](https://docs.rs/bevy/latest/bevy/transform/components/struct.Transform.html)
- [`Visibility`](https://docs.rs/bevy/latest/bevy/render/view/visibility/enum.Visibility.html)

You can browse the [examples](https://github.com/adrien-bon/bevy_ecs_tiled/tree/main/examples/README.md) for more advanced use cases.
