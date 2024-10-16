# bevy_ecs_tiled

[![Crates.io](https://img.shields.io/crates/v/bevy_ecs_tiled)](https://crates.io/crates/bevy_ecs_tiled)
[![docs](https://docs.rs/bevy_ecs_tiled/badge.svg)](https://docs.rs/bevy_ecs_tiled/)
[![license](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/adrien-bon/bevy_ecs_tiled/blob/main/LICENSE)
[![Crates.io](https://img.shields.io/crates/d/bevy_ecs_tiled)](https://crates.io/crates/bevy_ecs_tiled)
[![Following released Bevy versions](https://img.shields.io/badge/Bevy%20tracking-released%20version-lightblue)](https://bevyengine.org/learn/quick-start/plugin-development/#main-branch-tracking)

[`bevy_ecs_tiled`](https://github.com/adrien-bon/bevy_ecs_tiled) is a [Bevy](https://bevyengine.org/) plugin for working with 2D tilemaps created with the [Tiled](https://www.mapeditor.org/) map editor.

It relies upon:

- the official [Tiled Rust bindings](https://github.com/mapeditor/rs-tiled) to load Tiled map files
- the [`bevy_ecs_tilemap` crate](https://github.com/StarArawn/bevy_ecs_tilemap) to perform rendering

Each tile or object is represented by a Bevy entity:

- layers are children of the tilemap entity
- tiles and objects are children of layers

`Visibility` and `Transform` are inherited: map -> layer -> tile / object

![screenshot](./res/screenshot.gif)

## Features

- Orthogonal, isometric and hexagonal maps
- Finite and infinite maps
- Embedded and separate tileset
- Easily spawn / despawn maps
- Animated tiles
- Rapier and Avian colliders added from tilesets and object layers (`rapier` or `avian` feature flag)
- Tiled custom properties mapped to Bevy components (`user_properties` feature flag)

## Documentation

This crate is documented in three places:

- The [`bevy_ecs_tiled` book](https://adrien-bon.github.io/bevy_ecs_tiled/) with design explanations, how-to guides and migrations guides.
- The [API reference](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/)
- The [examples folders](./examples/README.md), for concrete use cases.

There is notably a [FAQ](https://adrien-bon.github.io/bevy_ecs_tiled/FAQ.html) that will hopefully answer most of your questions.

Good reading!

## Getting started

Add dependencies to your `Cargo.toml` file:

```toml
[dependencies]
bevy = "0.14"
bevy_ecs_tiled = "0.4"
bevy_ecs_tilemap = "0.14"
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
    commands.spawn(Camera2dBundle::default());

    // Load the map: ensure any tile / tileset paths are relative to assets/ folder
    let map_handle: Handle<TiledMap> = asset_server.load("map.tmx");

    // Spawn the map with default options
    commands.spawn(TiledMapHandle(map_handle));
}
```

Please note that you should have the `map.tmx` file in your local `assets/` folder, as well as required dependencies (for instance, associated tilesets).

You can customize various settings about how to load the map by inserting the [`TiledMapSettings`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/components/struct.TiledMapSettings.html) component on the map entity.

Also, you can browse the [examples](https://github.com/adrien-bon/bevy_ecs_tiled/tree/main/examples/README.md) for more advanced use cases.

## Bevy Compatibility

|bevy|bevy_ecs_tilemap|bevy_ecs_tiled|
|---|---|---|
|0.14|0.14|0.3 - 0.4|
|0.13|main@e4f3cc6|branch 0.2|
|0.12|0.12|0.1|

## Assets credits

- [colored tiles](./assets/tiles/): orthogonal tileset from [Steve Pryde](https://github.com/stevepryde), licensed under [CC0 1.0](https://creativecommons.org/publicdomain/zero/1.0/)
- [drjamgo_hex_16x16](https://opengameart.org/content/basic-hex-tile-set-16x16): an hexagonal "pointy-top" tileset from [Dr. Jango](https://opengameart.org/users/dr-jamgo), licensed under [CC0 1.0](https://creativecommons.org/publicdomain/zero/1.0/)
- [simple hex flat top](https://opengameart.org/content/simple-flat-top-hexagonal-tiles): an hexagonal "flat-top" tileset from [All things hex](https://opengameart.org/users/all-things-hex), licensed under [CC0 1.0](https://creativecommons.org/publicdomain/zero/1.0/)
- [kenney-sketch-desert](https://kenney.nl/assets/sketch-desert): an isometric tileset from [Kenney](https://kenney.nl/), licensed under [CC0 1.0](https://creativecommons.org/publicdomain/zero/1.0/)

## Contributing

If you can contribute, please do!

If you would like to contribute but don't know where to start, [read this section in the book](https://adrien-bon.github.io/bevy_ecs_tiled/misc/contributing.html).

## LICENSE

This work is licensed under the MIT license.

`SPDX-License-Identifier: MIT`
