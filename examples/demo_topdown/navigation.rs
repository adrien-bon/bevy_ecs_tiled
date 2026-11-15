//! Navigation grid built from the map tiles, and pathfinding on this grid.

use std::{cmp::Reverse, collections::BinaryHeap};

use bevy::{
    platform::collections::{HashMap, HashSet},
    prelude::*,
};
use bevy_ecs_tiled::prelude::*;

use crate::UpdateSystems;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Obstacle>();
    app.init_resource::<NavGrid>();
    app.add_systems(Update, update_nav_grid.in_set(UpdateSystems::Navigation));
}

/// Marker component for tiles which can not be walked on.
///
/// It is added from Tiled, on the tiles of the tileset: every tile of the map
/// using one of them automatically gets this component.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component, Default)]
pub struct Obstacle;

/// The positions the player can walk on, for the map currently loaded.
///
/// A position is walkable if there is at least a tile there and if none of the tiles
/// at this position, whatever their layer, is an [`Obstacle`].
#[derive(Resource, Debug, Default)]
pub struct NavGrid {
    walkable: HashSet<TilePos>,
}

impl NavGrid {
    pub fn is_walkable(&self, position: TilePos) -> bool {
        self.walkable.contains(&position)
    }

    /// The walkable positions next to the provided one.
    pub fn neighbors(&self, position: TilePos) -> impl Iterator<Item = TilePos> + '_ {
        [
            position
                .x
                .checked_sub(1)
                .map(|x| TilePos::new(x, position.y)),
            position
                .y
                .checked_sub(1)
                .map(|y| TilePos::new(position.x, y)),
            Some(TilePos::new(position.x + 1, position.y)),
            Some(TilePos::new(position.x, position.y + 1)),
        ]
        .into_iter()
        .flatten()
        .filter(|neighbor| self.is_walkable(*neighbor))
    }

    /// Find the shortest path between two positions, using the A* algorithm.
    ///
    /// The returned path does not contain `start`: it is empty if we are already there.
    pub fn find_path(&self, start: TilePos, goal: TilePos) -> Option<Vec<TilePos>> {
        if !self.is_walkable(goal) {
            return None;
        }

        // We can only move horizontally or vertically
        let distance_to_goal =
            |position: TilePos| position.x.abs_diff(goal.x) + position.y.abs_diff(goal.y);

        // Positions to explore, the most promising one first
        let mut open = BinaryHeap::from([Reverse((distance_to_goal(start), 0, start))]);
        let mut costs = HashMap::from([(start, 0)]);
        let mut came_from = HashMap::new();

        while let Some(Reverse((_, cost, current))) = open.pop() {
            if current == goal {
                // Walk back to the start to build the path
                let mut path = Vec::new();
                let mut position = goal;
                while position != start {
                    path.push(position);
                    position = came_from[&position];
                }
                path.reverse();
                return Some(path);
            }
            if cost > costs[&current] {
                // We already found a better way to reach this position
                continue;
            }
            for neighbor in self.neighbors(current) {
                let neighbor_cost = cost + 1;
                if costs
                    .get(&neighbor)
                    .is_none_or(|known_cost| neighbor_cost < *known_cost)
                {
                    costs.insert(neighbor, neighbor_cost);
                    came_from.insert(neighbor, current);
                    open.push(Reverse((
                        neighbor_cost + distance_to_goal(neighbor),
                        neighbor_cost,
                        neighbor,
                    )));
                }
            }
        }
        None
    }
}

/// Filter for tiles which were added or moved, or which became an obstacle.
type ModifiedTile = Or<(Added<TiledTile>, Added<Obstacle>, Changed<TilePos>)>;

/// Build the navigation grid again when tiles are added, moved or removed.
///
/// Since the grid is only built from the tiles entities, it does not matter whether
/// they come from the map we loaded or if they were modified afterward.
fn update_nav_grid(
    mut nav_grid: ResMut<NavGrid>,
    tiles_query: Query<(&TilePos, Has<Obstacle>), With<TiledTile>>,
    modified_tiles_query: Query<(), ModifiedTile>,
    mut removed_tiles: RemovedComponents<TiledTile>,
    mut removed_obstacles: RemovedComponents<Obstacle>,
) {
    // Always consume removal events, so we do not handle them twice
    let removed = removed_tiles.read().count() + removed_obstacles.read().count();
    if removed == 0 && modified_tiles_query.is_empty() {
        return;
    }

    nav_grid.walkable = tiles_query.iter().map(|(position, _)| *position).collect();
    for (position, is_obstacle) in &tiles_query {
        if is_obstacle {
            nav_grid.walkable.remove(position);
        }
    }
}
