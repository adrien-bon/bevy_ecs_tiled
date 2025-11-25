## Overview

The **demo_platformer** example is a comprehensive showcase of `bevy_ecs_tiled` capabilities in a practical game scenario. It demonstrates how to build a small 2D platformer game using Tiled maps and custom properties to define game logic directly in the editor.

It was heavilly inspired by the [Bevy 2D template](https://github.com/TheBevyFlock/bevy_new_2d).

## Key Concepts Illustrated

### 1. **User Properties Integration**

This example showcases the power of `bevy_ecs_tiled`'s custom properties feature by using Tiled to define game data.

Game elements are represented as Bevy entities with custom components that are automatically loaded from Tiled:

- Add marker components to objects in the editor (e.g., [`PlayerSpawnPoint`](./player.rs), [`Enemy`](./enemy.rs), [`PatrolRoute`](./patrol.rs) or [`Trigger`](./trigger.rs))
- Let the plugin automatically instantiate components on map load
- Write your own game logic associated to these components (using `On<Add>` observers or regular query)

### 2. **Physics Integration**

The example demonstrates physics collider spawning for:

- Platform tiles and objects (static colliders)
- Dynamic enemies and player character with an associated controller

It also leverage `Avian` physics integration to [trigger](./trigger.rs) game logic events such as teleportation or dead zones when the player enter a certain zone.

### 3. **Builtin minimap**

Using Bevy `RenderLayers` we also present a working [minimap](./minimap.rs) system.

## Usage

You can use the following command to run the example:

```bash
cargo run --example demo_platformer --features="avian user_properties"
```

Also, if you load the map with Tiled editor, don't forget to load the custom types export file as well: `./assets/demo_platformer/demo_platformer_types.json`.

## Related Resources

- [Tiled Custom Properties Guide](https://adrien-bon.github.io/bevy_ecs_tiled/guides/properties.html)
- [Physics Integration Guide](https://adrien-bon.github.io/bevy_ecs_tiled/guides/physics.html)
- [API Documentation](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/)