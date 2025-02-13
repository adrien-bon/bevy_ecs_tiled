use bevy::input::common_conditions::input_toggle_active;
use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

#[allow(dead_code)]
pub mod assets;

mod camera;
mod map;

#[derive(Default)]
pub struct HelperPlugin;

impl Plugin for HelperPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins(
            WorldInspectorPlugin::default().run_if(input_toggle_active(false, KeyCode::Escape)),
        );
        app.add_systems(Update, camera::movement);
        app.add_systems(Update, map::rotate);
        app.add_systems(Startup, setup_help_text);
    }
}

fn setup_help_text(mut commands: Commands) {
    commands
        .spawn(Node {
            display: Display::Flex,
            align_self: AlignSelf::FlexEnd,
            flex_direction: FlexDirection::Column,
            ..default()
        })
        .with_children(|builder| {
            builder.spawn(Text("Toggle inspector: [Echap]".to_string()));
            builder.spawn(Text("Pan camera: [W/A/S/D]".to_string()));
            builder.spawn(Text("Zoom camera: [Z/X]".to_string()));
            builder.spawn(Text("Rotate map / world: [Q/E]".to_string()));
        });
}
