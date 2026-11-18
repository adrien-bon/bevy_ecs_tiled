//! ECS components for Tiled layers.
//!
//! This module defines Bevy components used to represent Tiled layers within the ECS world.
//! The [`TiledLayer`] enum allows systems to identify and differentiate between various types of Tiled layers,
//! such as tile layers, object layers, image layers, and group layers.

use crate::prelude::*;
use bevy::{platform::collections::HashMap, prelude::*};

/// ID of a layer, as defined in Tiled.
///
/// This ID is unique within a map, whatever the layer type.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord, Reflect)]
#[reflect(Clone, Debug, Default, PartialEq, Hash)]
pub struct LayerId(pub u32);

crate::tiled::helpers::impl_id!(LayerId);

/// Marker [`Component`] for a Tiled map layer.
///
/// This enum is attached to entities representing Tiled layers in the ECS world.
/// Each variant corresponds to a specific Tiled layer type and indicates the expected children entities.
///
/// - `Tiles`: A layer containing tiles, parent of [`TiledTile`] and [`TiledTilemap`] entities.
/// - `Objects`: A layer containing objects, parent of [`TiledObject`] entities.
/// - `Image`: A layer containing an image, parent of a single [`TiledImage`] entity.
/// - `Group`: A group of layers, used to organize multiple layers hierarchically.
#[derive(Component, Reflect, Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[reflect(Component, Debug, PartialEq, Hash)]
#[require(Visibility, Transform, TiledName, TiledTileLayerStorage)]
pub enum TiledLayer {
    /// A layer containing tiles.
    ///
    /// Parent of [`TiledTile`] entities and of the [`TiledTilemap`] entities rendering them.
    Tiles,
    /// A layer containing objects.
    ///
    /// Parent of [`TiledObject`] entities.
    Objects,
    /// A layer containing an image.
    ///
    /// Parent of a single [`TiledImage`] entity.
    Image,
    /// A group of layers, used to organize multiple layers hierarchically.
    Group,
}

/// [`Component`] indexing the [`TiledTile`] entities of a [`TiledLayer::Tiles`] by their [`TilePos`].
///
/// It is automatically added to all [`TiledLayer`] entities, but it only holds tiles for
/// [`TiledLayer::Tiles`] ones: a position of a tiles layer holds at most one tile.
///
/// It is kept up to date when tiles are added, moved or removed after the map has been spawned.
/// Note that new or moved tiles are only registered during [`TiledPostUpdateSystems::SyncMapStorage`].
///
/// Example:
/// ```rust,no_run
/// use bevy::prelude::*;
/// use bevy_ecs_tiled::prelude::*;
///
/// /// Despawn the tile at a given position, for all tiles layers
/// fn despawn_tiles(mut commands: Commands, layers: Query<&TiledTileLayerStorage>) {
///     for storage in layers.iter() {
///         if let Some(tile) = storage.get(TilePos::new(3, 4)) {
///             commands.entity(tile).despawn();
///         }
///     }
/// }
/// ```
#[derive(Component, Default, Reflect, Clone, Debug)]
#[reflect(opaque, Component, Default, Debug)]
pub struct TiledTileLayerStorage {
    /// Tiles are stored by square blocks of positions: only blocks holding at least one tile are allocated,
    /// so the storage does not depend on the map size and handles any [`TilePos`].
    blocks: HashMap<UVec2, StorageBlock>,
    /// Number of tiles in the storage
    len: usize,
}

/// A square block of [`TiledTileLayerStorage::BLOCK_SIZE`] positions.
#[derive(Clone, Debug)]
struct StorageBlock {
    tiles: Box<[Option<Entity>]>,
    /// Number of tiles in the block
    count: u32,
}

impl Default for StorageBlock {
    fn default() -> Self {
        Self {
            tiles: vec![
                None;
                (TiledTileLayerStorage::BLOCK_SIZE * TiledTileLayerStorage::BLOCK_SIZE)
                    as usize
            ]
            .into_boxed_slice(),
            count: 0,
        }
    }
}

impl TiledTileLayerStorage {
    /// Size of a storage block, in tiles
    const BLOCK_SIZE: u32 = 16;

    /// Block and index in this block of a position
    fn locate(position: TilePos) -> (UVec2, usize) {
        let position = UVec2::new(position.x, position.y);
        let local = position % Self::BLOCK_SIZE;
        (
            position / Self::BLOCK_SIZE,
            (local.y * Self::BLOCK_SIZE + local.x) as usize,
        )
    }

