## Overview

The **demo_topdown** example is a small top-down game where the player is moved with the mouse, on a grid and without any physics engine. As much as possible is defined in Tiled: the code only gives a meaning to the custom properties set on tiles, layers and objects.

It complements the [demo_platformer](../demo_platformer/) example, which focuses on objects properties and physics.

## Key Concepts Illustrated

### 1. **User Properties on Tiles**

Custom properties are added to the tiles of the [tileset](../../assets/demo_topdown/tilesets/sokoban.tsx): every tile of a map using one of them automatically gets the associated components. Nothing has to be configured on the maps themselves.

- [`Obstacle`](./navigation.rs): tiles which can not be walked on (walls, crates, gates)
- [`Destructible`](./interaction.rs): tiles the player can break
- [`Button`](./interaction.rs) and [`Gate`](./interaction.rs): they hold a `Channel` enum, to link a button with the gates it opens

Layers and objects also have their own properties: [`HighlightLayer`](./highlight.rs) on tiles layers, [`SpawnPoint`](./level.rs) and [`Door`](./level.rs) on point objects.

### 2. **Picking**

Tiles are rendered by their tilemap and can not be picked by Bevy out of the box. The example adds a small [picking backend](./picking.rs) which reports the tiles below the pointer: tiles entities then receive regular pointer events (`PointerClick`, `PointerOver`, ...) which can be observed as for any other entity.

### 3. **Pathfinding**

A [navigation grid](./navigation.rs) is built from the tiles entities and their `Obstacle` component, and an A* algorithm finds the path to the clicked tile. Since the grid only relies on tiles entities, it is automatically updated when the map is modified.

### 4. **Modifying the Map**

Tiles entities only hold data, so the map can be [modified while the game is running](./interaction.rs):

- breaking a crate despawns its tile
- walking on a button changes the tile ID of the button, so another tile is displayed, and despawns the gates of the same channel

Tilemaps and the navigation grid are updated accordingly.

### 5. **Switching Between Maps**

When the player walks on a [`Door`](./level.rs), the map asset of the map entity is replaced by the one named in the door properties, and the player arrives on the `SpawnPoint` whose name is also given by the door. This is how the player enters and leaves the house.

Since the map we leave is despawned, its state is lost: crates and gates are back the next time it is loaded.

### 6. **Custom Tilemap Material**

A [custom material](./highlight.rs) replaces the default one for the tilemaps of the layers flagged in Tiled. Its [shader](../../assets/demo_topdown/highlight.wesl) highlights the tile below the pointer and the tile the player is walking to.

## Usage

You can use the following command to run the example:

```bash
cargo run --example demo_topdown --features="user_properties"
```

Add the `debug` feature to toggle a world inspector with `F1`:

```bash
cargo run --example demo_topdown --features="user_properties debug"
```

Also, if you load the maps with Tiled editor, either open the [Tiled project](../../assets/demo_topdown/demo_topdown.tiled-project) or load the custom types export file: `./exports/demo_topdown_types.json`. This file is generated again every time the example is run.

### Controls

| Action                 | Input                    |
| ---------------------- | ------------------------ |
| Walk to a tile         | Left click on the ground |
| Break a crate          | Left click on the crate  |
| Press a button         | Walk on it               |
| Enter or leave a house | Walk through its door    |

## Related Resources

- [Tiled Custom Properties Guide](https://adrien-bon.github.io/bevy_ecs_tiled/guides/properties.html)
- [Modify a map at runtime](https://adrien-bon.github.io/bevy_ecs_tiled/guides/dynamic_maps.html)
- [Spawn and reload maps](https://adrien-bon.github.io/bevy_ecs_tiled/guides/spawn_reload.html)
- [API Documentation](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/)
