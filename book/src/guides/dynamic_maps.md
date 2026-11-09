# Modify a Map at Runtime

Once a map has been loaded, you can freely modify it: spawn, despawn or modify its tiles, objects and images.
`bevy_ecs_tiled` takes care of keeping their rendering and their physics colliders up to date.

This is useful to build destructible terrain, open doors, let the player build things, generate parts of a level procedurally, and more.

A [dedicated example](https://github.com/adrien-bon/bevy_ecs_tiled/blob/main/examples/map_dynamic.rs) demonstrates these features.

---

## How It Works

The entities spawned when loading a map fall into two categories:

- **Tiled data**: the map, its layers, tiles, objects and images entities.
  They hold what was defined in Tiled: which tile is at which position, the shape of an object, the image of an image layer, ...
- **Derived entities and components**: tilemaps, objects visuals (sprites and texts), images sprites and physics colliders.
  They are computed from Tiled data and you should not modify them yourself, since your changes would be overwritten.

Tiled data are the source of truth: when they change, derived entities are updated during `PostUpdate`, in the following [`TiledPostUpdateSystems`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/tiled/sets/enum.TiledPostUpdateSystems.html) sets:

| System set | What is updated |
| --- | --- |
| `First` | A [`TiledMapReference`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/tiled/map/struct.TiledMapReference.html) is inserted on new tiles and objects which do not have one. |
| `SyncMapStorage` | Tiles are registered in the map [`TiledMapStorage`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/tiled/map/storage/struct.TiledMapStorage.html), and a tile moved onto another one replaces it. |
| `SyncRender` | Tilemaps, objects visuals and images sprites are spawned, updated or despawned. |
| `SyncPhysics` | Physics colliders are spawned again. |

These sets run before Bevy transform and visibility propagation: if you modify Tiled data during `Update` (or earlier), the result is visible in the same frame.

See the [entities hierarchy page](../design/entities_hierarchy.md) for a description of all the entities composing a map.

---

## Tiles

Each tile is an entity with a [`TiledTile`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/tiled/tile/struct.TiledTile.html) component, child of its [`TiledLayer::Tiles`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/tiled/layer/enum.TiledLayer.html#variant.Tiles) entity.
Its Tiled data are held by the following components, which you can freely modify:

| Component | Description |
| --- | --- |
| [`TiledTile`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/tiled/tile/struct.TiledTile.html) | Tileset of the tile (`tileset_id`, a `TilesetId`) and tile in this tileset (`tile_id`) |
| `TilePos` | Position of the tile in the layer (origin is the bottom-left corner of the map) |
| `TileFlip` | Horizontal, vertical and diagonal flips |
| `TileVisible`, `TileColor` | Visibility and color of the tile |

Other components, such as `TilemapId`, `TileTextureIndex` or `AnimatedTile`, are derived from these data: do not modify them.

### Modify, Move or Despawn a Tile

```rust,no_run
use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;

/// Replace tile 0 with tile 1 of the same tileset, everywhere
fn replace_tiles(mut tiles: Query<&mut TiledTile>) {
    for mut tile in tiles.iter_mut() {
        if tile.tile_id == 0 {
            tile.tile_id = 1;
        }
    }
}

/// Move all the tiles of a layer one step to the right
fn move_tiles(mut tiles: Query<&mut TilePos, With<TiledTile>>) {
    for mut tile_pos in tiles.iter_mut() {
        tile_pos.x += 1;
    }
}

/// Despawn the tile at a given position of a given layer
fn despawn_tile(
    mut commands: Commands,
    maps: Query<&TiledMapStorage>,
    tiles: Query<&ChildOf, With<TiledTile>>,
) {
    let (layer, position) = (Entity::PLACEHOLDER, TilePos::new(3, 4));
    for storage in maps.iter() {
        // Tiles of all the layers of the map at this position
        for &entity in storage.tiles_at(position) {
            if tiles.get(entity).is_ok_and(|child_of| child_of.parent() == layer) {
                commands.entity(entity).despawn();
            }
        }
    }
}
```

When a tile changes, it is moved to the tilemap of its new tileset, and the colliders of its layer are spawned again.
A tilemap is spawned when a tile using a new tileset is added to a layer, and despawned once it does not contain any tile anymore.

### Spawn a Tile

To spawn a new tile, you only need to provide its Tiled data and its parent layer:

```rust,no_run
use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;

fn spawn_tile(mut commands: Commands, layers: Query<(Entity, &TiledLayer)>) {
    let Some((layer, _)) = layers.iter().find(|(_, l)| matches!(l, TiledLayer::Tiles)) else {
        return;
    };
    commands.spawn((
        // Tile 2 of the first tileset of the map
        TiledTile::new(TilesetId(0), 2),
        ChildOf(layer),
        TilePos::new(3, 4),
    ));
}
```

Other components (`TileFlip`, `TileVisible`, `TileColor`, ...) get a default value, and the `TiledMapReference` is inserted from the parent layer.
If the tile has [custom properties](properties.md) in its tileset, they are also inserted on the new entity.

### Rules and Limitations

- **A position holds a single tile**, as in Tiled: spawning a tile, or moving a tile, onto a position already used by another tile of the same layer despawns this other tile, whatever its tileset.
- **Layers cannot grow**: tiles outside of the map bounds (see [`TiledMapAsset::tilemap_size`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/tiled/map/asset/struct.TiledMapAsset.html#structfield.tilemap_size)) are neither rendered nor used to compute physics colliders, and a warning is logged.
- **Tiles cannot change of layer**: changing the `ChildOf` of a tile is not supported. Despawn it and spawn a new one in the other layer instead.
- **Custom properties are not updated** when you modify the `TiledTile` component of an existing tile.
- A tileset which cannot be used in a tiles layer (for instance an images collection with images of different sizes) cannot be used for spawned tiles either.

---

## Objects

Each object is an entity with a [`TiledObject`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/tiled/object/enum.TiledObject.html) component, describing its shape, child of its [`TiledLayer::Objects`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/tiled/layer/enum.TiledLayer.html#variant.Objects) entity.
Its position is given by its `Transform`, relative to its layer.

When the `TiledObject` component changes, the object visual (for tile and text objects) and its physics colliders are spawned again.
When it is removed or when the object is despawned, they are despawned as well.

```rust,no_run
use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;

/// Turn all ellipses into rectangles
fn change_shapes(mut objects: Query<&mut TiledObject>) {
    for mut object in objects.iter_mut() {
        if let TiledObject::Ellipse { width, height } = *object {
            *object = TiledObject::Rectangle { width, height };
        }
    }
}

/// Spawn a new rectangle object in the given objects layer
fn spawn_object(mut commands: Commands) {
    let layer = Entity::PLACEHOLDER;
    commands.spawn((
        TiledObject::Rectangle {
            width: 32.,
            height: 16.,
        },
        TiledName("door".to_string()),
        Transform::from_xyz(64., -32., 0.),
        ChildOf(layer),
    ));
}
```

When spawning an object:

- give it a [`TiledName`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/tiled/filter/struct.TiledName.html) if you use [physics settings](physics.md) filtering objects by name,
- as for tiles, its `TiledMapReference` is inserted from the parent layer, and a [`TiledObject::Tile`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/tiled/object/enum.TiledObject.html#variant.Tile) gets the custom properties of its tile.

Moving an object (ie. modifying its `Transform`) does not need any update: its visual and colliders are its children and move along with it.

---

## Images

An image layer has a single child holding a [`TiledImage`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/tiled/image/struct.TiledImage.html) component: the image handle, its position, size and repeat settings.
Its `Sprite` is derived from it: to change the image, modify the `TiledImage` component instead of the `Sprite`.

Without the `render` feature, the image is not loaded: `TiledImage` has no image handle and there is no `Sprite`.

---

## Reacting to Changes

[Map loading events](../design/map_events.md) (`MapCreated`, `LayerCreated`, `TileCreated`, `ObjectCreated`) are only sent when a map is loaded, not when you modify it afterwards.
Two events are however sent when derived entities are spawned:

- `TiledEvent<TilemapCreated>`, each time a tilemap is spawned: when the map is loaded, but also when a tile using a new tileset is added to a layer.
- `TiledEvent<ColliderCreated>`, each time a collider is spawned: since colliders are spawned again when their source changes, you receive this event every time a tiles layer or an object is modified.

For instance, if you add a `RigidBody` to colliders when receiving `ColliderCreated` (see the [physics guide](physics.md)), new colliders will get it as well.

The map [`TiledMapStorage`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/tiled/map/storage/struct.TiledMapStorage.html) is also kept up to date, so you can keep using it to list the layers and objects of the map, to find the entities of a given tile (`tile_entities()`), or the tiles at a given position (`tiles_at()`, whatever their layer).
Layers and objects spawned at runtime are listed after the ones spawned along with the map.

---

## Respawning a Map Discards Modifications

All the modifications you made are lost when the map is respawned from its asset, which happens when:

- you insert the [`RespawnTiledMap`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/tiled/map/struct.RespawnTiledMap.html) component,
- the map is hot-reloaded,
- you modify the `TiledMap`, `TilemapAnchor`, `TiledMapLayerZOffset`, `TilemapRenderSettings` or `TiledPhysicsSettings` components of the map entity.

See [Spawn / Despawn / Reload a Map](spawn_reload.md) for more information.

---

## Performance

- Modifying tiles only updates the tiles which changed, and the tilemaps they belong to.
- However, **physics colliders of a tiles layer are computed again as a whole** whenever one of its tiles changes: on a layer with many tiles, it can take a significant amount of time.
  If you frequently modify a large tiles layer, consider keeping colliders out of it (see the [physics settings](physics.md) to filter layers), or splitting it into several layers.
- Modifications are processed once per frame: modifying many tiles in the same frame costs much less than modifying them over several frames.
