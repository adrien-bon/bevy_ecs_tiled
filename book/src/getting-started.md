
Add the required dependencies to your `Cargo.toml` file:

```toml
[dependencies]
bevy = "0.16"
bevy_ecs_tiled = "0.7"
```

### Basic Usage

To get started, add the plugin to your app and spawn a map entity.
All you need to do is spawn a [`TiledMap`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/struct.TiledMap.html) component with the map asset you want to load (e.g., your `map.tmx` file).
Make sure this map asset, along with any required dependencies (such as images or tilesets), is present in your local assets folder (by default, `./assets/`).

```rust,no_run
use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;

fn main() {
    App::new()
        // Add Bevy's default plugins
        .add_plugins(DefaultPlugins)
        // Add the bevy_ecs_tiled plugin (bevy_ecs_tilemap::TilemapPlugin will be added automatically if needed)
        .add_plugins(TiledPlugin::default())
        // Add your startup system and run the app
        .add_systems(Startup, startup)
        .run();
}

fn startup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    // Spawn a 2D camera
    commands.spawn(Camera2d);

    // Load a map asset and retrieve its handle
    let map_handle: Handle<TiledMapAsset> = asset_server.load("map.tmx");

    // Spawn a new entity with the TiledMap component
    commands.spawn(TiledMap(map_handle));
}
```

This simple example will load a map using the default settings.

### Customizing Map Loading

You can customize how the map is loaded by adding various components to the map entity, such as:

- [`TilemapAnchor`](https://docs.rs/bevy_ecs_tilemap/latest/bevy_ecs_tilemap/anchor/enum.TilemapAnchor.html) — Controls the anchor point of the tilemap.
- [`TiledMapLayerZOffset`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/tiled/map/struct.TiledMapLayerZOffset.html) — Adjusts the Z offset between map layers.
- [`TilemapRenderSettings`](https://docs.rs/bevy_ecs_tilemap/latest/bevy_ecs_tilemap/map/struct.TilemapRenderSettings.html) — Configures rendering options.
- [`Transform`](https://docs.rs/bevy/latest/bevy/transform/components/struct.Transform.html) — Sets the position, rotation, and scale of the map.
- [`Visibility`](https://docs.rs/bevy/latest/bevy/render/view/visibility/enum.Visibility.html) — Controls the visibility of the map entity.

For example, to load a map and set its anchor point to the center instead of the default bottom-left:

```rust,no_run
use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;

fn spawn_map(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    // Spawn the map with a custom anchor point
    commands.spawn((
        TiledMap(asset_server.load("map.tmx")),
        TilemapAnchor::Center,
    ));
}
```

### More Examples

For more advanced use cases, such as loading worlds, chunking, custom properties, or integrating with physics, see the [examples directory](https://github.com/adrien-bon/bevy_ecs_tiled/tree/main/examples/README.md) in the repository.

You can also refer to the [API documentation](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/) for details on all available components and configuration options.
