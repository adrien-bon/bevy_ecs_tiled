
[`bevy_ecs_tiled`](https://github.com/adrien-bon/bevy_ecs_tiled) is a [Bevy](https://bevyengine.org/) plugin for working with 2D tilemaps created with the [Tiled](https://www.mapeditor.org/) map editor.

It relies upon:

- the official [Tiled Rust bindings](https://github.com/mapeditor/rs-tiled) to parse Tiled maps
- the [`bevy_ecs_tilemap` crate](https://github.com/StarArawn/bevy_ecs_tilemap) to perform rendering

Each tile or object is represented by a Bevy entity:

- layers are children of the tilemap entity
- tiles and objects are children of layers

`Visibility` and `Transform` are inherited: map -> layer -> tile / object
