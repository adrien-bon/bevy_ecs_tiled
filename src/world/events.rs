use crate::prelude::*;
use bevy::prelude::*;

/// Event sent when a Tiled world has finished loading
#[derive(Event, Clone, Debug)]
pub struct TiledWorldCreated {
    pub world: Entity,
    pub world_handle: Handle<TiledWorld>,
}

impl<'a> TiledWorldCreated {
    pub fn world(&self, world_asset: &'a Res<Assets<TiledWorld>>) -> &'a TiledWorld {
        &world_asset.get(self.world_handle.id()).unwrap()
    }
}