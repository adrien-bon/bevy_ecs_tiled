//! Events related to Tiled world loading
//!
//! These events will be fired after the whole map has loaded.

use crate::prelude::*;
use bevy::prelude::*;

/// Event sent when a Tiled world has finished loading.
#[derive(Event, Clone, Debug)]
pub struct TiledWorldCreated {
    /// Spawned world [Entity].
    pub world: Entity,
    /// [AssetId] of the corresponding [super::asset::TiledWorld] asset.
    pub world_asset_id: AssetId<TiledWorld>,
}

impl<'a> TiledWorldCreated {
    /// Retrieve the [TiledWorld] associated to this [TiledWorldCreated] event.
    pub fn get_world(&self, world_asset: &'a Res<Assets<TiledWorld>>) -> Option<&'a TiledWorld> {
        world_asset.get(self.world_asset_id)
    }
}
