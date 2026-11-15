// Tilemap fragment shader used by the `demo_topdown` example: it displays tiles
// as usual, then highlights the tile below the pointer and the player destination.

import bevy_ecs_tilemap::render::shaders::common::process_fragment;
import bevy_ecs_tilemap::render::shaders::tilemap_vertex_output::MeshVertexOutput;
import bevy_sprite_render::mesh2d::view_bindings::globals;

struct HighlightMaterial {
    hovered_tile: vec2<f32>,
    destination_tile: vec2<f32>,
    hovered_color: vec4<f32>,
    destination_color: vec4<f32>,
};

@group(3) @binding(0)
var<uniform> material: HighlightMaterial;

// How close we are from the border of the tile: 1 on the border, 0 inside.
fn tile_border(tile_uv: vec2<f32>) -> f32 {
    let distance_to_border = min(
        min(tile_uv.x, 1.0 - tile_uv.x),
        min(tile_uv.y, 1.0 - tile_uv.y),
    );
    return 1.0 - smoothstep(0.05, 0.1, distance_to_border);
}

@fragment
fn fragment(in: MeshVertexOutput) -> @location(0) vec4<f32> {
    // Regular tile color, from the tileset texture
    var color = process_fragment(in);

    // Position of the tile in its tilemap, and position of the pixel in this tile
    let tile_position = vec2<f32>(in.storage_position);
    let tile_uv = in.uv.zw;

    // Destination: blinking tint over the whole tile
    if all(tile_position == material.destination_tile) {
        let strength = 0.3 + 0.2 * sin(globals.time * 6.0);
        color = vec4(mix(color.rgb, material.destination_color.rgb, strength), color.a);
    }

    // Tile below the pointer: draw a frame
    if all(tile_position == material.hovered_tile) {
        let strength = tile_border(tile_uv) * material.hovered_color.a;
        color = vec4(mix(color.rgb, material.hovered_color.rgb, strength), color.a);
    }

    return color;
}
