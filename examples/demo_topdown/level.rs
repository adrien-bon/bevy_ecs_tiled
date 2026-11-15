//! Load the Tiled maps and move the player from one map to another.
//!
//! There is a single map entity: going through a [`Door`] replaces its map asset, so the map
//! we leave is despawned and the new one is spawned. As a consequence, what the player did
//! on the map we leave is lost: crates are back when we come again.

use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;

use crate::{
    UpdateSystems,
    picking::TileGeometry,
    player::{GridMover, GridPosition, Player, PlayerAssets},
};

/// The map we start on, and the name of the [`SpawnPoint`] the player starts from.
const FIRST_MAP: &str = "demo_topdown/outdoor.tmx";
const FIRST_SPAWN_POINT: &str = "start";

pub(super) fn plugin(app: &mut App) {
    app.register_type::<SpawnPoint>();
    app.register_type::<Door>();
    app.add_systems(Startup, startup);
    app.add_systems(Update, go_through_door.in_set(UpdateSystems::Interact));
    // Wait for transforms to be propagated, so we get the actual position of our new map
    app.add_systems(
        PostUpdate,
        place_player
            .after(TransformSystems::Propagate)
            .run_if(resource_exists::<PendingSpawn>),
    );
}

/// A named location where the player can arrive on a map.
///
/// It is added from Tiled, on a point object.
#[derive(Component, Debug, Clone, PartialEq, Eq, Default, Reflect)]
#[reflect(Component, Default)]
pub struct SpawnPoint {
    pub name: String,
}

/// Sends the player to another map when it walks on the tile below this entity.
///
/// It is added from Tiled, on a point object.
#[derive(Component, Debug, Clone, PartialEq, Eq, Default, Reflect)]
#[reflect(Component, Default)]
pub struct Door {
    /// Path of the map to load, relative to the assets directory.
    pub target_map: String,
    /// Name of the [`SpawnPoint`] of the target map where the player arrives.
    pub target_spawn: String,
}

/// Name of the [`SpawnPoint`] where to put the player once the map is ready.
#[derive(Resource, Debug)]
struct PendingSpawn(String);

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        TiledMap(asset_server.load(FIRST_MAP)),
        TilemapAnchor::Center,
    ));
    commands.insert_resource(PendingSpawn(FIRST_SPAWN_POINT.to_string()));
}

/// Load another map when the player walks on a [`Door`].
fn go_through_door(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    geometry: TileGeometry,
    map_single: Single<Entity, With<TiledMap>>,
    player_single: Single<(Ref<GridPosition>, &mut GridMover, &mut Visibility), With<Player>>,
    doors_query: Query<(&Door, &GlobalTransform)>,
) {
    let (player_position, mut mover, mut visibility) = player_single.into_inner();
    if !player_position.is_changed() {
        return;
    }
    let Some((door, _)) = doors_query.iter().find(|(_, transform)| {
        geometry.tile_at(transform.translation().truncate()) == Some(player_position.0)
    }) else {
        return;
    };

    info!("Go to map '{}'", door.target_map);
    // Replace the map asset: the current map is despawned and the new one is spawned instead
    commands
        .entity(*map_single)
        .insert(TiledMap(asset_server.load(&door.target_map)));
    commands.insert_resource(PendingSpawn(door.target_spawn.clone()));

    // The player does not belong to the map: hide it until the new map is ready
    mover.stop();
    *visibility = Visibility::Hidden;
}

/// Put the player on its [`SpawnPoint`], once it has been spawned.
fn place_player(
    mut commands: Commands,
    pending_spawn: Res<PendingSpawn>,
    geometry: TileGeometry,
    player_assets: Res<PlayerAssets>,
    spawn_points_query: Query<(&SpawnPoint, &GlobalTransform)>,
    mut player_query: Query<(&mut GridPosition, &mut Transform, &mut Visibility), With<Player>>,
    mut camera_query: Query<&mut Transform, (With<Camera2d>, Without<Player>)>,
) {
    // Wait for the spawn point to be there, along with the tilemaps we use to locate tiles
    let Some((tile_position, world_position)) = spawn_points_query
        .iter()
        .find(|(spawn_point, _)| spawn_point.name == pending_spawn.0)
        .and_then(|(_, transform)| geometry.tile_at(transform.translation().truncate()))
        .and_then(|tile_position| Some((tile_position, geometry.tile_center(tile_position)?)))
    else {
        return;
    };

    if let Ok((mut position, mut transform, mut visibility)) = player_query.single_mut() {
        position.0 = tile_position;
        transform.translation = world_position.extend(transform.translation.z);
        *visibility = Visibility::Inherited;
    } else {
        commands.spawn(player_assets.player(tile_position, world_position));
    }

    // Do not let the camera travel from where the player was on the previous map
    for mut camera_transform in &mut camera_query {
        camera_transform.translation = world_position.extend(camera_transform.translation.z);
    }
    commands.remove_resource::<PendingSpawn>();
}
