use bevy::{input::common_conditions::input_toggle_active, prelude::*};
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};
use iyes_perf_ui::prelude::*;

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
            // we want Bevy to measure these values for us:
            bevy::diagnostic::FrameTimeDiagnosticsPlugin::default(),
            bevy::diagnostic::EntityCountDiagnosticsPlugin,
            bevy::diagnostic::SystemInformationDiagnosticsPlugin,
            bevy::render::diagnostic::RenderDiagnosticsPlugin,
            PerfUiPlugin,
        ));
        app.add_systems(Startup, setup_help_text);
        app.add_systems(Update, camera::movement);
        app.add_systems(Update, map::rotate);
        app.add_systems(
            Update,
            toggle_perf_ui.before(iyes_perf_ui::PerfUiSet::Setup),
        );
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
            builder.spawn(Text(String::from("Toggle iyes_perf_ui: [F2]")));
            builder.spawn(Text(String::from("Pan camera: [W/A/S/D]")));
            builder.spawn(Text(String::from("Zoom camera: [Z/X]")));
            builder.spawn(Text(String::from("Rotate map / world: [Q/E]")));
        });
}

fn toggle_perf_ui(
    mut commands: Commands,
    q_perf_ui: Query<Entity, With<PerfUiRoot>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::F2) {
        if let Ok(e) = q_perf_ui.single() {
            commands.entity(e).despawn();
        } else {
            commands.spawn(PerfUiAllEntries::default());
        }
    }
}
