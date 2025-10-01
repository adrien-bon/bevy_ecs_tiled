
Add the required dependencies to your `Cargo.toml` file:

```toml
[dependencies]
bevy = "0.16"
bevy_ecs_tiled = "0.9"
```

### Basic Usage

To get started, add the plugin to your app and spawn a map entity.
All you need to do is spawn a [`TiledMap`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/tiled/map/struct.TiledMap.html) component with the map asset you want to load (e.g., your `map.tmx` file).
Make sure this map asset, along with any required dependencies (such as images or tilesets), is present in your local assets folder (by default, `./assets/`).

```rust,no_run
use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;

fn main() {
    App::new()
        // Add Bevy's default plugins
        .add_plugins(DefaultPlugins)
        // Add the bevy_ecs_tiled plugin
        // bevy_ecs_tilemap::TilemapPlugin will be added automatically if needed
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

You can customize how the map is loaded by listening to specific [`TiledEvent`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/tiled/event/struct.TiledEvent.html) or adding various components to the map entity, such as:

- [`TilemapAnchor`](https://docs.rs/bevy_ecs_tilemap/latest/bevy_ecs_tilemap/anchor/enum.TilemapAnchor.html) — Controls the anchor point of the tilemap.
- [`TiledMapLayerZOffset`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/tiled/map/struct.TiledMapLayerZOffset.html) — Adjusts the Z offset between map layers.
- [`TiledMapImageRepeatMargin`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/tiled/map/struct.TiledMapImageRepeatMargin.html) — Control the margin for repeated images.
- [`TilemapRenderSettings`](https://docs.rs/bevy_ecs_tilemap/latest/bevy_ecs_tilemap/map/struct.TilemapRenderSettings.html) — Configures rendering options.
- [`Transform`](https://docs.rs/bevy/latest/bevy/transform/components/struct.Transform.html) — Sets the position, rotation, and scale of the map.
- [`Visibility`](https://docs.rs/bevy/latest/bevy/render/view/visibility/enum.Visibility.html) — Controls the visibility of the map entity.

```rust,no_run
use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;

fn spawn_map(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands
        // Load a map and set its anchor point to the
        // center instead of the default bottom-left
        .spawn((
            TiledMap(asset_server.load("map.tmx")),
            TilemapAnchor::Center,
        ))
        // Add an "in-line" observer to detect when
        // the map has finished loading
        .observe(
            |map_created: On<TiledEvent<MapCreated>>,
             assets: Res<Assets<TiledMapAsset>>,
             query: Query<(&Name, &TiledMapStorage), With<TiledMap>>| {
                // We can access the map components via a regular query
                let Ok((name, storage)) = query.get(map_created.event().origin) else {
                    return;
                };
                info!("=> Observer TiledMapCreated was triggered for map '{name}'");

                // Or directly the underneath raw tiled::Map data
                let Some(map) = map_created.event().get_map(&assets) else {
                    return;
                };
                info!("Loaded map: {:?}", map);

                // Additionally, we can access Tiled items using the TiledMapStorage
                // component: we can retrieve Tiled items entity and access
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
        );
}
```

### More Examples

For more advanced use cases, such as loading worlds, chunking, custom properties, or integrating with physics, see the [examples directory](https://github.com/adrien-bon/bevy_ecs_tiled/tree/main/examples/README.md) in the repository.

You can also refer to the [API documentation](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/) for details on all available components and configuration options.
