# Changelog

## [unreleased]

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
