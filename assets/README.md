# Assets description

See [main page](../README.md) for assets credits.

## Maps list

| Map file | Tileset(s) | Asset(s) | Map type | Objects in map ? |
|----------|------------|----------|----------|------------------|
| `maps/hexagonal/finite_flat_top_even.tmx` | `tiles/simple hex flat top.tsx` | Images collection in `tiles/simple hex flat top/` | Finite hexagonal flat top (even index) | Yes |
| `maps/hexagonal/finite_flat_top_odd.tmx` | `tiles/simple hex flat top.tsx` | Images collection in `tiles/simple hex flat top/` | Finite hexagonal flat top (odd index) | Yes |
| `maps/hexagonal/finite_pointy_top_even.tmx` | `tiles/drjamgo_hex_16x16.tsx` | Tileset image in `tiles/drjamgo_hex_16x16.png` | Finite hexagonal pointy top (even index) | Yes |
| `maps/hexagonal/finite_pointy_top_odd.tmx` | `tiles/drjamgo_hex_16x16.tsx` | Tileset image in `tiles/drjamgo_hex_16x16.png` | Finite hexagonal pointy top (odd index) | Yes |
| `maps/hexagonal/infinite_flat_top_even.tmx` | `tiles/simple hex flat top.tsx` | Images collection in `tiles/simple hex flat top/` | Infinite hexagonal flat top (even index) | Yes |
| `maps/hexagonal/infinite_flat_top_odd.tmx` | `tiles/simple hex flat top.tsx` | Images collection in `tiles/simple hex flat top/` | Infinite hexagonal flat top (odd index) | Yes |
| `maps/hexagonal/infinite_pointy_top_even.tmx` | `tiles/drjamgo_hex_16x16.tsx` | Tileset image in `tiles/drjamgo_hex_16x16.png` | Infinite hexagonal pointy top (even index) | Yes |
| `maps/hexagonal/infinite_pointy_top_odd.tmx` | `tiles/drjamgo_hex_16x16.tsx` | Tileset image in `tiles/drjamgo_hex_16x16.png` | Infinite hexagonal pointy top (odd index) | Yes |
| `maps/isometric/finite_diamond.tmx` | `tiles/kenney-sketch-desert.tsx` | Images collection in `tiles/kenney-sketch-desert/` | Finite diamond isometric | Yes |
| `maps/isometric/infinite_diamond.tmx` | `tiles/kenney-sketch-desert.tsx` | Images collection in `tiles/kenney-sketch-desert/` | Infinite diamond isometric | Yes |
| `maps/orthogonal/finite_embedded.tmx` | N/A (embedded in map) | Images collection in `tiles/orthogonal/` | Finite orthogonal | No |
| `maps/orthogonal/finite.tmx` | `tiles/orthogonal_1.tsx` | Images collection in `tiles/orthogonal/` | Finite orthogonal | No |
| `maps/orthogonal/infinite_embedded.tmx` | N/A (embedded in map) | Images collection in `tiles/orthogonal/` | Infinite orthogonal | No |
| `maps/orthogonal/infinite.tmx` | `tiles/orthogonal_1.tsx` | Images collection in `tiles/orthogonal/` | Infinite orthogonal | No |
| `maps/orthogonal/multiple_layers_with_colliders.tmx` | `tiles/orthogonal_1.tsx` | Images collection in `tiles/orthogonal/` | Finite orthogonal | Yes |
| `maps/orthogonal/multiple_tilesets.tmx` | `tiles/orthogonal_1.tsx` | Images collection in `tiles/orthogonal/` | Finite orthogonal | Yes |
| `maps/demo.tmx` | All `.tsx` files in `Magic Market/Tilesets/` | Tileset images in `Magic Market/Art/` | Finite orthogonal | Yes |

For hexagonal maps :

- flat top = "stagger axis: X" in Tiled
- pointy top = "stagger axis: Y" in Tiled

For isometric maps: "stagerred" maps are not supported.

## Worlds list

| World file | Maps | Tileset(s) | Asset(s) | World type |
|------------|------|------------|----------|------------|
| `worlds/basic.world` | All `.tmx` files in `worlds/basic/` | `tiles/orthogonal_1.tsx` | Images collection in `tiles/orthogonal/` | Orthogonal |
