use bevy::{
    dev_tools::diagnostics_overlay::*, diagnostic::FrameTimeDiagnosticsPlugin,
    input::common_conditions::input_toggle_active, prelude::*,
};

use bevy_inspector_egui::{
    bevy_egui::{EguiGlobalSettings, EguiPlugin},
    quick::WorldInspectorPlugin,
};

const TOGGLE_INSPECTOR_KEY: KeyCode = KeyCode::F1;

pub(super) fn plugin(app: &mut App) {
    // We want to render egui on the camera with 'PrimaryEguiContext' component
    app.insert_resource(EguiGlobalSettings {
        auto_create_primary_context: false,
        ..Default::default()
    });
    app.add_plugins((
        EguiPlugin::default(),
        WorldInspectorPlugin::default().run_if(input_toggle_active(false, TOGGLE_INSPECTOR_KEY)),
        avian2d::prelude::PhysicsDebugPlugin,
        DiagnosticsOverlayPlugin,
        FrameTimeDiagnosticsPlugin::default(),
    ));

    app.add_systems(Startup, (setup_help_text, setup_diagnostics_overlay));
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
            builder.spawn(Text(String::from("Debug mode enabled")));
            builder.spawn(Text(String::from("Toggle inspector: [F1]")));
        });
}

fn setup_diagnostics_overlay(mut commands: Commands) {
    commands.spawn((Name::new("Diagnostic overlay"), DiagnosticsOverlay::fps()));
}
