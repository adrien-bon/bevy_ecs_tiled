//! Player-specific behavior: it walks on the grid, toward the tile which was clicked.

use std::collections::VecDeque;

use bevy::{picking::pointer::PointerButton, prelude::*};
use bevy_ecs_tiled::prelude::*;

use crate::{UpdateSystems, interaction::Destructible, navigation::NavGrid, picking::TileGeometry};

const PLAYER_SPRITE_FILE: &str = "demo_topdown/kenney_sokoban-pack/Tilesheet/sokoban_tilesheet.png";

/// Walking speed, in pixels per second.
const PLAYER_SPEED: f32 = 250.;
/// Depth of the player: tiles layers are all below zero.
const PLAYER_Z: f32 = 1.;
/// Number of frames of the walk animations, and how many of them we display per second.
const WALK_FRAMES: usize = 3;
const WALK_FRAMES_PER_SECOND: f32 = 8.;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<PlayerAssets>();
    app.add_observer(on_tile_clicked);
    app.add_systems(Update, follow_path.in_set(UpdateSystems::Move));
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default)]
#[require(GridPosition, GridMover, Facing)]
pub struct Player;

/// The tile an entity stands on.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Deref, DerefMut)]
pub struct GridPosition(pub TilePos);

/// Where an entity is going, and what it should do once arrived.
#[derive(Component, Debug, Clone, PartialEq, Eq, Default)]
pub struct GridMover {
    /// The next positions to walk through, the last one being the destination.
    pub path: VecDeque<TilePos>,
    /// The tile to interact with once we are next to it.
    pub interact_with: Option<Entity>,
}

impl GridMover {
    pub fn stop(&mut self) {
        self.path.clear();
        self.interact_with = None;
    }
}

/// Which way an entity is looking at.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default)]
enum Facing {
    #[default]
    Down,
    Up,
    Right,
    Left,
}

impl Facing {
    /// Index of the first frame of the walk animation in the texture atlas.
    fn atlas_index(self) -> usize {
        match self {
            Facing::Down => 52,
            Facing::Up => 55,
            Facing::Right => 78,
            Facing::Left => 81,
        }
    }
}

/// Assets used by the player: they are loaded only once.
#[derive(Resource)]
pub struct PlayerAssets {
    image: Handle<Image>,
    layout: Handle<TextureAtlasLayout>,
}

impl FromWorld for PlayerAssets {
    fn from_world(world: &mut World) -> Self {
        Self {
            image: world.resource::<AssetServer>().load(PLAYER_SPRITE_FILE),
            // The player uses the same image as the map tileset: 13x8 tiles of 64 pixels.
            // Tiles touch each other in this image, so leave out their outermost pixels
            // to prevent the neighboring tiles from bleeding on our sprite.
            layout: world.resource_mut::<Assets<TextureAtlasLayout>>().add(
                TextureAtlasLayout::from_grid(
                    UVec2::splat(62),
                    13,
                    8,
                    Some(UVec2::splat(2)),
                    Some(UVec2::splat(1)),
                ),
            ),
        }
    }
}

impl PlayerAssets {
    /// Returns the components of a player standing on the provided tile.
    pub fn player(&self, position: TilePos, world_position: Vec2) -> impl Bundle {
        (
            Name::new("Player"),
            Player,
            GridPosition(position),
            Transform::from_translation(world_position.extend(PLAYER_Z)),
            Sprite {
                image: self.image.clone(),
                texture_atlas: Some(TextureAtlas {
                    layout: self.layout.clone(),
                    index: Facing::default().atlas_index(),
                }),
                ..default()
            },
        )
    }
}

/// Send the player to the tile which was clicked.
///
/// Tiles receive pointer events thanks to our picking backend. Since only the topmost tile
/// below the pointer gets the event, we know what was clicked from its components:
/// - a destructible tile: walk next to it, then break it
/// - an obstacle: there is no path to it, nothing happens
/// - anything else: walk to it
fn on_tile_clicked(
    mut click: On<PointerClick>,
    tiles_query: Query<(&TilePos, Has<Destructible>), With<TiledTile>>,
    nav_grid: Res<NavGrid>,
    player_single: Single<(&GridPosition, &mut GridMover), With<Player>>,
) {
    let Ok((tile_position, is_destructible)) = tiles_query.get(click.entity) else {
        return;
    };
    // We handled this click: do not send it to the layer and the map of the tile
    click.propagate(false);
    if click.button != PointerButton::Primary {
        return;
    }

    let (player_position, mut mover) = player_single.into_inner();
    // If we are already walking, the new path starts from the tile we are about to reach
    let start = mover.path.front().copied().unwrap_or(player_position.0);
    let mut new_path = |goal: TilePos| {
        nav_grid.find_path(start, goal).map(|path| {
            let mut path = VecDeque::from(path);
            if start != player_position.0 {
                path.push_front(start);
            }
            path
        })
    };

    if is_destructible {
        // We can not walk on the tile itself: go to the closest position next to it
        let neighbors = nav_grid.neighbors(*tile_position).collect::<Vec<_>>();
        if let Some(path) = neighbors
            .into_iter()
            .filter_map(&mut new_path)
            .min_by_key(|path| path.len())
        {
            mover.path = path;
            mover.interact_with = Some(click.entity);
        }
    } else if let Some(path) = new_path(*tile_position) {
        mover.path = path;
        mover.interact_with = None;
    }
}

/// Move the player to the next position of its path.
fn follow_path(
    time: Res<Time>,
    geometry: TileGeometry,
    nav_grid: Res<NavGrid>,
    player_single: Single<
        (
            &mut Transform,
            &mut GridPosition,
            &mut GridMover,
            &mut Facing,
            &mut Sprite,
        ),
        With<Player>,
    >,
) {
    let (mut transform, mut position, mut mover, mut facing, mut sprite) =
        player_single.into_inner();

    let mut frame = 0;
    if let Some(next_position) = mover.path.front().copied() {
        if !nav_grid.is_walkable(next_position) {
            // The map changed since we computed our path
            mover.stop();
        } else if let Some(target) = geometry.tile_center(next_position) {
            let to_target = target - transform.translation.truncate();
            let step = PLAYER_SPEED * time.delta_secs();
            if to_target.length() <= step {
                // We reached the center of the tile
                transform.translation = target.extend(PLAYER_Z);
                position.0 = next_position;
                mover.path.pop_front();
            } else {
                transform.translation += (to_target.normalize() * step).extend(0.);
                *facing = if to_target.x.abs() > to_target.y.abs() {
                    if to_target.x > 0. {
                        Facing::Right
                    } else {
                        Facing::Left
                    }
                } else if to_target.y > 0. {
                    Facing::Up
                } else {
                    Facing::Down
                };
            }
            frame = (time.elapsed_secs() * WALK_FRAMES_PER_SECOND) as usize % WALK_FRAMES;
        }
    }

    if let Some(atlas) = sprite.texture_atlas.as_mut() {
        let index = facing.atlas_index() + frame;
        if atlas.index != index {
            atlas.index = index;
        }
    }
}
