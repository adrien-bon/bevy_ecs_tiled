use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

mod camera;
mod map;

#[cfg(feature = "rapier")]
pub mod rapier;

#[derive(Default)]
pub struct HelperPlugin;

impl Plugin for HelperPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins(WorldInspectorPlugin::new());
        app.add_systems(Update, camera::movement);
        app.add_systems(Update, map::rotate);
        #[cfg(feature = "rapier")]
        app.add_systems(Update, rapier::move_player);
    }
}
