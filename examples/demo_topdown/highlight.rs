//! A custom tilemap material, to highlight the tile below the pointer and the player destination.
//!
//! Tiles are not drawn one by one but by their tilemap, using a material: the default one
//! only displays the tileset texture. We replace it with our own material for the tilemaps of
//! the layers which have the [`HighlightLayer`] property, so our shader can change the way
//! some tiles are displayed without adding anything to the map.

use bevy::{
    color::palettes::css::{GOLD, WHITE},
    prelude::*,
    reflect::TypePath,
    render::render_resource::AsBindGroup,
    shader::ShaderRef,
};
use bevy_ecs_tiled::prelude::*;

use crate::{
    UpdateSystems,
    player::{GridMover, Player},
};

const HIGHLIGHT_SHADER_FILE: &str = "demo_topdown/highlight.wesl";

/// Value used in our material when there is no tile to highlight.
const NO_TILE: Vec2 = Vec2::splat(-1.);

pub(super) fn plugin(app: &mut App) {
    app.register_type::<HighlightLayer>();
    app.add_plugins(MaterialTilemapPlugin::<HighlightMaterial>::default());
    app.init_resource::<HighlightedTiles>();
    app.add_observer(use_highlight_material);
    app.add_observer(on_tile_over);
    app.add_observer(on_tile_out);
    app.add_systems(
        Update,
        update_highlight_material.in_set(UpdateSystems::Display),
    );
}

/// Marker component for layers whose tiles are displayed using our [`HighlightMaterial`].
///
/// It is added from Tiled, on a tiles layer.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component, Default)]
pub struct HighlightLayer;

/// Tilemap material which highlights two tiles: see the associated shader.
#[derive(Asset, AsBindGroup, TypePath, Debug, Clone)]
pub struct HighlightMaterial {
    /// Position of the tile below the pointer.
    #[uniform(0)]
    hovered_tile: Vec2,
    /// Position of the tile the player is walking to.
    #[uniform(0)]
    destination_tile: Vec2,
    #[uniform(0)]
    hovered_color: LinearRgba,
    #[uniform(0)]
    destination_color: LinearRgba,
}

impl MaterialTilemap for HighlightMaterial {
    fn fragment_shader() -> ShaderRef {
        HIGHLIGHT_SHADER_FILE.into()
    }
}

/// The tiles to highlight and the material, shared by all tilemaps, we use to do it.
#[derive(Resource)]
struct HighlightedTiles {
    material: Handle<HighlightMaterial>,
    hovered: Option<TilePos>,
}

impl FromWorld for HighlightedTiles {
    fn from_world(world: &mut World) -> Self {
        Self {
            material: world
                .resource_mut::<Assets<HighlightMaterial>>()
                .add(HighlightMaterial {
                    hovered_tile: NO_TILE,
                    destination_tile: NO_TILE,
                    hovered_color: WHITE.into(),
                    destination_color: GOLD.into(),
                }),
            hovered: None,
        }
    }
}

/// Replace the default material of the tilemaps created for a [`HighlightLayer`].
fn use_highlight_material(
    tilemap_created: On<TiledEvent<TilemapCreated>>,
    mut commands: Commands,
    highlighted_tiles: Res<HighlightedTiles>,
    layers_query: Query<(), With<HighlightLayer>>,
) {
    let event = tilemap_created.event();
    if event
        .layer_entity()
        .is_some_and(|layer| layers_query.contains(layer))
    {
        commands
            .entity(event.origin)
            .remove::<MaterialTilemapHandle<StandardTilemapMaterial>>()
            .insert(MaterialTilemapHandle::from(
                highlighted_tiles.material.clone(),
            ));
    }
}

/// Remember which tile is below the pointer.
fn on_tile_over(
    over: On<PointerOver>,
    mut highlighted_tiles: ResMut<HighlightedTiles>,
    tiles_query: Query<&TilePos, With<TiledTile>>,
) {
    if let Ok(position) = tiles_query.get(over.entity) {
        highlighted_tiles.hovered = Some(*position);
    }
}

fn on_tile_out(
    out: On<PointerOut>,
    mut highlighted_tiles: ResMut<HighlightedTiles>,
    tiles_query: Query<&TilePos, With<TiledTile>>,
) {
    // Events are not ordered: do not forget the tile the pointer just entered
    if let Ok(position) = tiles_query.get(out.entity)
        && highlighted_tiles.hovered == Some(*position)
    {
        highlighted_tiles.hovered = None;
    }
}

/// Send the tiles to highlight to our shader.
fn update_highlight_material(
    highlighted_tiles: Res<HighlightedTiles>,
    mut materials: ResMut<Assets<HighlightMaterial>>,
    player_single: Option<Single<&GridMover, With<Player>>>,
) {
    let to_vec2 = |position: Option<TilePos>| {
        position.map_or(NO_TILE, |position| UVec2::from(position).as_vec2())
    };
    let hovered_tile = to_vec2(highlighted_tiles.hovered);
    let destination_tile = to_vec2(player_single.and_then(|mover| mover.path.back().copied()));

    // Only access the material mutably when needed, so it is not sent again to the GPU every frame
    if materials
        .get(&highlighted_tiles.material)
        .is_some_and(|material| {
            material.hovered_tile != hovered_tile || material.destination_tile != destination_tile
        })
        && let Some(mut material) = materials.get_mut(&highlighted_tiles.material)
    {
        material.hovered_tile = hovered_tile;
        material.destination_tile = destination_tile;
    }
}
