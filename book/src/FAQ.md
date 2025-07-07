# FAQ

## What’s the current status of the crate?

The crate is already quite usable, but it is still under active development.

You may encounter bugs, missing features, or breaking changes as the API evolves.  
However, the project follows [semantic versioning](https://semver.org/) and provides a migration guide for each breaking change (for example, when a new Bevy release is supported).

---

## What kind of maps are supported?

Nearly all map types from Tiled are supported:

- Orthogonal
- "Flat-top" hexagonal
- "Pointy-top" hexagonal
- "Diamond" isometric

**Not supported:**  
Isometric "staggered" maps ([see issue #31](https://github.com/adrien-bon/bevy_ecs_tiled/issues/31)).
There are also some limitations for "diamond" isometric maps:

- Colliders may not have the correct shape ([#32](https://github.com/adrien-bon/bevy_ecs_tiled/issues/32))
- Colliders may not always be placed correctly ([#48](https://github.com/adrien-bon/bevy_ecs_tiled/issues/48))

Support for these cases will improve in the future.

---

## I’m using an isometric map and it seems all messed up!

- Make sure you are actually using a "diamond" isometric map, not a "staggered" one (which is not supported).
- For isometric maps, you may need to tweak the [`TilemapRenderSettings`](https://docs.rs/bevy_ecs_tilemap/latest/bevy_ecs_tilemap/map/struct.TilemapRenderSettings.html) component from `bevy_ecs_tilemap` to enable Y-sorting and adjust the chunk size.

See the [isometric maps example](https://github.com/adrien-bon/bevy_ecs_tiled/blob/main/examples/orientation_isometric.rs#L34) for more information.

---

## How do I add physics to my game?

You can automatically spawn physics colliders on tiles or objects using either the [Avian](https://github.com/Jondolf/avian) or [Rapier](https://github.com/dimforge/bevy_rapier) physics backends.

All you need to do is add the appropriate plugin to your app—`bevy_ecs_tiled` handles the rest.

You can control which objects or tiles receive colliders based on their name and by using Tiled’s built-in collision editor.
See the [dedicated physics guide](guides/physics.md) for details.

---

## How can I update my Bevy entities directly from Tiled?

Suppose you’re building a top-down RPG and want to assign properties (like movement cost or walkability) to each tile.

You can declare in your code some `struct` or `enum` to describe these properties then use [Tiled custom properties](https://doc.mapeditor.org/en/stable/manual/custom-properties/) editor to attach these properties to Tiled elements.
The crate will automatically insert the corresponding Bevy `Component` on the Tiled entity when you load the map, making it available for your game logic.

See the [custom properties guide](guides/properties.md) for more information.

---

## How do I enable map hot-reload?

Enable the Bevy `file_watcher` feature in your project.  
With this enabled, `bevy_ecs_tiled` will automatically reload a map or world whenever it is updated in Tiled.

---

## How do I access Tiled "raw" data?

Sometimes you may need to access the underlying Tiled data, such as a `tiled::ObjectData` or `tiled::TileData`.

You have two main options:

- Listen to [map loading events](./design/map_events.md) and use [`TiledEvent<E>`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/tiled/event/struct.TiledEvent.html) helper methods to access Tiled data.
- Retrieve the [`TiledMapStorage`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/tiled/map/storage/struct.TiledMapStorage.html) component from your map entity to get the mapping between Bevy entities and their corresponding Tiled data.

---

## I found a bug! / Feature 'X' is missing! / How do I do 'Y'?

This crate is still a work in progress, so you may encounter bugs, missing features, or have questions about usage.

- Check the [open issues](https://github.com/adrien-bon/bevy_ecs_tiled/issues) to see if your problem is already reported.
- If not, please open a new issue! I try to address every issue as quickly as possible.

**Contributions are very welcome!**  
If you’d like to contribute, please read the [contribution guide](misc/contributing.md) and feel free to open a PR!
