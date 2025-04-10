//! This example shows the basic usage of `TilemapAnchor`.

use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;

mod helper;

fn main() {
    App::new()
        // Bevy default plugins: prevent blur effect by changing default sampling.
        .add_plugins(DefaultPlugins.build().set(ImagePlugin::default_nearest()))
        // Add bevy_ecs_tiled plugin: bevy_ecs_tilemap::TilemapPlugin will
        // be automatically added as well if it's not already done.
        .add_plugins(TiledMapPlugin::default())
        // Examples helper plugins, such as the logic to pan and zoom the camera
        // This should not be used directly in your game (but you can always have a look).
        .add_plugins(helper::HelperPlugin)
        // Add our systems and run the app!
        .add_systems(Startup, startup)
        .add_systems(Update, origin_axes)
        .add_systems(Update, change_anchor)
        .run();
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
}

fn origin_axes(mut gizmos: Gizmos) {
    gizmos.axes_2d(Transform::IDENTITY, 1000.0);
}

fn change_anchor(
    mut anchor: Single<&mut TilemapAnchor, With<TiledMapHandle>>,
    key: Res<ButtonInput<KeyCode>>,
) {
    if key.just_pressed(KeyCode::Space) {
        **anchor = rotate_right(&*anchor);
    }
}

fn rotate_right(anchor: &TilemapAnchor) -> TilemapAnchor {
    use TilemapAnchor::*;
    match anchor {
        TopLeft => TopCenter,
        TopCenter => TopRight,
        TopRight => CenterRight,
        CenterRight => BottomRight,
        BottomRight => BottomCenter,
        BottomCenter => BottomLeft,
        BottomLeft => CenterLeft,
        CenterLeft => Center,
        Center => Custom(Vec2::splat(0.25)),
        Custom(_) => None,
        None => TopLeft,
    }
}
