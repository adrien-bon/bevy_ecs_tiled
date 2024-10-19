# Entities tree and marker components

When a map is loaded, it spawns **a lot** of entities: for the map, for layers, for tiles, for objects, for colliders, ...
These entites are organized in a [parent / child hierarchy](https://bevy-cheatbook.github.io/fundamentals/hierarchy.html).

It notably brings the capability to inherit some of the component down the tree.
For instance, if you change the `Visibility` of an entity, it will automatically apply to all entities below in the hierarchy.

It also helps to keep things nice and clean.

## Tree hierarchy

### Map

At the top of the tree, there is the map.
It notably holds the [`TiledMapHandle`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/struct.TiledMapHandle.html) pointing to your .TMX file and all the settings that apply to it.
It can be easily identified using a dedicated marker component: [`TiledMapMarker`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/components/struct.TiledMapMarker.html).

### Layers

Below the map, we have the layers.
They can be of different kinds, which each have their own marker component:

- [`TiledMapObjectLayer`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/components/struct.TiledMapObjectLayer.html): for objects layer.
- [`TiledMapTileLayer`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/components/struct.TiledMapTileLayer.html): for tiles layer.
- [`TiledMapGroupLayer`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/components/struct.TiledMapGroupLayer.html): for group layer (not supported for now).
- [`TiledMapImageLayer`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/components/struct.TiledMapImageLayer.html): for image layer (not supported for now).

All of them are also identified by the same generic marker: [`TiledMapLayer`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/components/struct.TiledMapLayer.html).

### Objects & Tiles

Objects are directly below their [`TiledMapObjectLayer`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/components/struct.TiledMapObjectLayer.html).
They are identified by a [`TiledMapObject`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/components/struct.TiledMapObject.html) marker.

For tiles, it's a little more complicated.
Below the [`TiledMapTileLayer`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/components/struct.TiledMapTileLayer.html), we first have one [`TiledMapTileLayerForTileset`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/components/struct.TiledMapTileLayerForTileset.html) per tileset in the map.
Finally, below these, we find the actual [`TiledMapTile`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/components/struct.TiledMapTile.html) which correspond to every tiles in the layer, for a given tileset.

### Physics colliders

At the end of the hierarchy, we find physics colliders.
They are spawned below they "source", ie. either a tile or an object and can be identified using their marker component: [`TiledColliderMarker`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/physics/collider/struct.TiledColliderMarker.html).
