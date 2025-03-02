# Entities hierarchy and marker components

When a map is loaded, it spawns **a lot** of entities: for the map, for layers, for tiles, for objects, for colliders, ...
To keep things nice and tidy, these entites are organized in a [parent / child hierarchy](https://bevy-cheatbook.github.io/fundamentals/hierarchy.html) and every entity has an associated marker component to help with queries.

Using a hierachy also brings the capability to inherit some of the component from top-level entities down the tree.
For instance, if you change the `Visibility` of a layer entity, it will automatically apply to all entities below in the hierarchy such as tiles or objects.

## Hierarchy

### World

When loading a `.world` asset, you will have a [`TiledWorldMarker`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/components/world/struct.TiledWolrdMarker.html) at the top of the tree.

This entity holds the [`TiledWorldHandle`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/world/struct.TiledWorldHandle.html) pointing to your `.world` asset and all the settings that apply to it.

### Map

When loading a single `.tmx` asset, you will have a single [`TiledMapMarker`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/components/struct.TiledMapMarker.html) at the top of the tree.
Otherwise, if working with a Tiled world, you will have several maps which are children of the top-level world.

This entity holds the [`TiledMapHandle`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/struct.TiledMapHandle.html) component pointing to your `.tmx` file and all the settings that apply to it.

### Layers

Below the map, we have the layers.
They can be of different kinds, which each have their own marker component:

- [`TiledMapObjectLayer`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/map/components/struct.TiledMapObjectLayer.html): for objects layer.
- [`TiledMapTileLayer`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/map/components/struct.TiledMapTileLayer.html): for tiles layer.
- [`TiledMapImageLayer`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/map/components/struct.TiledMapImageLayer.html): for image layer.
- [`TiledMapGroupLayer`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/map/components/struct.TiledMapGroupLayer.html): for group layer (not supported for now).

All of them are also identified by the same generic marker: [`TiledMapLayer`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/map/components/struct.TiledMapLayer.html).

### Objects

Objects are directly below their [`TiledMapObjectLayer`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/components/map/struct.TiledMapObjectLayer.html).
They are identified by a [`TiledMapObject`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/components/map/struct.TiledMapObject.html) marker.

### Tiles

For tiles, it's a little more complicated.

Below the [`TiledMapTileLayer`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/components/map/struct.TiledMapTileLayer.html), we first have one [`TiledMapTileLayerForTileset`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/map/components/struct.TiledMapTileLayerForTileset.html) per tileset in the map.
And below these, we find the actual [`TiledMapTile`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/map/components/struct.TiledMapTile.html) which correspond to a tile in the layer, for a given tileset.

### Physics colliders

At the bottom of the hierarchy, we find physics colliders.
They are spawned below they "source", ie. either a tile layer or an object and they can be identified using their marker component: [`TiledColliderMarker`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/physics/collider/struct.TiledColliderMarker.html).

## Transform and Visibility propagation

You can refer to the Bevy cheatbook to get an overview of how [`Transform`](https://bevy-cheatbook.github.io/fundamentals/transforms.html) and [`Visibility`](https://bevy-cheatbook.github.io/fundamentals/visibility.html) propagation works in general.

In two words, it means that if you change one of these components for a top-level entity, for instance a layer, it will propagate down the hierarchy and apply to all the entities below it.
For instance :

- adding the `Visibility::Hidden` component to an object layer will make all objects in it to be hidden
- moving an object layer will also move all objects it contains

However, there is a special case for tiles.
Since they are not rendered individually but using a "chunk" of several tiles, each individual tile does **not** have a `Transform` or `Visibility` component.
We propagate the `Transform` and `Visibility` down to the tilemap and `bevy_ecs_tilemap` take care of the rest to update the corresponding tiles chunk.

Eventhough you could, you should **not** try to add these components to individual tiles:

- it will not do what you think, everything is handled at the tilemap level anyway
- it may hurt performances badly
