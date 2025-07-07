# Spawn / Despawn / Reload a Map

These aspects are also covered in the [dedicated example](https://github.com/adrien-bon/bevy_ecs_tiled/blob/main/examples/map_reload.rs).

---

## Spawning a Map

Spawning a map is a two-step process:

1. **Load the map asset** using Bevy's `AssetServer`.
2. **Spawn a `TiledMap`** containing a reference to this map asset.

```rust,no_run
fn spawn_map(
    mut commands: Commands,
    asset_server: Res<AssetServer>
) {
    // Load the map asset
    let map_handle: Handle<TiledMapAsset> = asset_server.load("map.tmx");

    // Spawn the map entity with default settings
    commands.spawn(TiledMap(map_handle));
}
```

> **Tip:**  
> You can load maps at startup or at any time during your game.  
> There is no restriction on the number of maps loaded or spawned simultaneously.

---

## Despawning a Map

To despawn a map, simply remove its top-level entity.  
All child entities (layers, tiles, objects, etc.) will be automatically despawned.

```rust,no_run
pub fn despawn_map(
    mut commands: Commands,
    map_query: Query<Entity, With<TiledMap>>,
) {
    // Iterate over all map entities
    for map in map_query.iter() {
        // Despawn the map entity and all its children
        commands.entity(map).despawn();
    }
}
```

---

## Respawning / Reloading a Map

If you want to reload or respawn a map, you can despawn it and spawn it again as shown above.  
However, there is a more ergonomic way: **insert the `RespawnTiledMap` component** on the map entity.

```rust,no_run
fn respawn_map(
    mut commands: Commands,
    map_query: Query<Entity, With<TiledMap>>,
) {
    if let Ok(entity) = map_query.single() {
        commands.entity(entity).insert(RespawnTiledMap);
    }
}
```

This will reload the same map asset, despawning and respawning all child entities (layers, tiles, objects, etc.), while preserving the top-level map entity and its components.

- Any changes you made to child entities (e.g., tile color, object position) will be reset to their original state.
- Components on the map entity itself (such as `TiledMapSettings`, `TilemapRenderSettings`, or `Transform`) will be **preserved**.

This is useful for implementing level respawn or resetting a map to its initial state.

---

## Loading a New Map Over an Existing One

To replace the current map with a different one, simply insert a new `TiledMap` on the existing map entity:

```rust,no_run
fn handle_reload(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    map_query: Query<Entity, With<TiledMap>>,
) {
    if let Ok(entity) = map_query.single() {
        commands.entity(entity)
            .insert(
                TiledMap(asset_server.load("other_map.tmx"))
            );
    }
}
```

- If you insert the same map handle, it is equivalent to inserting the `RespawnTiledMap` component.

---

## Summary

- **Spawn**: Load the map asset and spawn a `TiledMap`.
- **Despawn**: Remove the map entity (children are cleaned up automatically).
- **Respawn/Reload**: Insert `RespawnTiledMap` or a new `TiledMap` to reload or swap maps efficiently.

For more advanced usage, see the [examples](https://github.com/adrien-bon/bevy_ecs_tiled/tree/main/examples/map_reload.rs).
