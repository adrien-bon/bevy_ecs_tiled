
Add dependencies to your `Cargo.toml` file:

```toml
[dependencies]
bevy = "0.14"
bevy_ecs_tiled = "0.3"
bevy_ecs_tilemap = "0.14"
```

Then add the plugin to your app and spawn a map:

```rust,no_run
use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;
use bevy_ecs_tilemap::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(TilemapPlugin)
        .add_plugins(TiledMapPlugin)
        .add_systems(Startup, startup)
        .run();
}

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Spawn a 2D camera
    commands.spawn(Camera2dBundle::default());

    // Load the map: ensure any tile / tileset paths are relative to assets/ folder
    let map_handle: Handle<TiledMap> = asset_server.load("map.tmx");

    // Spawn the map with default options
    commands.spawn(TiledMapBundle {
        tiled_map: map_handle,
        ..Default::default()
    });
}
```

Please note that you should have the `map.tmx` file in your local `assets/` folder (as well as required dependencies, for instance associated tilesets).

See the [examples](https://github.com/adrien-bon/bevy_ecs_tiled/tree/main/examples/README.md) for more advanced use cases.
