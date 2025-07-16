# Changelog

## [unreleased]

**BREAKING CHANGES**
This version brings breaking changes.
A [migration guide](https://adrien-bon.github.io/bevy_ecs_tiled/migrations/v0_8.html) is available to help adapt to these.

### Features

- Explicit `SystemSet` to allow systems ordering for user applications.
- Rationalize which types we re-export (#88).
- Attach shape information to `TiledObject` (#77).
- Automatically adjust translation for objects on the same layer to avoid Z-fighting (#81).

### Changed

- Several types or functions have been renamed to better reflect what they actually do.
- Rework the `Event` APi and provide an unified way to handle events and access underlying Tiled data.
- Add helpers functions to the `TiledMapStorage` component so it's easier to work with.
- Several files have been moved around or renamed.

### Bugfixes

- Rotation is not taken into account for tile objects (#76).
- Tile objects position for isometric maps (#105).

### Documentation

- Overall improvement of the documentation.
- Add some example to enable `physics` and `user_properties` features (#100).
- Add a FAQ entry about `.world` loading (#82).

## v0.7.3

### Features

- Add support for object templates files (.tx) (#98)

## v0.7.2

### Bugfixes

- Properly spawn all maps from a world when not using chunking, we were missing the last one (#96)

## v0.7.1

### Features

- Add a new `TiledColliderCreated` event and an associated observer, when a new collider is spawned.

### Bugfixes

- Do not restrict `world_chunking` system to camera with `Changed<Transform>` (#94)

## v0.7.0

**BREAKING CHANGES**
This version brings breaking changes.
A [migration guide](https://adrien-bon.github.io/bevy_ecs_tiled/migrations/v0_7.html) is available to help adapt to these.

### Features

- Update for Bevy v0.16
- Add support for `bevy_ecs_tilemap::TilemapAnchor`
- Make `export_types()`and `export_types_filtered()` public
- Add `Visibility::Hidden` to objects marked not visible in Tiled

## v0.6.0

### Features

- Add Tiled .world file support (#55)
- Aggregate tiles colliders together: likely to reduce the overall number of colliders which improves performances (#68)
- Most of `bevy_ecs_tiled` Components and Resources are now registered in Bevy type registry
- Properly spawn colliders for `TileOjbect`
- Add a new debug plugin for tiles position
- Create a `PluginGroup` to centralize all debug plugins
- Add a new debug plugin to visualize world chunking and map AABB
- Properly handle world chunking with a non null rotation
- Order plugin systems. Instead of doing everything in `Update`, schedule systems that spawn stuff in `PreUpdate` and systems that despawn stuff in `PostUpdate`
- Properly use the `Visible` flag for objects
- Properly react to components change: automatically respawn maps / worlds in case of update
- Automatically register `bevy_ecs_tilemap::TilemapPlugin` plugin when registering `TiledMapPlugin`

### Changed

- `TiledPhysicsBackend` now requires to implement the `Clone`, `Reflect` and `Debug` traits
- Switched some map logs from `info!()` to `debug!()`
- `TiledPhysicsBackend::spawn_collider()` is now expected to spawn several colliders in on call. To reflect that, now it returns a `Vec<TiledColliderSpawnInfos>` instead of an `Option<TiledColliderSpawnInfos>`
- Remove the `TiledColliderSourceType::Tile` which is superseded by `TiledColliderSourceType::TilesLayer`
- Update Map, World and Physics events to use safe methods for getting their inner data
- Update Map and physics events to use `AsssetId` instead of a direct `Handle`
- Update the `TiledMapLayer` marker component to not contains a reference to the map `AssetId`, we should instead query the map directly
- Prevent mutating our Map and World asset for nothing: it was triggering an additionnal asset reload
- Clear our tileset cache when we receive an `AssetEvent::Modified` (ie. the asset is reloaded from disk)
- Rename `TiledIdStorage` component to `TiledMapStorage` to be consistent with the new world API
- Rename `ObjectNameFilter` to `TiledNameFilter` since we also use it for layer name
- Rework physics backend: remove collider spawn events, rename types so they are easier to use
- Update `TiledPhysicsBackend::spawn_colliders` signature so it can now take an object name filter
- Change the way we retrieve `PhysicsSettings` so we don't require to get the map event first
- Replace global observers with an entity scoped observer + a global event writer
- Rework how we store tileset data in `TiledMap`
- Add new examples to test 'infinite' maps and rework how we organize our assets
- Rename `LayerPositioning::TiledOffset` to `LayerPositioning::BottomLeft`
- Split `TiledMapSettings` in `TiledMapAnchor` + `TiledMapLayerZOffset` and rename `TiledWorldSettings` into `TiledWorldChunking`
- Store all tiles in `TiledMapStorage`, not just ones which have user properties
- Renamed `from_tiled_coords_to_bevy` to `from_tiled_position_to_world_space`

### Bugfixes

- Various fixes around infinite maps: we should now have proper location for objects and tile layer chunks
- Fix `TilemapRenderSettings::chunk_size` value for isometric examples (#73)
- Properly load unit-variant enums as components without needing to encapsulate them in another component (#75)

### Documentation

- Rework documentation and examples related to map events (#74)

## v0.5.1

### Features

- Take into account the layer visibility information from Tiled and set Bevy Visibility component accordingly
- Add the 'Magic Market' tileset and example to showcase a more realistic / complex map
- Handle objects with embedded TileData: we will spawn a (potentially animated) Sprite using the corresponding tile image

### Bugfixes

- Fixed debug and info logging in `process_loaded_maps` to handle cases where a map handle's path is unavailable, using the handle ID as a fallback.
- Properly abort map loading and despawn map entity in case of asset load error
- Layers Z-offset was not correct in case of non-default value for 'layer_z_offset' setting
- Fix animated tiles animation speed
- Prevent bevy_ecs_tilemap panic when using a tileset with non constant image size

## v0.5.0

**BREAKING CHANGES**
This version brings breaking changes.
A [migration guide](https://adrien-bon.github.io/bevy_ecs_tiled/migrations/v0_5.html) is available to help adapt to these.

### Features

- Update to Bevy 0.15
- Support Tiled image layers (#16)
- Add support for non-unit enums user properties (#49)
- Better handling for user properties default values: you should now be able to load
  properties without having to fully define them

### Changed

- Remove `map_initial_transform` and `map_initial_visibility` from `TiledMapSettings`, use required components instead
- Simplify map reload mechanisms and update corresponding example
- Only export custom properties that are related to either a Component, a Bundle or a Resource (#50)
  This should reduce the clutter when adding a new property

### Bugfixes

- Do not skip last frame for animated tiles (#61)
- Make sure all custom properties we export have the 'useAs property' flag set (and only this one)
  Not having this flag was preventing to use some some of the properties
- Prevent infinite loop in case of an asset loading error
- Fix typo which was preventing to load the first object collider (#64)
- Fix convex polygon colliders (#65)

## v0.4.2

### Changed

- Removed the `dbg!` macro from the codebase and replaced it with `trace!` (#54)

### Documentation

- Added a note about the `logging` and `tracing` features in the debugging guide (#54)

## v0.4.1

### Changed

- Enable Bevy `file_watcher` feature when running examples to enable assets hot-reloading

### Documentation

- Add a note in the FAQ about assets hot-reloading

### Bugfixes

- Fix hot-reload (#52)

## v0.4.0

**BREAKING CHANGES**
This version brings breaking changes.
A [migration guide](https://adrien-bon.github.io/bevy_ecs_tiled/migrations/v0_4.html) is available to help adapt to these.

### Features

- Add map loading events to give user access to Tiled internal data structure.
- Add various settings to control where we spawn the map.

### Changed

- Rework how user properties are loaded from Tiled. Don't use macros anymore and instead rely upon `bevy_reflect`.
- Take advantage of map loading events and move everything related to physics in a dedicated plugin.
- Rework how we manage physics backends with the perspective to provide more advanced ones in the future.
- Do not insert a `Transform` on every tiles anymore, only do it for tiles with a collider which actually need it (which is likely to improve performances).
- Get ride of `TiledMapBundle` and add a dedicated component to hold the `Handle<TiledMap>`.
- Rename `MapPositioning` to `LayerPositioning`.

### Documentation

- Finalize API reference documentation: all public API are now documented.
- Add various guides and explanations to the book.

### Bugfixes

- Fix tiles colliders position for hexagonal maps (#40)

## v0.3.11

### Bugfixes

- Fix crash when the top-left tile of an infinite map is used (#46)

### Documentation

- Add debug guide
- Add map spawn/despawn/reload guide

## v0.3.10

### Documentation

- Initialize `bevy_ecs_tiled` book (#15)
- Update code documentation

## v0.3.9

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
