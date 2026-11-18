# API Overview

Where to look in the API, depending on what you want to do.
Everything listed here is in the prelude (`use bevy_ecs_tiled::prelude::*;`), unless a full path is given.

## Crate Layout

| Module | Content |
| --- | --- |
| [`asset`][docs-asset] | Tiled files loaded as Bevy assets: [`TiledMapAsset`][TiledMapAsset], `TiledWorldAsset` and Tiled IDs. |
| [`ecs`][docs-ecs] | Entities and components holding the Tiled data: the source of truth, which you can modify. |
| [`render`][docs-render] | Rendering derived from the `ecs` entities (`render` feature). |
| [`physics`][docs-physics] | Physics colliders derived from the `ecs` entities (`physics` feature). |
| [`properties`][docs-properties] | Tiled custom properties. |
| [`debug`][docs-debug] | Debug gizmos (`debug` feature). |

[docs-asset]: https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/asset/index.html
[docs-ecs]: https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/ecs/index.html
[docs-render]: https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/render/index.html
[docs-physics]: https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/physics/index.html
[docs-properties]: https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/properties/index.html
[docs-debug]: https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/debug/index.html

## I Want To…

| I want to… | Use |
| --- | --- |
| Load a map | Add the [`TiledPlugin`][TiledPlugin], spawn a [`TiledMap`][TiledMap] |
| Load a world | Spawn a [`TiledWorld`][TiledWorld] (`world` feature) |
| Only spawn the maps of a world around the camera | [`TiledWorldStreaming`][TiledWorldStreaming] |
| Reload a map | Insert [`RespawnTiledMap`][RespawnTiledMap], or a new `TiledMap` |
| Place the map | `TilemapAnchor` and `Transform` on the map entity |
| Change the Z offset between layers | [`TiledMapLayerZOffset`][TiledMapLayerZOffset] |
| React when a map, layer or object is spawned | An observer on [`TiledEvent<MapCreated>`][TiledEvent] (or `LayerCreated`, `ObjectCreated`, …) |
| Find the map of a layer, tile or object | [`TiledMapReference`][TiledMapReference], or [`TiledMapHelper`][TiledMapHelper] |
| Find a layer or object entity from its Tiled ID | [`TiledMapStorage`][TiledMapStorage] on the map entity |
| Find the tile at a given position | [`TiledTileLayerStorage`][TiledTileLayerStorage] on the layer entity |
| Know where a tile is displayed | [`TiledMapHelper::tile_relative_position`][TiledMapHelper] |
| Modify the map at runtime | Spawn, modify or despawn [`TiledTile`][TiledTile] and [`TiledObject`][TiledObject] entities (see [the guide](../guides/dynamic_maps.md)) |
| Read raw Tiled data | [`TiledMapAsset`][TiledMapAsset] (`map` field, `tileset()`, `tile()`, `layer()`, `object()`) and the re-exported [`tiled`](https://docs.rs/tiled/latest/tiled/) crate |
| Read custom properties as they are in Tiled | [`TiledProperties`][TiledProperties] |
| Get custom properties as my own components | The `user_properties` feature (see [the guide](../guides/properties.md)) |
| Customize the tilemaps of a layer | An observer on `TiledEvent<TilemapCreated>`, [`TiledTilemap`][TiledTilemap] |
| Add parallax to layers | [`TiledParallaxCamera`][TiledParallaxCamera] on the camera |
| Add physics colliders | [`TiledPhysicsPlugin`][TiledPhysicsPlugin] and [`TiledPhysicsSettings`][TiledPhysicsSettings] (see [the guide](../guides/physics.md)) |
| Split tiles colliders into chunks | [`TiledColliderChunkSize`][TiledColliderChunkSize] |
| React when colliders are created | An observer on [`TiledEvent<ColliderCreated>`][ColliderCreated] |
| Use another physics engine | [`TiledPhysicsBackend`][TiledPhysicsBackend] and [`physics::geometry::collider_shapes`][collider_shapes] |
| Run my systems before or after `bevy_ecs_tiled` ones | [System sets][sets] |
| See what is going on | [`TiledDebugPluginGroup`][TiledDebugPluginGroup] (see [the guide](../guides/debug.md)) |

[TiledPlugin]: https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/struct.TiledPlugin.html
[TiledMap]: https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/ecs/map/struct.TiledMap.html
[TiledWorld]: https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/ecs/world/struct.TiledWorld.html
[TiledWorldStreaming]: https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/ecs/world/streaming/enum.TiledWorldStreaming.html
[RespawnTiledMap]: https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/ecs/map/struct.RespawnTiledMap.html
[TiledMapLayerZOffset]: https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/ecs/map/struct.TiledMapLayerZOffset.html
[TiledEvent]: https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/ecs/event/struct.TiledEvent.html
[TiledMapReference]: https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/ecs/map/struct.TiledMapReference.html
[TiledMapHelper]: https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/ecs/map/struct.TiledMapHelper.html
[TiledMapStorage]: https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/ecs/map/storage/struct.TiledMapStorage.html
[TiledTileLayerStorage]: https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/ecs/layer/struct.TiledTileLayerStorage.html
[TiledTile]: https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/ecs/tile/struct.TiledTile.html
[TiledObject]: https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/ecs/object/enum.TiledObject.html
[TiledMapAsset]: https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/asset/map/struct.TiledMapAsset.html
[TiledProperties]: https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/properties/struct.TiledProperties.html
[TiledTilemap]: https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/render/tilemap/struct.TiledTilemap.html
[TiledParallaxCamera]: https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/ecs/layer/struct.TiledParallaxCamera.html
[TiledPhysicsPlugin]: https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/physics/struct.TiledPhysicsPlugin.html
[TiledPhysicsSettings]: https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/physics/settings/struct.TiledPhysicsSettings.html
[TiledColliderChunkSize]: https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/physics/collider/enum.TiledColliderChunkSize.html
[ColliderCreated]: https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/physics/collider/struct.ColliderCreated.html
[TiledPhysicsBackend]: https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/physics/backend/trait.TiledPhysicsBackend.html
[collider_shapes]: https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/physics/geometry/fn.collider_shapes.html
[sets]: https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/sets/index.html
[TiledDebugPluginGroup]: https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/debug/struct.TiledDebugPluginGroup.html
