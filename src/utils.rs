//! This modules contains utilities functions.
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

/// Convert from Tiled isometric coordinates to Bevy position.
///
/// This function will convert provided Tiled raw isometric position to a Bevy position, according to various maps settings.
///
/// Example:
/// ```rust,no_run
/// use bevy::prelude::*;
/// use bevy_ecs_tiled::prelude::*;
/// use bevy_ecs_tilemap::prelude::*;
///
/// let tiled_position = Vec2::new(0., 12.);
/// let coords = from_isometric_coords_to_bevy(
///     IsoCoordSystem::Diamond,
///     tiled_position,
///     &TilemapSize::new(0, 3),
///     &TilemapGridSize::new(16., 16.),
/// );
/// ```
pub fn from_isometric_coords_to_bevy(
    iso_coords: IsoCoordSystem,
    tiled_position: Vec2,
    map_size: &TilemapSize,
    grid_size: &TilemapGridSize,
) -> Vec2 {
    match iso_coords {
        IsoCoordSystem::Diamond => Vec2::new(
            ((tiled_position.x - tiled_position.y) / grid_size.y + map_size.y as f32) * grid_size.x
                / 2.,
            (map_size.y as f32
                - tiled_position.x / grid_size.y
                - tiled_position.y / grid_size.y
                - 1.)
                * grid_size.y
                / 2.,
        ),
        IsoCoordSystem::Staggered => {
            //warn!("Isometric (Staggered) map is not supported");
            tiled_position
        }
    }
}
