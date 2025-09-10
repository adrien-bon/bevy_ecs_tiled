use bevy::{input::common_conditions::input_toggle_active, prelude::*};

use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};
use iyes_perf_ui::prelude::*;

const TOGGLE_INSPECTOR_KEY: KeyCode = KeyCode::F1;
const TOGGLE_IYES_PERF_KEY: KeyCode = KeyCode::F2;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        EguiPlugin::default(),
        WorldInspectorPlugin::default().run_if(input_toggle_active(false, TOGGLE_INSPECTOR_KEY)),
        // we want Bevy to measure these values for us:
        bevy::diagnostic::FrameTimeDiagnosticsPlugin::default(),
        bevy::diagnostic::EntityCountDiagnosticsPlugin,
        bevy::diagnostic::SystemInformationDiagnosticsPlugin,
        bevy::render::diagnostic::RenderDiagnosticsPlugin,
        PerfUiPlugin,
        avian2d::prelude::PhysicsDebugPlugin::default(),
    ));

    app.add_systems(Startup, setup_help_text);

    // Toggle the debug overlay for UI.
    app.add_systems(
        Update,
        (toggle_perf_ui.before(iyes_perf_ui::PerfUiSet::Setup),),
    );
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
            builder.spawn(Text(String::from("Toggle iyes_perf_ui: [F2]")));
        });
}

fn toggle_perf_ui(
    mut commands: Commands,
    q_perf_ui: Query<Entity, With<PerfUiRoot>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if keyboard_input.just_pressed(TOGGLE_IYES_PERF_KEY) {
        if let Ok(e) = q_perf_ui.single() {
            commands.entity(e).despawn();
        } else {
            commands.spawn(PerfUiAllEntries::default());
        }
    }
}
