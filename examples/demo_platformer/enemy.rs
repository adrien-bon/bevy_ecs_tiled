//! Enemy-specific behavior.

use std::time::Duration;

use avian2d::prelude::*;
use bevy::{color::palettes::css::RED, prelude::*, sprite::Anchor};

use crate::{
    animation::{Animation, AnimationState},
    controller::character_controller,
    minimap::{HideOnMinimap, minimap_marker},
};

const ENEMY_SPRITE_FILE: &str =
    "demo_platformer/kenney_platformer-pack-redux/Spritesheets/spritesheet_enemies.png";

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Enemy>();
    app.init_resource::<EnemyAssets>();
    app.add_observer(on_enemy_added);
}

/// Marker component for enemies.
///
/// It is added from Tiled, on a point object: we then add everything else
/// an enemy needs to the object entity.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[require(Transform, Visibility)]
#[reflect(Component)]
pub struct Enemy;

/// Assets shared by all the enemies: they are loaded only once.
#[derive(Resource)]
struct EnemyAssets {
    image: Handle<Image>,
    layout: Handle<TextureAtlasLayout>,
}

impl FromWorld for EnemyAssets {
    fn from_world(world: &mut World) -> Self {
        Self {
            image: world.resource::<AssetServer>().load(ENEMY_SPRITE_FILE),
            layout: world.resource_mut::<Assets<TextureAtlasLayout>>().add(
                TextureAtlasLayout::from_grid(
                    UVec2::splat(128),
                    8,
                    16,
                    Some(UVec2::splat(2)),
                    None,
                ),
            ),
        }
    }
}

fn on_enemy_added(
    add_enemy: On<Add<Enemy>>,
    mut commands: Commands,
    enemy_assets: Res<EnemyAssets>,
) {
    let enemy_animation = Animation::default()
        .with_state(
            AnimationState::Idling,
            Duration::from_millis(100),
            vec![75, 83],
        )
        .with_state(
            AnimationState::Walking,
            Duration::from_millis(100),
            vec![75, 83],
        );

    commands.entity(add_enemy.event().entity).insert((
        Name::new("Enemy"),
        Sprite {
            image: enemy_assets.image.clone(),
            texture_atlas: Some(TextureAtlas {
                layout: enemy_assets.layout.clone(),
                index: 75,
            }),
            ..Default::default()
        },
        HideOnMinimap,
        Anchor::from(Vec2::new(0., -0.1)),
        enemy_animation,
        // Enemies can not be pushed around by the player
        Mass(1_000_000.),
        character_controller(Collider::capsule(40., 30.)),
        children![minimap_marker(RED)],
    ));
}
