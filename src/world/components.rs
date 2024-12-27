use crate::prelude::*;
use bevy::prelude::*;

#[derive(Component, Default)]
pub struct TiledWorldSettings {
    pub chunking: Option<(u32, u32)>,
}

/// Marker [Component] for a Tiled world.
#[derive(Component)]
pub struct TiledWorldMarker;

#[derive(Component)]
pub struct RespawnTiledWorld;

#[derive(Component, Default)]
pub struct TiledWorldStorage {
    pub maps: Vec<TiledWorldMapStorage>,
    pub spawned_maps: Vec<(Entity, Rect)>,
}

pub struct TiledWorldMapStorage {
    pub asset: Handle<TiledMap>,
    pub entity: Option<Entity>,
    pub x: i32,
    pub y: i32,
    pub height: u32,
    pub width: u32,
}