    /// Returns the [`TiledTile`] [`Entity`] at the given [`TilePos`], if any.
    pub fn get(&self, position: TilePos) -> Option<Entity> {
        let (block, index) = Self::locate(position);
        self.blocks.get(&block).and_then(|block| block.tiles[index])
    }

    /// Returns the number of tiles of the layer.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if the layer does not have any tile.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns an iterator over all the tiles of the layer and their position, in no particular order.
    pub fn iter(&self) -> impl Iterator<Item = (TilePos, Entity)> + '_ {
        self.blocks
            .iter()
            .flat_map(|(block_pos, block)| Self::block_tiles(*block_pos, block))
    }

    /// Returns an iterator over the tiles whose position is within the given rectangle
    /// (`min` and `max` positions are both included), in no particular order.
    pub fn iter_in(
        &self,
        min: TilePos,
        max: TilePos,
    ) -> impl Iterator<Item = (TilePos, Entity)> + '_ {
        let (min_block, max_block) = (Self::locate(min).0, Self::locate(max).0);
        let in_rect = move |position: &TilePos| {
            (min.x..=max.x).contains(&position.x) && (min.y..=max.y).contains(&position.y)
        };
        // Either visit the blocks of the rectangle, or all blocks if there are fewer of them
        let blocks_in_rect = (max_block.x.saturating_sub(min_block.x) as usize + 1)
            .saturating_mul(max_block.y.saturating_sub(min_block.y) as usize + 1);
        let blocks: Box<dyn Iterator<Item = (UVec2, &StorageBlock)> + '_> = if min.x > max.x
            || min.y > max.y
        {
            Box::new(std::iter::empty())
        } else if blocks_in_rect < self.blocks.len() {
            Box::new(
                (min_block.y..=max_block.y)
                    .flat_map(move |y| (min_block.x..=max_block.x).map(move |x| UVec2::new(x, y)))
                    .filter_map(|block_pos| {
                        self.blocks.get(&block_pos).map(|block| (block_pos, block))
                    }),
            )
        } else {
            Box::new(self.blocks.iter().filter_map(move |(block_pos, block)| {
                ((min_block.x..=max_block.x).contains(&block_pos.x)
                    && (min_block.y..=max_block.y).contains(&block_pos.y))
                .then_some((*block_pos, block))
            }))
        };
        blocks
            .flat_map(|(block_pos, block)| Self::block_tiles(block_pos, block))
            .filter(move |(position, _)| in_rect(position))
    }

    /// Tiles of a block and their position
    fn block_tiles(
        block_pos: UVec2,
        block: &StorageBlock,
    ) -> impl Iterator<Item = (TilePos, Entity)> + '_ {
        block
            .tiles
            .iter()
            .enumerate()
            .filter_map(move |(index, tile)| {
                let index = index as u32;
                let position = block_pos * Self::BLOCK_SIZE
                    + UVec2::new(index % Self::BLOCK_SIZE, index / Self::BLOCK_SIZE);
                tile.map(|entity| (TilePos::new(position.x, position.y), entity))
            })
    }

    /// Register a [`TiledTile`] [`Entity`] at the given position, returning the one which was previously there
    pub(crate) fn set(&mut self, position: TilePos, entity: Entity) -> Option<Entity> {
        let (block, index) = Self::locate(position);
        let block = self.blocks.entry(block).or_default();
        let previous = block.tiles[index].replace(entity);
        if previous.is_none() {
            block.count += 1;
            self.len += 1;
        }
        previous
    }

    /// Unregister a [`TiledTile`] [`Entity`] from the given position, if it is registered there
    pub(crate) fn remove(&mut self, position: TilePos, entity: Entity) {
        let (block_pos, index) = Self::locate(position);
        let Some(block) = self.blocks.get_mut(&block_pos) else {
            return;
        };
        if block.tiles[index] != Some(entity) {
            return;
        }
        block.tiles[index] = None;
        block.count -= 1;
        self.len -= 1;
        if block.count == 0 {
            self.blocks.remove(&block_pos);
        }
    }
}

/// Component that stores parallax information for Tiled layers.
#[derive(Component, Reflect, Clone, Debug, Copy)]
#[reflect(Component, Debug)]
pub struct TiledLayerParallax {
    /// The horizontal (`x`) and vertical (`y`) parallax factors, as defined in Tiled.
    ///
    /// `0.0` keeps the layer fixed relative to the camera, `1.0` moves it at the same speed than the map.
    pub parallax: Vec2,
    /// The base position of the layer before parallax is applied.
    pub base_position: Vec2,
}

