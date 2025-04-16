# Debug your project

## Logging

Bevy uses the `tracing` crate for logging, which is very powerful in debugging and profiling, you can find more information in the [official documentation](https://docs.rs/tracing/).

We recommend to enable the `trace` level in your application to get more informations about what's happening, just set the `RUST_LOG` environment variable to `trace`:

```sh
RUST_LOG=trace cargo run
```

But this will be very verbose, so you can also filter the logs to only display the informations you need:

```sh
RUST_LOG=bevy_ecs_tiled=trace cargo run
```

This will only display logs from the `bevy_ecs_tiled` crate in `trace` level.

## `bevy_ecs_tiled` debug plugins

When the `debug` feature is enabled, `bevy_ecs_tiled` provides several debug plugins.

You can easily turn them all by adding the `TiledDebugPluginGroup` to your app :

```rust,no_run
use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;

fn main() {
    App::new()
        .add_plugins(TiledDebugPluginGroup)
        .run();
}
```

Or add them individually :

- [`TiledDebugObjectsPlugin`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/debug/objects/index.html) : display a Bevy `Gizmos` to indicate where Tiled objects are
- [`TiledDebugTilesPlugin`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/debug/tiles/index.html) : display the `bevy_ecs_tilemap` index, ie. `Tilepos` on each tile
- [`TiledDebugWorldChunkPlugin`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/debug/world_chunk/index.html) : display a Bevy `Gizmos` for each map boundary and world render chunk
- [`TiledDebugAxiskPlugin`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/debug/axis/index.html) : display a Bevy `Gizmos` to localize the origin

More informations in the [API reference](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/debug/index.html).

## 3rdparty

### `bevy-inspector-egui`

This may be obvious but this plugin is a must have for debugging.

Just add the required dependency in `Cargo.toml`:

```toml
[dependencies]
bevy-inspector-egui = "0.30"
```

Then add the `WorldInspectorPlugin` to your application:

```rust,no_run
use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

fn main() {
    App::new()
        .add_plugins(WorldInspectorPlugin::new())
        .run();
}
```

Now, you can browse components from all entities spawned in your game.

More informations on the project [github page](https://github.com/jakobhellermann/bevy-inspector-egui).

### Physics plugins

Both Avian and Rapier provide their own way of debugging.
It can be very useful, especially when working with colliders.
Note that physics debugging is enabled by default in all `bevy_ecs_tiled` examples using physics.

To enable physics debugging in Avian, you need to simply add the corresponding plugin:

```rust,no_run
use bevy::prelude::*;
use avian2d::prelude::*;

fn main() {
    App::new()
        // Add Avian regular plugin
        .add_plugins(PhysicsPlugins::default().with_length_unit(100.0))
        // Add Avian debug plugin
        .add_plugins(PhysicsDebugPlugin::default())
        .run();
}
```

For Rapier, you need to enable a debug plugin:

```rust,no_run
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

fn main() {
    App::new()
        // Add Rapier regular plugin
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        // Add Rapier debug plugin
        .add_plugins(RapierDebugRenderPlugin::default())
        .run();
}
```

And you also need to enable either the `debug-render-2d` feature on `bevy_rapier2d` crate or the `rapier_debug` feature on `bevy_ecs_tiled`
