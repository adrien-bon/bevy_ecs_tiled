//! Axis debug visualization for bevy_ecs_tiled.
//!
//! This module provides a simple plugin to display the world origin and axes in 2D using Bevy's gizmo system.
//! It is useful for debugging map alignment, orientation, and coordinate systems in Tiled maps.

use bevy::prelude::*;

#[derive(Debug, Clone, Copy)]
/// Plugin to display the world origin and axes for debugging purposes.
///
/// When enabled, this plugin draws 2D axes at the origin using Bevy's gizmo system.
/// This helps visualize the coordinate system and origin placement in your Tiled maps.
pub struct TiledDebugAxisPlugin;

impl Plugin for TiledDebugAxisPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, origin_axes);
    }
}

fn origin_axes(mut gizmos: Gizmos) {
    gizmos.axes_2d(Transform::IDENTITY, 1000.0);
}
