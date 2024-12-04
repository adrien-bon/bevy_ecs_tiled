
use crate::prelude::*;
use bevy::prelude::*;

#[derive(Component, Default)]
pub struct TiledWorldSettings {
    // XXX: here we could have world related settings, such as culling
}

/// Marker [Component] for a Tiled world.
#[derive(Component)]
pub struct TiledWorldMarker;

#[derive(Component)]
pub struct RespawnTiledWorld;

#[derive(Component, Default)]
pub struct TiledWorldStorage(pub Vec<TiledWorldMapStorage>);

pub struct TiledWorldMapStorage {
    pub asset: Handle<TiledMap>,
    pub entity: Option<Entity>,
    // XXX: add informations about where this particular map is postionned
}