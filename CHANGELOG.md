# Changelog

## [unreleased]

### Bugfixes

- Fix compilation issue when not using `physics` feature

## v0.3.8

### Features

- Add the possibility for the user to use a custom physics backend and add an associated example (#30)
- Add a note about isometric (staggered) maps which are not supported and add a dedicated example
- Add a bit of documentation
- Update layers Z-offset so the upper layer will be at offset 0 and layers behind will have negative offset values

### Bugfixes

- Fix objects position for isometric (diamond) maps and update associated example

## v0.3.7

### Features

- Add dedicated marker for Tiled objects (#22), object layers, group layers and image layers
- Regroup tilemaps from the same tile layer but using different tilesets under a common parent entity `TiledMapTileLayerForTileset`
- Add a new `TiledMapDebugPlugin` to draw a simple `arrow_2d` gizmo where Tiled objects are spawned (#18)
- Add a `README.md` file to describe Tiled maps used in examples
- Add `simple hex flat top` asset so we can have some hexagonal 'flat-top' examples
- Rework the `hex_map` example to cycle through different kinds of hexagonal maps

### Bugfixes

- Prevent duplicating objects when there are multiple tilesets (#28)
- Do not rely upon `AssetEvent::added()` to actually spawn a map (#23)
  Instead, query all maps that have a `Changed<Handle<TiledMap>>` or are explictelly marked for reload.
  This allows to delay actual map spawn from asset loading (hence allowing to pre-load maps).
- Fix objects position for hexagonal maps: may not work in all cases, especially for 'infinite' maps

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
