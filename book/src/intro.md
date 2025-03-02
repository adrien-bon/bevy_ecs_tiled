
[`bevy_ecs_tiled`](https://github.com/adrien-bon/bevy_ecs_tiled) is a [Bevy](https://bevyengine.org/) plugin for working with 2D tilemaps created with the [Tiled map editor](https://www.mapeditor.org/).

It relies upon the official [Tiled Rust bindings](https://github.com/mapeditor/rs-tiled) to parse and load Tiled map files and the [`bevy_ecs_tilemap` crate](https://github.com/StarArawn/bevy_ecs_tilemap) to perform rendering.

It aims to provide a simple and ergonomic workflow by using Tiled as an editor when working on Bevy 2D games.

## Features

- Support for several kind of maps: orthogonal, isometric or hexagonal maps, finite or infinite layers, with external or embedded tilesets, using atlases or several images.
- Support various Tiled features: animated tiles, images layers, tile objects or [Tiled world](https://doc.mapeditor.org/en/stable/manual/worlds/) when a single map is not enough.
- Each Tiled item, such as layer, tile or object, is represented by a Bevy entity and everything is organized under a Bevy hierarchy: layers are children of the Tiled map entity, tiles and objects are children of these layers. `Visibility` and `Transform` are automatically propagated down the hierarchy.
- Easily control how to spawn and despawn maps. Use Bevy events and observers to customize how your scene is spawned or notify you when the map is actually loaded and ready to use.
- Build your map in Tiled and let the plugin take care of the rest:
  - Automatically spawn [Rapier](https://rapier.rs/) or [Avian](https://github.com/Jondolf/avian) physics colliders on tiles or objects.
  - Use [Tiled custom properties](https://doc.mapeditor.org/en/stable/manual/custom-properties/) to automatically insert your own components on objects, tiles or layers.
- Hot-reloading: work on your map in Tiled and see it update in Bevy without having to re-compile / restart your game.
