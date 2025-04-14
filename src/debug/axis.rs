//! Axis debug
//!
//! Shows the origin
use bevy::prelude::*;

#[derive(Debug, Copy)]
/// Show the origin with axes.
pub struct TiledAxisDebugPlugin;

impl Plugin for TiledAxisDebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, origin_axes);
    }
}

fn origin_axes(mut gizmos: Gizmos) {
    gizmos.axes_2d(Transform::IDENTITY, 1000.0);
}
