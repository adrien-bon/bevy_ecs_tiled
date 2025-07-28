# Debug your project

## Logging

Bevy uses the `tracing` crate for logging, which is very powerful for debugging and profiling.  
You can find more information in the [official documentation](https://docs.rs/tracing/).

To get detailed information about what's happening in your app, set the `RUST_LOG` environment variable to `trace`:

```sh
RUST_LOG=trace cargo run
```

This will show all logs at the `trace` level, but it can be very verbose.  
To filter logs and only display information from `bevy_ecs_tiled`, use:

```sh
RUST_LOG=bevy_ecs_tiled=trace cargo run
```

This will only display logs from the `bevy_ecs_tiled` crate at the `trace` level.

---

## `bevy_ecs_tiled` Debug Plugins

When the `debug` feature is enabled, `bevy_ecs_tiled` provides several debug plugins to help visualize and inspect your maps.

You can enable all debug plugins at once by adding the `TiledDebugPluginGroup` to your app:

```rust,no_run
use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;

fn main() {
    App::new()
        .add_plugins(TiledDebugPluginGroup)
        .run();
}
```

Or add them individually as needed:

- [`TiledDebugObjectsPlugin`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/debug/objects/index.html): Displays an `arrow_2d` and a polyline outline `Gizmos` at each Tiled object's position and shape.
- [`TiledDebugTilesPlugin`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/debug/tiles/index.html): Shows the `bevy_ecs_tilemap` tile index (`TilePos`) above each tile.
- [`TiledDebugWorldChunkPlugin`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/debug/world_chunk/index.html): Draws a `Gizmos` rectangle for each map boundary and world render chunk.
- [`TiledDebugAxisPlugin`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/debug/axis/index.html): Displays a `Gizmos` axes marker at the world origin.

For more details, see the [API reference](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/debug/index.html).

---

## Third-Party Debugging Tools

### `bevy-inspector-egui`

This plugin is highly recommended for debugging and inspecting your game world.

Add the dependency to your `Cargo.toml`:

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
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin { enable_multipass_for_primary_context: true })
        .add_plugins(WorldInspectorPlugin::new())
        .run();
}
```

Now, you can browse and edit components from all entities spawned in your game in real time.

More information is available on the [project's GitHub page](https://github.com/jakobhellermann/bevy-inspector-egui).

---

### Physics Plugins

Both Avian and Rapier provide their own debug visualization plugins, which are invaluable when working with colliders and physics.

**Avian Physics Debugging:**

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

**Rapier Physics Debugging:**

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

> **Note:**  
> For Rapier, you must enable either the `debug-render-2d` feature on the `bevy_rapier2d` crate or the `rapier_debug` feature on `bevy_ecs_tiled`.
