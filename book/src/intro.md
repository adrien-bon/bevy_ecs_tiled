[`bevy_ecs_tiled`](https://github.com/adrien-bon/bevy_ecs_tiled) is a [Bevy](https://bevyengine.org/) plugin for working with 2D tilemaps created using the [Tiled map editor](https://www.mapeditor.org/).

It leverages the official [Tiled Rust bindings](https://github.com/mapeditor/rs-tiled) for parsing and loading Tiled map files, and uses the [`bevy_ecs_tilemap` crate](https://github.com/StarArawn/bevy_ecs_tilemap) for efficient rendering.

This plugin aims to provide a simple and ergonomic workflow for integrating Tiled into your Bevy 2D games, letting you focus on game design while handling the technical details of map loading, rendering, and entity management.

## Features

- **Comprehensive Map Support:**  
  Load orthogonal, isometric, or hexagonal maps, with finite or infinite layers, using either external or embedded tilesets, atlases, or multiple images.
- **Rich Tiled Feature Integration:**  
  Supports animated tiles, image layers, tile objects, and [Tiled worlds](https://doc.mapeditor.org/en/stable/manual/worlds/) for multi-map projects.
- **Entity-Based Architecture:**  
  Every Tiled item (layer, tile, object, etc.) is represented as a Bevy entity, organized in a clear hierarchy: layers are children of the map entity, tiles and objects are children of their respective layers. `Visibility` and `Transform` are automatically propagated.
- **Flexible Map Lifecycle:**  
  Control how maps are spawned and despawned. Use Bevy events and observers to customize scene setup or react when a map is loaded and ready.
- **Automatic Physics Integration:**  
  Automatically spawn [Rapier](https://rapier.rs/) or [Avian](https://github.com/Jondolf/avian) physics colliders for tiles and objects, with support for custom backends.
- **Custom Properties as Components:**  
  Use [Tiled custom properties](https://doc.mapeditor.org/en/stable/manual/custom-properties/) to automatically insert your own components on objects, tiles, or layers, enabling powerful data-driven workflows.
- **Hot-Reloading:**  
  Edit your maps in Tiled and see updates reflected live in your Bevy game, without recompiling or restarting.
