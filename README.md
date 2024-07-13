# bevy_ecs_tiled

Plugin for working with 2D tilemaps created with the Tiled map editor.

## Status

** **VERY EARLY BUILD - EXPECT BUGS AND BREAKING CHANGES** **

This crate is currently in a very early state while I build the basics.
I'm still new to Bevy so I'm learning as I go.

The code was originally copied from the Tiled example in `bevy_ecs_tilemap`,
plus the fix from [this PR](https://github.com/StarArawn/bevy_ecs_tilemap/pull/429).

Contributions are welcome.

## Features

- Finite and infinite maps
- Embedded and separate tileset

## Getting started

Add this crate to your Cargo.toml. For Bevy 0.14 this requires a git 
dependency until a compatible release is published for bevy_ecs_tilemap.

```toml
[dependencies]
bevy = {version = "0.14.0"}
bevy_ecs_tiled = {git = "https://github.com/stevepryde/bevy_ecs_tiled"}
bevy_ecs_tilemap = {git = "https://github.com/StarArawn/bevy_ecs_tilemap"}
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
|0.12|0.12|0.1|
|0.13|main@e4f3cc6|0.2 *|
|0.14|main|main (0.3 *)|

\* These versions are not yet published to crates.io as they are waiting on 
  upstream versions to be published. Use git dependencies instead.

## LICENSE

This work is licensed under the MIT license.

`SPDX-License-Identifier: MIT`