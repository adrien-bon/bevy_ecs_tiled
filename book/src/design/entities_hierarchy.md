# Entities Hierarchy and Marker Components

When a map is loaded, it spawns **many** entities: for the map, for layers, for tiles, for objects, for colliders, and more.  
To keep things organized, these entities are structured in a [parent/child hierarchy](https://bevy-cheatbook.github.io/fundamentals/hierarchy.html), and every entity has an associated marker component to help with queries and system targeting.

Using a hierarchy also enables inheritance of certain components from top-level entities down the tree.  
For example, if you change the [`Visibility`](https://docs.rs/bevy/latest/bevy/render/view/visibility/enum.Visibility.html) of a layer entity, it will automatically apply to all entities below it in the hierarchy, such as tiles or objects.

---

## Hierarchy Overview

### World

When loading a `.world` asset, you get a [`TiledWorld`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/tiled/world/struct.TiledWorld.html) entity at the top of the tree, which holds a handle to the [`TiledWorldAsset`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/tiled/world/asset/struct.TiledWorldAsset.html) corresponding to your `.world` asset, along with all settings that apply to it.

### Map

When loading a `.tmx` asset, you get a [`TiledMap`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/tiled/map/struct.TiledMap.html) entity at the top of the tree, which holds a handle to the [`TiledMapAsset`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/tiled/map/asset/struct.TiledMapAsset.html) corresponding to your `.tmx` asset, plus all map-level settings.

If working with a Tiled world, you will have several maps, each as a child of the top-level world entity.

Note that all map-level settings can also be added to the [`TiledWorld`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/tiled/world/struct.TiledWorld.html) entity as they will be propagated down to the underlying maps.

### Layers

Below the map, you have the layers.  
Layer entities are identified by the generic [`TiledLayer`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/tiled/layer/enum.TiledLayer.html) component whose value can help distinguish between layer types.

### Objects

Objects are direct children of their parent [`TiledLayer::Object`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/tiled/layer/enum.TiledLayer.html#variant.Objects).  
They are identified by the [`TiledObject`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/tiled/object/enum.TiledObject.html) marker.

Note that in case of a [`TiledObject::Tile`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/tiled/object/enum.TiledObject.html#variant.Tile), the object entity will have another child entity with a [`TiledObjectVisualOf`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/tiled/object/struct.TiledObjectVisualOf.html) marker component, holding the actual visual of the object: a `Sprite` and eventually a [`TiledAnimation`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/tiled/animation/struct.TiledAnimation.html).

### Tiles

Tiles have a slightly more complex structure:

- Below each [`TiledLayer::Tiles`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/tiled/layer/enum.TiledLayer.html#variant.Tiles), there is one [`TiledTilemap`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/tiled/tile/struct.TiledTilemap.html) per tileset used in the map. It notably holds the `TilemapBundle` from `bevy_ecs_tilemap`.
- Below these tilemaps, you find the actual [`TiledTile`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/tiled/tile/struct.TiledTile.html) entities, each corresponding to a tile in the layer for a given tileset.

### Physics Colliders

At the bottom of the hierarchy, you find physics colliders.  
They are spawned as children of their "source" (either a [`TiledLayer::Tiles`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/tiled/layer/enum.TiledLayer.html#variant.Tiles) or a [`TiledObject`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/tiled/object/enum.TiledObject.html)) and can be identified using the [`TiledColliderOrigin`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/physics/collider/enum.TiledColliderOrigin.html) marker component.

Entities responsible for spawning a physics collider will have a [`TiledColliders`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/physics/collider/struct.TiledColliders.html) component containing a list of attached colliders.

---

## Transform and Visibility Propagation

Bevy automatically propagates [`Transform`](https://docs.rs/bevy/latest/bevy/transform/components/struct.Transform.html) and [`Visibility`](https://docs.rs/bevy/latest/bevy/render/view/visibility/enum.Visibility.html) components down the entity hierarchy.

**In practice:**  
If you change one of these components for a top-level entity (e.g., a layer), it will propagate down and apply to all child entities. For example:

- Adding `Visibility::Hidden` to an object layer will hide all objects in that layer.
- Moving an object layer will also move all objects it contains.

**Special case for tiles:**  
Tiles are not rendered as individual entities, but as part of a "chunk" of several tiles for performance reasons.  
Each individual tile entity does **not** have its own [`Transform`](https://docs.rs/bevy/latest/bevy/transform/components/struct.Transform.html) or [`Visibility`](https://docs.rs/bevy/latest/bevy/render/view/visibility/enum.Visibility.html) component.  
Instead, these components are propagated down to the [`TiledTilemap`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/tiled/tile/struct.TiledTilemap.html), and `bevy_ecs_tilemap` handles updating the corresponding tile chunks.

> **Important:**  
> Even though you technically could, you should **not** add [`Transform`](https://docs.rs/bevy/latest/bevy/transform/components/struct.Transform.html) or [`Visibility`](https://docs.rs/bevy/latest/bevy/render/view/visibility/enum.Visibility.html) components to individual tile entities:
> - It will not have the intended effect—everything is handled at the tilemap level.
> - It may hurt performance significantly.
