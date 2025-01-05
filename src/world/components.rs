use bevy::{prelude::*, utils::HashMap};

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
    pub spawned_maps: HashMap<usize, Entity>,
}
