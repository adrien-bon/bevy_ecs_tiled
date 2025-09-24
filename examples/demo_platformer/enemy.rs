use std::time::Duration;

use avian2d::{math::*, prelude::*};
use bevy::{prelude::*, sprite::Anchor};

use crate::{
    animation::{Animation, AnimationState, AnimationStateConfig},
    controller::CharacterControllerBundle,
};

const ENEMY_SPRITE_FILE: &str =
    "demo_platformer/kenney_platformer-pack-redux/Spritesheets/spritesheet_enemies.png";

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Enemy>();
    app.add_observer(on_enemy_added);
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[require(Transform, Visibility)]
#[reflect(Component)]
pub struct Enemy;

fn on_enemy_added(
    trigger: Trigger<OnAdd, Enemy>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let layout =
        TextureAtlasLayout::from_grid(UVec2::splat(128), 8, 16, Some(UVec2::splat(2)), None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);

    let enemy_animation = Animation::default()
        .add_config(
            AnimationState::Idling,
            AnimationStateConfig {
                duration: Duration::from_millis(100),
                frames: vec![75, 83],
            },
        )
        .add_config(
            AnimationState::Walking,
            AnimationStateConfig {
                duration: Duration::from_millis(100),
                frames: vec![75, 83],
            },
        );

    commands.entity(trigger.target()).insert((
        Name::new("Enemy"),
        Sprite {
            image: asset_server.load(ENEMY_SPRITE_FILE),
            texture_atlas: Some(TextureAtlas {
                layout: texture_atlas_layout,
                index: 75,
            }),
            anchor: Anchor::Custom(Vec2::new(0., -0.1)),
            ..Default::default()
        },
        enemy_animation,
        Mass(1_000_000.),
        CharacterControllerBundle::new(Collider::capsule(40., 30.)).with_movement(
            2000.,
            0.9,
            600.,
            PI * 0.45,
        ),
    ));
}
