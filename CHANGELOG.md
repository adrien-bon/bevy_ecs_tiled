# Changelog

## [unreleased]

### Features

- Add dedicated marker for Tiled objects (#22) and for objects layer, group layer and images layer
- Regroup tilemaps of the same tile layer using different tilesets under a common parent entity `TiledMapTileLayerForTileset`
- Add a new `TiledMapDebugPlugin` to draw a simple gizmo where Tiled objects are spawned (#18)

### Bugfixes

- Prevent duplicating objects when there are multiple tilesets (#28)
- Do not rely upon `AssetEvent::added()` to actually spawn a map (#23)
  Instead, query all maps that have a `Changed<Handle<TiledMap>>` or are explictelly marked for reload.
  This allows to delay actual map spawn from asset loading (hence allowing to pre-load maps).

## v0.3.6

### Bugfixes

- Remove the need to add an explicit dependency against 'tiled' crate when using the 'user_properties' feature flag (#21)

## v0.3.5

### Features

- Add support for Avian2D Colliders added from tilesets and object layers (`avian` feature flag) (#14 and #20)

## v0.3.4

### Features

- Add a new example with a simple player-controlled object using Rapier physics
- Add a new example with an isometric map
- Properly handle layers Z-order (see #4, #7 and #9)
- Provide a callback to attach additional components to tiles / objects colliders to improve collisions detection (#7 and #9)
- New feature flag: `user_properties`, map Tiled user properties on tiles and objects to Bevy components (#5 and #10)

### Bugfixes

- Fix bad positioning of objects collider when using MapPositioning::Centered (#11)

## v0.3.3

### Features

- Add support for animated tiles (#4) (note: the Z-layer fix have been reverted, see #7)
- Add `wasm` feature
- Add support for transforming TilemapBundle using (0, 0) as the center (#8)

## v0.3.1

### Features

- Properly handle hexagonal maps (#3)

### Bugfixes

- Do not duplicate path when loading a tilesheet (#2)

## v0.3.0

### Features

- Finite and infinite maps
- Embedded and separate tileset
- Layers are children of the tilemap entity. Tiles are children of layers.
- Visibility is inherited: map -> layer -> tile
- Spawn/despawn
- Rapier Colliders added from tilesets and object layers (`rapier` feature flag)
