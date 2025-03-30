# Spawn / Despawn / Reload a map

These aspects are also covered in the [dedicated example](https://github.com/adrien-bon/bevy_ecs_tiled/blob/main/examples/map_reload.rs).

## Spawn a map

Spawning a map is done in two steps:

- first, load a map asset / a map file using the Bevy `AssetServer`
- then, spawn a `TiledMapHandle` containing a reference to this map asset

```rust,no_run
fn spawn_map(
    mut commands: Commands,
    asset_server: Res<AssetServer>
) {
    // First, load the map
    let map_handle: Handle<TiledMap> = asset_server.load("map.tmx");

    // Then, spawn it, using default settings
    commands.spawn(TiledMapHandle(map_handle));
}
```

Note that you can perform the initial map loading beforehand (for instance, during your game startup) and  that there is no restriction on the number of maps loaded or spawned at the same time.

## Despawn a map

If you want to despawn a map, the easiest is to actually remove its top-level entity:

```rust,no_run
pub fn despawn_map(
    mut commands: Commands,
    maps_query: Query<Entity, With<TiledMapMarker>>,
) {
    // Iterate over entities with a TiledMapMarker component
    for map in q_maps.iter() {
        // Despawn these entities, as well as their child entities
        commands.entity(map).despawn();
    }
}
```

All child entities, like layers and tiles, will automatically be despawned.

## Respawn / reload a map

If you want to reload or respawn a map, you can of course despawn it then spawn it again.
It's tedious, but it works.

However, there is an easier way: you can insert the `RespawnTiledMap` component to the map entity:

```rust,no_run
fn respawn_map(
    mut commands: Commands,
    maps_query: Query<Entity, With<TiledMapMarker>>,
) {
    if let Ok(entity) =  maps_query.get_single() {
        commands.entity(entity).insert(RespawnTiledMap);
    }
}
```

This will load the exact same map but will despawn / spawn again new entities for layers, tiles and objects.
Only the top-level map entity and its components will be persisted.

It means that if you updated components on the children entities (for instance, a tile color or an object position) they will be back as they were when you first loaded the map.
But map components such as `TiledMapSettings`, `TilemapRenderSettings` or `Transform` will be persisted through a respawn.

It could be used to implement a level respawn for instance.

Another use case is to load a new map over an existing one.
An easy way to do that is to just spawn a new `TiledMapHandle` over an existing map:

```rust,no_run
fn handle_reload(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    maps_query: Query<Entity, With<TiledMapMarker>>,
) {
    if let Ok(entity) = maps_query.get_single() {
        commands.entity(entity)
            .insert(
                TiledMapHandle(asset_server.load("other_map.tmx"))
            );
    }
}
```

Note that loading the same map `Handle` is equivalent to insert the `RespawnTiledMap` component.
