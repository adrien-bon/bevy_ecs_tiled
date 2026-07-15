use bevy::{
    dev_tools::diagnostics_overlay::*, diagnostic::FrameTimeDiagnosticsPlugin,
    input::common_conditions::input_toggle_active, prelude::*,
};
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
            DiagnosticsOverlayPlugin,
            FrameTimeDiagnosticsPlugin::default(),
        ));
        app.add_systems(Startup, (setup_help_text, setup_diagnostics_overlay));
        app.add_systems(Update, (camera::movement, map::rotate));
    }
}

fn setup_help_text(mut commands: Commands) {
    commands
        .spawn((
            Name::new("Helper text"),
            Node {
                display: Display::Flex,
                align_self: AlignSelf::FlexEnd,
                flex_direction: FlexDirection::Column,
                ..default()
            },
        ))
        .with_children(|builder| {
            builder.spawn(Text(String::from("Toggle inspector: [F1]")));
            builder.spawn(Text(String::from("Pan camera: [W/A/S/D]")));
            builder.spawn(Text(String::from("Zoom camera: [Z/X]")));
            builder.spawn(Text(String::from("Rotate map / world: [Q/E]")));
        });
}

fn setup_diagnostics_overlay(mut commands: Commands) {
    commands.spawn((Name::new("Diagnostic overlay"), DiagnosticsOverlay::fps()));
}
