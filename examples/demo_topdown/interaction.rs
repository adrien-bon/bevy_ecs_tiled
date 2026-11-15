//! Tiles the player can interact with: they modify the map while the game is running.
//!
//! Tiles entities only hold data: when we despawn a tile or change its `TiledTile` component,
//! `bevy_ecs_tiled` updates what is displayed. Our navigation grid follows as well, since it
//! is also built from these entities.

use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;

use crate::{
    UpdateSystems,
    navigation::NavGrid,
    player::{GridMover, GridPosition, Player},
};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Destructible>();
    app.register_type::<Channel>();
    app.register_type::<Button>();
    app.register_type::<Gate>();
    app.add_systems(
        Update,
        (break_destructible_tile, press_button).in_set(UpdateSystems::Interact),
    );
}

/// Marker component for tiles the player can break.
///
/// It is added from Tiled, on the tiles of the tileset.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component, Default)]
pub struct Destructible;

/// Links a [`Button`] with the [`Gate`]s it opens.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Default)]
pub enum Channel {
    #[default]
    Red,
    Blue,
}

/// A tile which opens all the [`Gate`]s of its channel when the player walks on it.
///
/// It is added from Tiled, on the tiles of the tileset.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component, Default)]
pub struct Button {
    pub channel: Channel,
    /// ID, in the same tileset, of the tile to display once the button is pressed.
    pub pressed_tile: u32,
}

/// A tile which is removed when a [`Button`] of its channel is pressed.
///
/// It is added from Tiled, on the tiles of the tileset.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component, Default)]
pub struct Gate {
    pub channel: Channel,
}

/// Marker component for a [`Button`] which was already pressed.
#[derive(Component)]
struct Pressed;

/// Break the tile the player was sent to, once it has arrived next to it.
fn break_destructible_tile(
    mut commands: Commands,
    nav_grid: Res<NavGrid>,
    player_single: Single<(&GridPosition, &mut GridMover), With<Player>>,
    tiles_query: Query<&TilePos, With<Destructible>>,
) {
    let (player_position, mut mover) = player_single.into_inner();
    if !mover.path.is_empty() {
        return;
    }
    let Some(tile) = mover.interact_with.take() else {
        return;
    };

    if let Ok(tile_position) = tiles_query.get(tile)
        && nav_grid
            .neighbors(*tile_position)
            .any(|neighbor| neighbor == player_position.0)
    {
        // Despawn the tile: it is removed from its tilemap, and from our navigation grid
        commands.entity(tile).despawn();
    }
}

/// Press the button the player just walked on.
fn press_button(
    mut commands: Commands,
    player_single: Single<&GridPosition, (With<Player>, Changed<GridPosition>)>,
    mut buttons_query: Query<(Entity, &Button, &TilePos, &mut TiledTile), Without<Pressed>>,
    gates_query: Query<(Entity, &Gate)>,
) {
    for (entity, button, button_position, mut tile) in &mut buttons_query {
        if *button_position != player_single.0 {
            continue;
        }

        // Change the tile ID: the tilemap is updated to display the new tile
        tile.tile_id = button.pressed_tile;
        commands.entity(entity).insert(Pressed);

        for (gate, _) in gates_query
            .iter()
            .filter(|(_, gate)| gate.channel == button.channel)
        {
            commands.entity(gate).despawn();
        }
    }
}
