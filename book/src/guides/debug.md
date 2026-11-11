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

- [`TiledDebugObjectsPlugin`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/debug/objects/index.html): outline and position of each Tiled object;
- [`TiledDebugTilesPlugin`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/debug/tiles/index.html): `bevy_ecs_tilemap` index (`TilePos`) of each tile;
- [`TiledDebugMapAabbPlugin`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/debug/map_aabb/index.html): axis-aligned bounding box (AABB) of each map;
- [`TiledDebugWorldChunkPlugin`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/debug/world_chunk/index.html): maps boundaries and chunking area of each world (requires the `world` feature);
- [`TiledDebugAxisPlugin`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/debug/axis/index.html): axes at the world origin;
- [`TiledDebugTogglePlugin`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/debug/struct.TiledDebugTogglePlugin.html): toggle all debug gizmos using a keyboard key.

Most plugins take a configuration, which is inserted as a `Resource`: you can also update it at runtime.

### Objects

The `TiledDebugObjectsPlugin` draws, for each object, its outline and an arrow pointing to its position (ie. its `Transform`).
Colors are cycled from a list, so neighbor objects can be told apart.
It is useful to check objects placement, for instance on isometric maps or after a rotation.

```rust,no_run
use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;

fn main() {
    App::new()
        .add_plugins(TiledDebugObjectsPlugin(TiledDebugObjectsConfig {
            // Draw all objects in white, with a longer arrow
            objects_colors_list: vec![Color::WHITE],
            arrow_length: Vec2::new(-30., 30.),
        }))
        .run();
}
```

### Tiles

The `TiledDebugTilesPlugin` draws the `TilePos` of each tile above it.
Note that `bevy_ecs_tilemap` origin is the bottom-left tile and its Y axis points up, whereas Tiled origin is the top-left tile.

On maps with small tiles, indexes may overlap and become hard to read: reduce the text size using `TiledDebugTilesConfig`.
Since text gizmos are drawn using lines, you can also reduce the lines width of the `TiledDebugGizmos` group (see [below](#toggle-and-configure-debug-gizmos)):

```rust,no_run
use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;

fn main() {
    App::new()
        .add_plugins(TiledDebugTilesPlugin(TiledDebugTilesConfig {
            text_size: 5.,
            ..default()
        }))
        .add_systems(Startup, |mut config_store: ResMut<GizmoConfigStore>| {
            config_store.config_mut::<TiledDebugGizmos>().0.line.width = 1.;
        })
        .run();
}
```

> **Note:**
> One text is drawn per tile, every frame: on large maps, this plugin can be expensive.

### Map bounding boxes

The `TiledDebugMapAabbPlugin` draws two rectangles for each map:

- in orange, the AABB of the tiles grid as computed by `bevy_ecs_tilemap` (ie. where tiles are actually rendered);
- in aqua, the AABB obtained by converting the map corners from Tiled coordinates to world space.

Both rectangles should overlap, whatever the map orientation or `TilemapAnchor`.
If they do not, there is probably an issue with coordinates conversion: objects and image layers may not be positioned where you expect them.
This plugin is especially useful when checking map placement and anchoring.

> **Note:**
> For infinite maps, the AABB only covers the chunks containing tiles: objects located outside of these chunks will lie outside of the AABB.

You can change the colors, or hide one of the rectangles by setting its color to `None`, using the `TiledDebugMapAabbConfig`:

```rust,no_run
use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;

fn main() {
    App::new()
        .add_plugins(TiledDebugMapAabbPlugin(TiledDebugMapAabbConfig {
            // Only display the AABB computed from Tiled coordinates
            tilemap_aabb_color: None,
            tiled_aabb_color: Some(Color::WHITE),
        }))
        .run();
}
```

### World chunks

The `TiledDebugWorldChunkPlugin` draws, for each world:

- a rectangle around each map of the world, colors being cycled from a list;
- if `TiledWorldChunking` is set, a red rectangle centered on the camera: maps overlapping this area are spawned, the others are despawned.

It helps tuning chunking size and checking maps alignment within a world.
Set `world_chunk_color` to `None` or `maps_colors_list` to an empty list in `TiledDebugWorldChunkConfig` to hide one or the other.

### Axis

The `TiledDebugAxisPlugin` draws the X (red) and Y (green) axes at the world origin.
It does not have any configuration: use it as a reference when positioning maps, worlds or anchors.

### Toggle and configure debug gizmos

All debug plugins draw their gizmos using the `TiledDebugGizmos` [gizmo config group](https://docs.rs/bevy/latest/bevy/gizmos/config/trait.GizmoConfigGroup.html).
When this group is disabled, debug gizmos are hidden and debug systems do not run at all.

The easiest way to toggle debug gizmos is to choose a keyboard key for the `TiledDebugTogglePlugin`, which is part of `TiledDebugPluginGroup` but does not use any key by default:

```rust,no_run
use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;

fn main() {
    App::new()
        .add_plugins(TiledDebugPluginGroup.set(TiledDebugTogglePlugin {
            key: Some(KeyCode::F2),
        }))
        .run();
}
```

If you add debug plugins individually, add `TiledDebugTogglePlugin` as well.

You can also configure debug gizmos (enabled, lines width, render layers...) from your own systems using the `GizmoConfigStore` resource, and use the `tiled_debug_gizmos_enabled` run condition for your own debug systems:

```rust,no_run
use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;

fn hide_debug_gizmos(mut config_store: ResMut<GizmoConfigStore>) {
    let (config, _) = config_store.config_mut::<TiledDebugGizmos>();
    config.enabled = false;
}
```

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
