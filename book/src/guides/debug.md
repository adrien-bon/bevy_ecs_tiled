# Debug your project

## `bevy-inspector-egui`

This may be obvious but this plugin is a must have for debugging.

Just add the required dependency in `Cargo.toml`:

```toml
[dependencies]
bevy-inspector-egui = "0.25.2"
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

## `TiledMapDebugPlugin`

`bevy_ecs_tiled` provides a debug plugin that displays a gizmos where Tiled object are spawned.

To use it, you just have to add the plugin to your application:

```rust,no_run
use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;

fn main() {
    App::new()
        .add_plugins(TiledMapDebugPlugin::default())
        .run();
}
```

More informations in the [API reference](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/debug/index.html).

## Physics

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
