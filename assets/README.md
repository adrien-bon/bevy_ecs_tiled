# Maps list

| Map file | Tileset(s) | Asset(s) | Map type | Objects in map ? |
|----------|------------|----------|----------|------------------|
| `colliders_and_user_properties.tmx` | `Tileset1.tsx` | Images collection in `tiles/` | Finite orthogonal | Yes |
| `finite_embedded.tmx` | N/A (embedded in map) | Images collection in `tiles/` | Finite orthogonal | No |
| `finite.tmx` | `Tileset1.tsx` | Images collection in `tiles/` | Finite orthogonal | No |
| `hex_map_flat_top_even.tmx` | `simple hex flat top.tsx` | Images collection in `tiles/simple hex flat top/` | Finite hexagonal flat top (even index) | Yes |
| `hex_map_flat_top_odd.tmx` | `simple hex flat top.tsx` | Images collection in `tiles/simple hex flat top/` | Finite hexagonal flat top (odd index) | Yes |
| `hex_map_pointy_top_even.tmx` | `drjamgo_hex_16x16.tsx` | Tileset image in `tiles/drjamgo_hex_16x16.png` | Finite hexagonal pointy top (even index) | Yes |
| `hex_map_pointy_top_odd.tmx` | `drjamgo_hex_16x16.tsx` | Tileset image in `tiles/drjamgo_hex_16x16.png` | Finite hexagonal pointy top (odd index) | Yes |
| `infinite_embedded.tmx` | N/A (embedded in map) | Images collection in `tiles/` | Infinite orthogonal | No |
| `finite.tmx` | `Tileset1.tsx` | Images collection in `tiles/` | Infinite orthogonal | No |
| `isometric_diamond_map.tmx` | `kenney-sketch-desert.tsx` | Images collection in `tiles/kenney-sketch-desert/` | Finite diamond isometric | Yes |
| `isometric_staggered_map.tmx` | `kenney-sketch-desert.tsx` | Images collection in `tiles/kenney-sketch-desert/` | Finite staggered isometric | Yes |
| `multiple_layers_with_colliders.tmx` | `Tileset1.tsx` | Images collection in `tiles/` | Finite orthogonal | Yes |
| `multiple_tilesets.tmx` | `Tileset1.tsx` + `Tileset2.tsx` | Images collection in `tiles/` | Finite orthogonal | Yes |

For hexagonal maps :

- flat top = "stagger axis: Y" in Tiled
- pointy top = "stagger axis: X" in Tiled

See [main page](../README.md) for assets credits.
