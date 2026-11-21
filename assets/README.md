# Assets description

See [main page](../README.md#assets-credits) for assets credits.

## Demos

| File | Tileset(s) | Asset(s) | Map type | Content | Used by |
| --- | --- | --- | --- | --- | --- |
| `demo_platformer/demo.tmx` | `demo_platformer/tilesets/ground.tsx`, `demo_platformer/tilesets/tiles.tsx` | Spritesheets in `demo_platformer/kenney_platformer-pack-redux/Spritesheets/`, background in `demo_platformer/kenney_platformer-pack-redux/PNG/Backgrounds/` | Infinite orthogonal | Objects, parallax image layer | `demo_platformer` example |
| `demo_topdown/outdoor.tmx` | `demo_topdown/tilesets/sokoban.tsx` | Tileset image in `demo_topdown/kenney_sokoban-pack/Tilesheet/` | Finite orthogonal | Objects | `demo_topdown` example |
| `demo_topdown/house.tmx` | `demo_topdown/tilesets/sokoban.tsx` | Tileset image in `demo_topdown/kenney_sokoban-pack/Tilesheet/` | Finite orthogonal | Objects | `demo_topdown` example (loaded through the `target_map` property of an object in `outdoor.tmx`) |

Other files:

- `demo_topdown/highlight.wesl`: shader of the custom tilemap material used by the `demo_topdown` example.
- `demo_platformer/demo_platformer.tiled-project` and `demo_topdown/demo_topdown.tiled-project`: Tiled projects to edit the demos maps.

## Maps list

Unless stated otherwise, maps with an image layer use `bevy_icon.png`.

| Map file | Tileset(s) | Asset(s) | Map type | Content | Used by |
| --- | --- | --- | --- | --- | --- |
| `maps/hexagonal/finite_flat_top_even.tmx` | `tiles/simple hex flat top.tsx` | Images collection in `tiles/simple hex flat top/` | Finite hexagonal flat top (even index) | Objects, image layer | `orientation_hexagonal`, `physics_orientation` examples, tests |
| `maps/hexagonal/finite_flat_top_odd.tmx` | `tiles/simple hex flat top.tsx` | Images collection in `tiles/simple hex flat top/` | Finite hexagonal flat top (odd index) | Objects, image layer | `orientation_hexagonal`, `physics_orientation` examples, tests |
| `maps/hexagonal/finite_pointy_top_even.tmx` | `tiles/drjamgo_hex_16x16.tsx` | Tileset image in `tiles/drjamgo_hex_16x16.png` | Finite hexagonal pointy top (even index) | Objects, image layer | `orientation_hexagonal`, `physics_orientation` examples, tests |
| `maps/hexagonal/finite_pointy_top_odd.tmx` | `tiles/drjamgo_hex_16x16.tsx` | Tileset image in `tiles/drjamgo_hex_16x16.png` | Finite hexagonal pointy top (odd index) | Objects, image layer | `orientation_hexagonal`, `physics_orientation`, `properties_basic` examples, tests |
| `maps/hexagonal/infinite_flat_top_even.tmx` | `tiles/simple hex flat top.tsx` | Images collection in `tiles/simple hex flat top/` | Infinite hexagonal flat top (even index) | Objects, image layer | `orientation_hexagonal`, `physics_orientation` examples |
| `maps/hexagonal/infinite_flat_top_odd.tmx` | `tiles/simple hex flat top.tsx` | Images collection in `tiles/simple hex flat top/` | Infinite hexagonal flat top (odd index) | Objects, image layer | `orientation_hexagonal`, `physics_orientation` examples |
| `maps/hexagonal/infinite_pointy_top_even.tmx` | `tiles/drjamgo_hex_16x16.tsx` | Tileset image in `tiles/drjamgo_hex_16x16.png` | Infinite hexagonal pointy top (even index) | Objects, image layer | `orientation_hexagonal`, `physics_orientation` examples |
| `maps/hexagonal/infinite_pointy_top_odd.tmx` | `tiles/drjamgo_hex_16x16.tsx` | Tileset image in `tiles/drjamgo_hex_16x16.png` | Infinite hexagonal pointy top (odd index) | Objects, image layer | `orientation_hexagonal`, `physics_orientation` examples |
| `maps/isometric/finite_diamond.tmx` | `tiles/kenney-sketch-desert.tsx` | Images collection in `tiles/kenney-sketch-desert/` | Finite diamond isometric | Objects, image layer | `orientation_isometric`, `physics_orientation` examples |
| `maps/isometric/infinite_diamond.tmx` | `tiles/kenney-sketch-desert.tsx` | Images collection in `tiles/kenney-sketch-desert/` | Infinite diamond isometric | Objects, image layer | `orientation_isometric`, `physics_orientation` examples |
| `maps/orthogonal/finite.tmx` | `tiles/orthogonal_1.tsx` | Images collection in `tiles/orthogonal/` | Finite orthogonal | Objects, image layer | Most `map_*` and `physics_*` examples, `orientation_orthogonal` example, tests |
| `maps/orthogonal/finite_embedded.tmx` | N/A (embedded in map) | Images collection in `tiles/orthogonal/` | Finite orthogonal | Tiles only | `orientation_orthogonal` example |
| `maps/orthogonal/finite_parallax.tmx` | `tiles/orthogonal_1.tsx` | Images collection in `tiles/orthogonal/` | Finite orthogonal | Objects, image layer, parallax layers | `map_parallax` example |
| `maps/orthogonal/group_layers.tmx` | `tiles/orthogonal_1.tsx` | Images collection in `tiles/orthogonal/` | Finite orthogonal | Objects, image layer, group layers | `map_group_layers` example |
| `maps/orthogonal/infinite.tmx` | `tiles/orthogonal_1.tsx` | Images collection in `tiles/orthogonal/` | Infinite orthogonal | Objects, image layer | `map_reload`, `orientation_orthogonal`, `physics_orientation`, `physics_settings` examples |
| `maps/orthogonal/infinite_embedded.tmx` | N/A (embedded in map) | Images collection in `tiles/orthogonal/` | Infinite orthogonal | Tiles only | `orientation_orthogonal` example |
| `maps/orthogonal/multiple_layers_with_colliders.tmx` | `tiles/orthogonal_1.tsx` | Images collection in `tiles/orthogonal/` | Finite orthogonal | Objects, several tiles layers with colliders | `physics_avian_controller`, `physics_rapier_controller` examples |
| `maps/orthogonal/multiple_tilesets.tmx` | `tiles/orthogonal_1.tsx`, `tiles/orthogonal_2.tsx` | Images collection in `tiles/orthogonal/` | Finite orthogonal | Objects | `orientation_orthogonal` example |
| `maps/orthogonal/text.tmx` | `tiles/orthogonal_1.tsx` | Images collection in `tiles/orthogonal/` | Finite orthogonal | Text objects | `map_text` example, tests |

For hexagonal maps:

- flat top = "stagger axis: X" in Tiled
- pointy top = "stagger axis: Y" in Tiled

For isometric maps: "staggered" maps are not supported.

## Worlds list

| World file | Maps | Tileset(s) | Asset(s) | World type | Used by |
| --- | --- | --- | --- | --- | --- |
| `worlds/orthogonal.world` | All 25 `.tmx` files in `worlds/orthogonal/` | `tiles/orthogonal_1.tsx` | Images collection in `tiles/orthogonal/` | Orthogonal | `world_basic`, `world_orientation`, `world_streaming` examples, tests |
| `worlds/hexagonal_flat_top.world` | All 4 `.tmx` files in `worlds/hexagonal_flat_top/` | `tiles/simple hex flat top.tsx` | Images collection in `tiles/simple hex flat top/` | Hexagonal flat top (even index) | `world_orientation` example, tests |
| `worlds/hexagonal_pointy_top.world` | All 4 `.tmx` files in `worlds/hexagonal_pointy_top/` | `tiles/drjamgo_hex_16x16.tsx` | Tileset image in `tiles/drjamgo_hex_16x16.png` | Hexagonal pointy top (odd index) | `world_orientation` example, tests |
| `worlds/isometric.world` | All 4 `.tmx` files in `worlds/isometric/` | `tiles/kenney-sketch-desert.tsx` | Images collection in `tiles/kenney-sketch-desert/` | Diamond isometric | `world_orientation` example, tests |

## Tilesets list

| Tileset file | Asset(s) | Tileset type |
| --- | --- | --- |
| `tiles/orthogonal_1.tsx` | `tiles/orthogonal/` | Images collection |
| `tiles/orthogonal_2.tsx` | `tiles/orthogonal/` | Images collection |
| `tiles/simple hex flat top.tsx` | `tiles/simple hex flat top/` | Images collection |
| `tiles/kenney-sketch-desert.tsx` | `tiles/kenney-sketch-desert/` | Images collection |
| `tiles/drjamgo_hex_16x16.tsx` | `tiles/drjamgo_hex_16x16.png` | Single image |
| `demo_platformer/tilesets/ground.tsx` | `demo_platformer/kenney_platformer-pack-redux/Spritesheets/spritesheet_ground.png` | Single image |
| `demo_platformer/tilesets/tiles.tsx` | `demo_platformer/kenney_platformer-pack-redux/Spritesheets/spritesheet_tiles.png` | Single image |
| `demo_topdown/tilesets/sokoban.tsx` | `demo_topdown/kenney_sokoban-pack/Tilesheet/sokoban_tilesheet.png` | Single image |

## Tiled projects

| Project file | Description |
| --- | --- |
| `projects/default.tiled-project` | Generic project to browse and edit the maps, worlds and tilesets of this folder |
| `projects/properties_basic.tiled-project` | Custom property types exported for the `properties_basic` example |

## Test assets

These maps are only used by unit tests. They use embedded tilesets (or no tileset at all) and are kept as small as possible.

| Folder | Content |
| --- | --- |
| `tests/chunking/` | An empty map and a world containing it twice, used by events and world streaming tests |
| `tests/dynamic/` | Orthogonal, hexagonal and isometric maps used to test adding, removing and moving tiles at runtime |
| `tests/hexagonal/` | Hexagonal maps with custom side length and odd height, used to check tiles positions against Tiled |
| `tests/loader/` | A staggered map, which the loader must reject |
| `tests/perf/` | A large (200x200) map used by the ignored performance test |
| `tests/properties/` | A map with an image (file) property, used to test properties loading |
