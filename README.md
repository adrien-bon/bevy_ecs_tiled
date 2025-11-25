# bevy_ecs_tiled

[![Crates.io](https://img.shields.io/crates/v/bevy_ecs_tiled)](https://crates.io/crates/bevy_ecs_tiled)
[![docs](https://docs.rs/bevy_ecs_tiled/badge.svg)](https://docs.rs/bevy_ecs_tiled/)
[![license](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/adrien-bon/bevy_ecs_tiled/blob/main/LICENSE)
[![Crates.io](https://img.shields.io/crates/d/bevy_ecs_tiled)](https://crates.io/crates/bevy_ecs_tiled)
[![Following released Bevy versions](https://img.shields.io/badge/Bevy%20tracking-released%20version-lightblue)](https://bevyengine.org/learn/quick-start/plugin-development/#main-branch-tracking)

[`bevy_ecs_tiled`](https://github.com/adrien-bon/bevy_ecs_tiled) is a [Bevy](https://bevyengine.org/) plugin for working with 2D tilemaps created using the [Tiled map editor](https://www.mapeditor.org/).

It leverages the official [Tiled Rust bindings](https://github.com/mapeditor/rs-tiled) for parsing and loading Tiled map files, and uses the [`bevy_ecs_tilemap` crate](https://github.com/StarArawn/bevy_ecs_tilemap) for efficient rendering.

This plugin aims to provide a simple and ergonomic workflow for integrating Tiled into your Bevy 2D games, letting you focus on game design while handling the technical details of map loading, rendering, and entity management.

---

## Features

- **Comprehensive Map Support:**  
  Load orthogonal, isometric, or hexagonal maps, with finite or infinite layers, using either external or embedded tilesets, atlases, or multiple images.
- **Rich Tiled Feature Integration:**  
  Supports animated tiles, image layers, tile objects, and [Tiled worlds](https://doc.mapeditor.org/en/stable/manual/worlds/) for multi-map projects.
- **Entity-Based Architecture:**  
  Every Tiled item (layer, tile, object, etc.) is represented as a Bevy entity, organized in a clear hierarchy: layers are children of the map entity, tiles and objects are children of their respective layers. `Visibility` and `Transform` are automatically propagated.
- **Flexible Map Lifecycle:**  
  Control how maps are spawned and despawned. Use Bevy events and observers to customize scene setup or react when a map is loaded and ready.
- **Automatic Physics Integration:**  
  Automatically spawn [Rapier](https://rapier.rs/) or [Avian](https://github.com/Jondolf/avian) physics colliders for tiles and objects, with support for custom backends.
- **Custom Properties as Components:**  
  Use [Tiled custom properties](https://doc.mapeditor.org/en/stable/manual/custom-properties/) to automatically insert your own components on objects, tiles, or layers, enabling powerful data-driven workflows.
- **Hot-Reloading:**  
  Edit your maps in Tiled and see updates reflected live in your Bevy game, without recompiling or restarting.

---

## Showcases

**Update your Bevy components directly from Tiled editor:**

![screenshot](./res/showcase_properties.gif)

**Use several maps to build a huge world:**

![screenshot](./res/showcase_world_chunking.gif)

---

## Documentation

This crate is documented in three places:

- The [`bevy_ecs_tiled` book](https://adrien-bon.github.io/bevy_ecs_tiled/) with design explanations, how-to guides and migrations guides.
- The [API reference](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/)
- The [examples folders](./examples/README.md), for concrete use cases.

There is notably a [FAQ](https://adrien-bon.github.io/bevy_ecs_tiled/FAQ.html) that will hopefully answer most of your questions.

---

## Getting started

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
        // Add the bevy_ecs_tiled plugin.
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

For more advanced use cases, such as loading worlds, chunking, custom properties, or integrating with physics, see the [examples directory](https://github.com/adrien-bon/bevy_ecs_tiled/tree/main/examples/README.md) in the repository and notably the [demo_platformer](https://github.com/adrien-bon/bevy_ecs_tiled/tree/main/examples/demo_platformer/).

You can also refer to the [API documentation](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/) for details on all available components and configuration options.

---

## Bevy Compatibility

|bevy|bevy_ecs_tilemap|bevy_ecs_tiled|
|---|---|---|
|0.16|0.16|0.7 - 0.9|
|0.15|0.15|0.5 - 0.6|
|0.14|0.14|0.3 - 0.4|
|0.13|main@e4f3cc6|branch 0.2|
|0.12|0.12|0.1|

---

## Assets credits

- [colored tiles](./assets/tiles/): orthogonal tileset from [Steve Pryde](https://github.com/stevepryde), licensed under [CC0 1.0](https://creativecommons.org/publicdomain/zero/1.0/)
- [drjamgo_hex_16x16](https://opengameart.org/content/basic-hex-tile-set-16x16): an hexagonal "pointy-top" tileset from [Dr. Jango](https://opengameart.org/users/dr-jamgo), licensed under [CC0 1.0](https://creativecommons.org/publicdomain/zero/1.0/)
- [simple hex flat top](https://opengameart.org/content/simple-flat-top-hexagonal-tiles): an hexagonal "flat-top" tileset from [All things hex](https://opengameart.org/users/all-things-hex), licensed under [CC0 1.0](https://creativecommons.org/publicdomain/zero/1.0/)
- [kenney-sketch-desert](https://kenney.nl/assets/sketch-desert): an isometric tileset from [Kenney](https://kenney.nl/), licensed under [CC0 1.0](https://creativecommons.org/publicdomain/zero/1.0/)
- [bevy_logo](https://thebevyflock.github.io/the-bird/birds/alternate-bevy-logo.jpg): an alternative Bevy logo from [Steam Ed Duck](https://www.instagram.com/steamedduck_fest), licensed under [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/)

---

## Contributing

If you can contribute, please do!

If you would like to contribute but don't know where to start, [read this section in the book](https://adrien-bon.github.io/bevy_ecs_tiled/misc/contributing.html).

---

## License

This work is licensed under the MIT license.

`SPDX-License-Identifier: MIT`
