//! This example shows the basic usage of `TilemapAnchor`.

use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;

mod helper;

#[derive(Component)]
struct AnchorLabel;

fn main() {
    let mut app = App::new();
    app
        // Bevy default plugins: prevent blur effect by changing default sampling.
        .add_plugins(DefaultPlugins.build().set(ImagePlugin::default_nearest()))
        // Add bevy_ecs_tiled plugin: bevy_ecs_tilemap::TilemapPlugin will
        // be automatically added as well if it's not already done.
        .add_plugins(TiledMapPlugin::default())
        // Examples helper plugins, such as the logic to pan and zoom the
        // camera. This should not be used directly in your game (but you can
        // always have a look).
        .add_plugins(helper::HelperPlugin)
        // Add our systems and run the app!
        .add_systems(Startup, startup)
        .add_systems(Update, change_anchor);
    #[cfg(feature = "debug")]
    app.add_plugins(bevy_ecs_tiled::debug::axis::TiledAxisDebugPlugin);
    app.run();
}

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Spawn a 2D camera (required by Bevy).
    commands.spawn(Camera2d);

    // Load a map then spawn it.
    commands.spawn((
        // Only the [TiledMapHandle] component is actually required to spawn a map.
        TiledMapHandle(asset_server.load("maps/orthogonal/finite.tmx")),
        // But you can add extra components to change the defaults settings and how
        // your map is actually displayed.
        TilemapAnchor::Center,
    ));

    let font_size = 20.0;
    commands
        .spawn((
            Text::new("[Space] Anchor: "),
            TextFont {
                font_size,
                ..default()
            },
            TextLayout::new_with_justify(JustifyText::Left),
            AnchorLabel,
        ))
        .with_children(|parent| {
            parent.spawn((
                TextSpan::new(format!("{:?}", TilemapAnchor::Center)),
                TextFont {
                    font_size,
                    ..default()
                },
            ));
        });
}

fn change_anchor(
    mut anchor: Single<&mut TilemapAnchor, With<TiledMapHandle>>,
    label: Single<Entity, With<AnchorLabel>>,
    mut writer: TextUiWriter,
    key: Res<ButtonInput<KeyCode>>,
) {
    if key.just_pressed(KeyCode::Space) {
        let new_anchor = helper::anchor::rotate_right(&anchor);
        *writer.text(*label, 1) = format!("{:?}", &new_anchor);
        **anchor = new_anchor;
    }
}
