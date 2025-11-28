use bevy::{input::common_conditions::input_toggle_active, prelude::*};

use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};

const TOGGLE_INSPECTOR_KEY: KeyCode = KeyCode::F1;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        EguiPlugin::default(),
        WorldInspectorPlugin::default().run_if(input_toggle_active(false, TOGGLE_INSPECTOR_KEY)),
        avian2d::prelude::PhysicsDebugPlugin,
    ));

    app.add_systems(Startup, setup_help_text);
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
            builder.spawn(Text(String::from("Debug mode enabled")));
            builder.spawn(Text(String::from("Toggle inspector: [F1]")));
        });
}
