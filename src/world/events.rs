use crate::prelude::*;
use bevy::prelude::*;

/// Event sent when a Tiled world has finished loading
#[derive(Event, Clone, Debug)]
pub struct TiledWorldCreated {
    pub world: Entity,
    pub world_asset_id: AssetId<TiledWorld>,
}

impl<'a> TiledWorldCreated {
    pub fn get_world(&self, world_asset: &'a Res<Assets<TiledWorld>>) -> Option<&'a TiledWorld> {
        world_asset.get(self.world_asset_id)
    }
}
