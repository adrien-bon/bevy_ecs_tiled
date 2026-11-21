## Overview

The **demo_platformer** example is a comprehensive showcase of `bevy_ecs_tiled` capabilities in a practical game scenario. It demonstrates how to build a small 2D platformer game using Tiled maps and custom properties to define game logic directly in the editor.

It was heavily inspired by the [Bevy 2D template](https://github.com/TheBevyFlock/bevy_new_2d).

## Key Concepts Illustrated

### 1. **User Properties Integration**

This example showcases the power of `bevy_ecs_tiled`'s custom properties feature by using Tiled to define game data.

Game elements are represented as Bevy entities with custom components that are automatically loaded from Tiled:

- Add marker components to objects or layers in the editor (e.g., [`PlayerSpawnPoint`](./player.rs), [`Enemy`](./enemy.rs), [`PatrolRoute`](./patrol.rs), [`TriggerZone`](./trigger.rs), [`OneWayPlatform`](./platform.rs) or [`DynamicProp`](./prop.rs))
- Let the plugin automatically instantiate components on map load
- Write your own game logic associated to these components (using `On<Add>` observers or regular query)

### 2. **Physics Integration**

The example demonstrates physics collider spawning for:

- Platform tiles and objects (static colliders)
- Dynamic enemies and player character with an associated [controller](./controller.rs)
- Dynamic [props](./prop.rs) (crates, a bomb and a weight) placed in Tiled as tile objects: their colliders are created from the collision shapes of their tile and they only need the `DynamicProp` user property to fall and be pushed around

The [level](./level.rs) also shows how to tune colliders creation using `TiledPhysicsSettings`: only some objects get a collider and tiles colliders are split in chunks. Map colliders are made static, except for the ones which already have a rigid body, such as our props.

It also leverages `Avian` physics integration to [trigger](./trigger.rs) game logic events such as teleportation or dead zones when the player enters a certain zone.

### 3. **One-way platforms**

Floating platforms are drawn on a dedicated tiles layer, which has the [`OneWayPlatform`](./platform.rs) user property: the player can jump through them from below, land on top of them and drop through them.

Colliders are automatically created from the tiles collision shapes, we only tag them and use `Avian` collision hooks to ignore their contacts when needed. This part is adapted from the [`one_way_platform_2d`](https://github.com/avianphysics/avian/blob/main/crates/avian2d/examples/one_way_platform_2d.rs) Avian example.

### 4. **Builtin minimap**

Using Bevy `RenderLayers` we also present a working [minimap](./minimap.rs) system.

### 5. **Other Tiled features**

The map also uses an infinite tiles layer, animated tiles and an image layer with a parallax effect, which follows the camera thanks to the `TiledParallaxCamera` component.

## Usage

You can use the following command to run the example:

```bash
cargo run --example demo_platformer --features="avian user_properties"
```

Add the `debug` feature to display physics colliders and an FPS counter, and to toggle a world inspector with `F1`:

```bash
cargo run --example demo_platformer --features="avian user_properties debug"
```

Also, if you load the map with Tiled editor, don't forget to load the custom types export file as well: `./exports/demo_platformer_types.json`. This file is generated again every time the example is run.

### Controls

| Action                          | Keys                                 |
| ------------------------------- | ------------------------------------ |
| Move left / right               | `A` / `D` or `←` / `→`               |
| Jump                            | `Space`, `W` or `↑`                  |
| Drop through a one-way platform | Hold `S` or `↓`, then press jump     |

## Related Resources

- [Tiled Custom Properties Guide](https://adrien-bon.github.io/bevy_ecs_tiled/guides/properties.html)
- [Physics Integration Guide](https://adrien-bon.github.io/bevy_ecs_tiled/guides/physics.html)
- [API Documentation](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/)