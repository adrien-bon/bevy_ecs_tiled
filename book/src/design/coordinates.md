# Coordinates conversion between Tiled and Bevy

Tiled and Bevy do not describe positions the same way.
`bevy_ecs_tiled` converts everything for you when it spawns a map, but you will need to understand the differences as soon as you want to:

- find which tile is under the cursor,
- place a gameplay entity where an object was drawn in Tiled,
- read raw data from [`tiled::Map`](https://docs.rs/tiled/latest/tiled/struct.Map.html) (the `map` field of [`TiledMapAsset`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/tiled/map/asset/struct.TiledMapAsset.html)) and use it in Bevy.

This page explains the two coordinate systems and how `bevy_ecs_tiled` converts from one to the other.

---

## Two different conventions

| | Tiled | Bevy |
|---|---|---|
| Unit | pixels | world units (1 unit = 1 pixel with a default 2D camera) |
| Origin | top-left corner of the map | the map entity [`Transform`](https://docs.rs/bevy/latest/bevy/transform/components/struct.Transform.html), adjusted by the [`TilemapAnchor`](https://docs.rs/bevy_ecs_tilemap/latest/bevy_ecs_tilemap/anchor/enum.TilemapAnchor.html) |
| X axis | points right | points right |
| Y axis | points **down** | points **up** |
| Tile indexes | `(0, 0)` is the **top-left** tile | `TilePos { x: 0, y: 0 }` is the **bottom-left** tile |
| Rotation | degrees, **clockwise** | radians, **counter-clockwise** |
| Layer offset | `(offset_x, offset_y)`, Y down | `(offset_x, -offset_y)`, Y up |

```text
        Tiled                                   Bevy

  (0,0) ──────────► +X                    +Y ▲
    │  ┌───┬───┬───┐                         │  ┌───┬───┬───┐
    │  │0,0│1,0│2,0│                         │  │0,2│1,2│2,2│
    │  ├───┼───┼───┤                         │  ├───┼───┼───┤
    │  │0,1│1,1│2,1│                         │  │0,1│1,1│2,1│
    │  ├───┼───┼───┤                         │  ├───┼───┼───┤
    │  │0,2│1,2│2,2│                         │  │0,0│1,0│2,0│
    ▼  └───┴───┴───┘                         │  └───┴───┴───┘
   +Y                                        └──────────────► +X
```

The X axis is the same in both systems.
Almost every conversion therefore comes down to **flipping the Y axis** and **moving the origin**.

---

## Converting a Tiled position

Every Tiled pixel position (objects, image layers, map corners, ...) is converted by [`TiledMapAsset::world_space_from_tiled_position`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/tiled/map/asset/struct.TiledMapAsset.html#method.world_space_from_tiled_position).
It returns a position **relative to the map entity**.

---

## Infinite maps

In an infinite map, Tiled stores tiles in chunks of 16x16 tiles, and chunk indexes can be negative.
`bevy_ecs_tilemap` does not support negative tile positions, so all chunks are shifted so that the **top-left chunk becomes chunk `(0, 0)`**:

```text
          Tiled chunks                         TilePos origin

   (-1,-1) │ (0,-1) │ (1,-1)            ┌────────┬────────┬────────┐
   ────────┼────────┼────────           │        │        │        │
   (-1, 0) │ (0, 0) │ (1, 0)    ──►     ├────────┼────────┼────────┤
   ────────┼────────┼────────           │        │        │        │
                                        └────────┴────────┴────────┘
                                       (0,0) ▲ bottom-left tile
```

The map size used by Bevy ([`TiledMapAsset::tilemap_size`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/tiled/map/asset/struct.TiledMapAsset.html#structfield.tilemap_size)) is the bounding box of all the chunks, not the `width` / `height` stored in the `.tmx` file.
Tiled pixel positions are shifted by the same amount, so objects stay aligned with tiles.

---

## Isometric maps

On isometric (diamond) maps, Tiled object coordinates are **not** screen pixels.
They are expressed along the two diagonal axes of the grid, in pixels where **one tile measures `tile_height` on both axes**:

```text
                     ◆  ← Tiled (0, 0)
                   ╱   ╲
       Tiled +Y  ╱       ╲  Tiled +X
               ╱           ╲
             ◆      map      ◆
               ╲           ╱
                 ╲       ╱
                   ╲   ╱
                     ◆
```

To get a screen position, `bevy_ecs_tiled` first projects these coordinates:

```text
fx = tiled.x / tile_height
fy = tiled.y / tile_height
screen.x = (fx - fy) * tile_width  / 2 + map_height_in_tiles * tile_width / 2
screen.y = (fx + fy) * tile_height / 2
```

and then flips the Y axis, as for other orientations.

Since `bevy_ecs_tilemap` and Tiled do not place isometric tile images the same way, the tilemap entity also receives a small offset so that tiles are rendered exactly where they are in Tiled.

Image layers are positioned relative to Tiled "screen origin", which, for isometric maps, is located `map_height * tile_width / 2` pixels on the left of the top corner of tile `(0, 0)`.

> **Note:**
> Staggered isometric maps are not supported and are rejected by the map loader.

---

## Hexagonal maps

Hexagonal maps use the same Y flip as orthogonal maps.
However, flipping the Y axis changes which rows (or columns) are "staggered", so the Tiled stagger settings map to the `bevy_ecs_tilemap` [`HexCoordSystem`](https://docs.rs/bevy_ecs_tilemap/latest/bevy_ecs_tilemap/map/enum.HexCoordSystem.html) as follows:

| Tiled stagger axis | Tiled stagger index | `HexCoordSystem` |
|---|---|---|
| X | Even | `ColumnOdd` |
| X | Odd | `ColumnEven` |
| Y | Even | `RowOdd` |
| Y | Odd | `RowEven` |

This is handled by [`tilemap_type_from_map`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/tiled/helpers/fn.tilemap_type_from_map.html), which you can use if you need the `TilemapType` of a map.

---

## Worlds

In a `.world` file, each map position is the Tiled pixel position of its **top-left** corner, with Y pointing down.
`bevy_ecs_tiled` flips it so that the world bounding box ([`TiledWorldAsset::rect`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/tiled/world/asset/struct.TiledWorldAsset.html#structfield.rect)) starts at `(0, 0)` and grows towards positive X and Y.

The `TilemapAnchor` of a [`TiledWorld`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/tiled/world/struct.TiledWorld.html) applies to the whole world bounding box, not to each map individually.
