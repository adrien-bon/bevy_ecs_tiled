use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::TilemapAnchor;

#[allow(dead_code)]
/// Rotate the tilemap anchor to the right generally but also show custom and
/// none for completeness.
pub fn rotate_right(anchor: &TilemapAnchor) -> TilemapAnchor {
    use TilemapAnchor::*;
    match anchor {
        TopLeft => TopCenter,
        TopCenter => TopRight,
        TopRight => CenterRight,
        CenterRight => BottomRight,
        BottomRight => BottomCenter,
        BottomCenter => BottomLeft,
        BottomLeft => CenterLeft,
        CenterLeft => Center,
        Center => Custom(Vec2::splat(0.25)),
        Custom(_) => None,
        None => TopLeft,
    }
}
