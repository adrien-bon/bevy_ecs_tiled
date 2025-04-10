use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::TilemapAnchor;

/// Show the origin with axes.
pub struct TiledAxisDebugPlugin;

impl Plugin for TiledAxisDebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, origin_axes);
    }
}

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

fn origin_axes(mut gizmos: Gizmos) {
    gizmos.axes_2d(Transform::IDENTITY, 1000.0);
}