/// Component that marks the camera to use for parallax calculations.
#[derive(Component, Reflect, Clone, Debug, Copy)]
#[reflect(Component, Debug)]
pub struct TiledParallaxCamera;

pub(crate) fn plugin(app: &mut App) {
    app.register_type::<LayerId>();
    app.register_type::<TiledLayer>();
    app.register_type::<TiledTileLayerStorage>();
    app.register_type::<TiledLayerParallax>();
    app.register_type::<TiledParallaxCamera>();
    app.add_systems(
        Update,
        update_layer_parallax.in_set(TiledUpdateSystems::UpdateParallaxLayers),
    );
}

fn update_layer_parallax(
    camera_query: Query<&Transform, (With<TiledParallaxCamera>, Changed<Transform>)>,
    mut layer_query: Query<(&TiledLayerParallax, &mut Transform), Without<TiledParallaxCamera>>,
) {
    let Ok(camera_transform) = camera_query.single() else {
        return;
    };

    for (parallax, mut transform) in layer_query.iter_mut() {
        let camera_position = Vec2::new(
            camera_transform.translation.x,
            camera_transform.translation.y,
        );
        if parallax.parallax.x != 1.0 {
            let offset_x = camera_position.x * (1.0 - parallax.parallax.x);
            transform.translation.x = parallax.base_position.x + offset_x;
        }
        if parallax.parallax.y != 1.0 {
            let offset_y = camera_position.y * (1.0 - parallax.parallax.y);
            transform.translation.y = parallax.base_position.y + offset_y;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sorted(tiles: impl Iterator<Item = (TilePos, Entity)>) -> Vec<(TilePos, Entity)> {
        let mut tiles = tiles.collect::<Vec<_>>();
        tiles.sort();
        tiles
    }

    #[test]
    fn storage_set_get_and_remove() {
        let mut world = World::new();
        let (a, b) = (world.spawn_empty().id(), world.spawn_empty().id());
        let (p1, p2) = (TilePos::new(1, 2), TilePos::new(1000, 70000));

        let mut storage = TiledTileLayerStorage::default();
        assert!(storage.is_empty());
        assert_eq!(storage.set(p1, a), None);
        assert_eq!(storage.set(p2, b), None);
        assert_eq!(storage.len(), 2);
        assert_eq!(storage.get(p1), Some(a));
        assert_eq!(storage.get(p2), Some(b));
        assert_eq!(storage.get(TilePos::new(2, 1)), None);
        assert_eq!(sorted(storage.iter()), vec![(p1, a), (p2, b)]);

        // Setting a position returns the previous tile
        assert_eq!(storage.set(p1, b), Some(a));
        assert_eq!(storage.len(), 2);

        // Removing an entity which is not at this position does nothing
        storage.remove(p1, a);
        assert_eq!(storage.get(p1), Some(b));
        storage.remove(p1, b);
        storage.remove(p1, b);
        assert_eq!(storage.get(p1), None);
        assert_eq!(storage.len(), 1);

        // Empty blocks are released
        storage.remove(p2, b);
        assert!(storage.is_empty());
        assert!(storage.blocks.is_empty());
    }

    #[test]
    fn storage_iterates_over_a_rectangle() {
        let mut world = World::new();
        let mut storage = TiledTileLayerStorage::default();
        let mut tiles = Vec::new();
        for (x, y) in [(0, 0), (15, 15), (16, 16), (17, 40), (40, 17), (100, 100)] {
            let entity = world.spawn_empty().id();
            storage.set(TilePos::new(x, y), entity);
            tiles.push((TilePos::new(x, y), entity));
        }
        let expected = |min: TilePos, max: TilePos| {
            let mut expected = tiles
                .iter()
                .copied()
                .filter(|(p, _)| p.x >= min.x && p.x <= max.x && p.y >= min.y && p.y <= max.y)
                .collect::<Vec<_>>();
            expected.sort();
            expected
        };
        for (min, max) in [
            (TilePos::new(0, 0), TilePos::new(15, 15)),
            (TilePos::new(15, 15), TilePos::new(16, 16)),
            (TilePos::new(1, 1), TilePos::new(14, 14)),
            (TilePos::new(16, 0), TilePos::new(47, 47)),
            (TilePos::new(0, 0), TilePos::new(u32::MAX, u32::MAX)),
            (TilePos::new(5, 5), TilePos::new(4, 4)),
        ] {
            assert_eq!(sorted(storage.iter_in(min, max)), expected(min, max));
        }
        assert_eq!(
            expected(TilePos::new(15, 15), TilePos::new(16, 16)).len(),
            2
        );
    }
}
