# bevy_ecs_tiled

[![Crates.io](https://img.shields.io/crates/v/bevy_ecs_tiled)](https://crates.io/crates/bevy_ecs_tiled)
[![docs](https://docs.rs/bevy_ecs_tiled/badge.svg)](https://docs.rs/bevy_ecs_tiled/)
[![license](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/stevepryde/bevy_ecs_tiled/blob/main/LICENSE)
[![Crates.io](https://img.shields.io/crates/d/bevy_ecs_tiled)](https://crates.io/crates/bevy_ecs_tiled)
[![Following released Bevy versions](https://img.shields.io/badge/Bevy%20tracking-released%20version-lightblue)](https://bevyengine.org/learn/quick-start/plugin-development/#main-branch-tracking)

Plugin for working with 2D tilemaps created with the Tiled map editor.

## Status

** **VERY EARLY BUILD - EXPECT BUGS AND BREAKING CHANGES** **

This crate is currently in a very early state while I build the basics.
I'm new to Bevy (but not new to Rust or game dev) so I'm learning as I go.

The code was originally copied from the Tiled example in `bevy_ecs_tilemap`,
plus the fix from [this PR](https://github.com/StarArawn/bevy_ecs_tilemap/pull/429).

If you can contribute, please do!

If you would like to contribute but don't know where to start, [click here](https://github.com/stevepryde/bevy_ecs_tiled/discussions/1).

## Features

- Finite and infinite maps
- Embedded and separate tileset
- Layers are children of the tilemap entity. Tiles are children of layers.
- Visibility is inherited: map -> layer -> tile
- Spawn/despawn


## Getting started

Add this crate to your Cargo.toml. For Bevy 0.14 this requires a git 
dependency until a compatible release is published for bevy_ecs_tilemap.

```toml
[dependencies]
bevy = "0.14"
bevy_ecs_tiled = "0.3"
bevy_ecs_tilemap = "0.14"
```

Then add the plugin to your app:

```rust
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
    commands.spawn(Camera2dBundle::default());

    // Ensure any tile / tileset paths are relative to assets/
    let map_handle: Handle<TiledMap> = asset_server.load("map.tmx");
    commands.spawn(TiledMapBundle {
        tiled_map: map_handle,
        ..Default::default()
    });
}
```

## Bevy Compatibility

|bevy|bevy_ecs_tilemap|bevy_ecs_tiled|
|---|---|---|
|0.14|0.14|0.3|
|0.13|main@e4f3cc6|branch 0.2|
|0.12|0.12|0.1|


## LICENSE

This work is licensed under the MIT license.

`SPDX-License-Identifier: MIT`