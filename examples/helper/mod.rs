use bevy::{input::common_conditions::input_toggle_active, prelude::*};
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};

pub mod anchor;
#[allow(dead_code)]
pub mod assets;

mod camera;
mod map;

#[derive(Default)]
pub struct HelperPlugin;

impl Plugin for HelperPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins((
            EguiPlugin::default(),
            WorldInspectorPlugin::default().run_if(input_toggle_active(false, KeyCode::F1)),
        ));
        app.add_systems(Startup, setup_help_text);
        app.add_systems(Update, camera::movement);
        app.add_systems(Update, map::rotate);
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
            builder.spawn(Text(String::from("Toggle inspector: [F1]")));
            builder.spawn(Text(String::from("Pan camera: [W/A/S/D]")));
            builder.spawn(Text(String::from("Zoom camera: [Z/X]")));
            builder.spawn(Text(String::from("Rotate map / world: [Q/E]")));
        });
}
