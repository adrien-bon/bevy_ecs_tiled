# Add Physics Colliders

Tiled allows you to add objects to your map—either directly on an object layer or attached to a tile.
`bevy_ecs_tiled` can use these objects to automatically spawn physics colliders when loading the map.

Two physics engines are supported out of the box:

- [`avian`](https://github.com/Jondolf/avian)
- [`rapier`](https://github.com/dimforge/bevy_rapier)

Both are well-integrated and offer similar features in the context of `bevy_ecs_tiled`. You can use them as-is, customize their behavior, or implement your own backend.

---

## Creating Collider Objects in Tiled

To spawn colliders, you need to define objects in your map:

- **Objects on an object layer:**
  These are the standard objects you place in Tiled on an object layer.
- **Objects attached to a tile:**
  Edit your tileset and use Tiled's [built-in collision editor](https://doc.mapeditor.org/en/stable/manual/editing-tilesets/#tile-collision-editor) to define collision shapes for tiles.

Both types of objects are supported and will result in physics colliders being spawned in your game, based on their shapes.

---

## Automatically Spawning Colliders

To automatically spawn colliders from Tiled objects, you need to:

1. **Enable the appropriate feature in your `Cargo.toml`:**
   Either `avian` or `rapier`, depending on your backend choice.
   You can also enable only the `physics` feature and provide your own backend (see below).

**Example: Using the Avian backend**

```toml
[dependencies]
bevy = "0.20"
bevy_ecs_tiled = { version = "0.15", features = ["avian"] }
```

> **Note:**
> You may need to adjust `bevy` and `bevy_ecs_tiled` versions.

2. **Add the [`TiledPhysicsPlugin<T>`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/physics/struct.TiledPhysicsPlugin.html)**
   with the [`TiledPhysicsBackend`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/physics/backend/trait.TiledPhysicsBackend.html) of your choice.

**Example: Using the Avian backend**

```rust,no_run
use bevy::prelude::*;
use avian2d::prelude::*;
use bevy_ecs_tiled::prelude::*;

// You must enable the 'avian' feature for bevy_ecs_tiled

fn main() {
    App::new()
        // Load bevy_ecs_tiled main plugin
        .add_plugins(TiledPlugin::default())
        // Load bevy_ecs_tiled physics plugin (select Avian backend)
        .add_plugins(TiledPhysicsPlugin::<TiledPhysicsAvianBackend>::default())
        // Load Avian main plugin
        .add_plugins(PhysicsPlugins::default().with_length_unit(100.0))
        // Add your systems and run the app!
        .add_systems(Startup, startup)
        .run();
}

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);
    commands.spawn(TiledMap(asset_server.load("map.tmx")));
}
```

Colliders will be spawned for every object or tiles colliders in your map automatically.

Colliders are derived from the map entities: if you add, modify or remove a tile or an object after the map has been loaded, the corresponding colliders are automatically spawned again.

We provide two [`TiledPhysicsBackend`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/physics/backend/trait.TiledPhysicsBackend.html) that can be used out of the box:

- [`TiledPhysicsAvianBackend`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/physics/backend/avian/struct.TiledPhysicsAvianBackend.html): for the Avian 2D physics engine
- [`TiledPhysicsRapierBackend`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/physics/backend/rapier/struct.TiledPhysicsRapierBackend.html): for the Rapier 2D physics engine

> **Note:**
> You can actually use several physics backends at the same time if you register the [`TiledPhysicsPlugin<T>`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/physics/struct.TiledPhysicsPlugin.html) plugin multiple times, but with different [`TiledPhysicsBackend`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/physics/backend/trait.TiledPhysicsBackend.html).

---

## Filtering Which Objects Get Colliders

By default, colliders are spawned for **all** objects (regular objects and tile colliders).
If you want more control, use the [`TiledPhysicsSettings`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/physics/settings/struct.TiledPhysicsSettings.html) component.

This component lets you filter which objects/layers should have colliders, based on their names.

**Example: Only spawn colliders for certain objects/layers**

```rust,no_run
use bevy::prelude::*;
use avian2d::prelude::*;
use bevy_ecs_tiled::prelude::*;

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);
    commands.spawn((
        TiledMap(asset_server.load("finite.tmx")),
        // Restrict colliders to:
        // - objects named 'hitbox' or 'collision'
        // - tile colliders in a layer named 'collision'
        TiledPhysicsSettings::<TiledPhysicsAvianBackend> {
            objects_filter: TiledFilter::Names(vec!["hitbox".into(), "collision".into()]),
            tiles_layer_filter: TiledFilter::Names(vec!["collision".into()]),
            ..default()
        },
    ));
}
```

---

## Colliders Aggregation

Tiled uses simple shapes (rectangles, ellipses, polygons) to define colliders.
To reduce the number of physics colliders and improve performance, `bevy_ecs_tiled` tries to merge these simple shape colliders into a single complex one whenever possible.

- **Tiles:** Merging is actually performed at the tiles layer level.
  Eventhough you can use several shapes to define a tile's collider, these shape will be merged into a single collider and all tile colliders for a given tiles layer will also be merged together.
  The collider is inserted on the tiles layer entity, and spawned again whenever a tile of this layer is added, modified (including its `TileFlip`) or removed.
  Colliders of large layers can also be [split in chunks](#colliders-chunks).
  Tiles flips (horizontal, vertical and diagonal) are applied to their collider, the same way they are applied to their rendering.
- **Regular objects:** If an object references a tile, it will inherit that tile's collider and if the tile has several shapes, they will be merged.
  The collider is inserted on the object entity and each object always gets its own collider.

The first collider of a tiles layer or an object is inserted on the layer or object entity itself, so you can directly add other physics components to it.
If there are several colliders (see below), the next ones are spawned as children of this entity.

---

## Collider Strategy

The [`TiledColliderStrategy`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/physics/collider/enum.TiledColliderStrategy.html) selects how physics colliders are created from Tiled shapes:

- `Polyline` (default): a single collider made of all the segments outlining the shapes. Bodies can end up inside of the shape.
- `Triangulation`: shapes are triangulated and all triangles are aggregated in a single "solid" compound collider.
- `LineStrip`: one collider for each ring outlining the shapes, so there can be several colliders.

The strategy is set for the whole map, in [`TiledPhysicsSettings`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/physics/settings/struct.TiledPhysicsSettings.html):

```rust,no_run
use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        TiledMap(asset_server.load("map.tmx")),
        TiledPhysicsSettings::<TiledPhysicsAvianBackend> {
            collider_strategy: TiledColliderStrategy::Triangulation,
            ..default()
        },
    ));
}
```

It can be overridden for a given layer or object: `TiledColliderStrategy` is also a component, which you can insert on a layer or an object entity.
An object uses its own strategy if it has one, otherwise the one of its layer, otherwise the one of the map.

```rust,no_run
use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;

// Use line strips for the tiles layer named "walls"
fn on_layer_created(
    layer_created: On<TiledEvent<LayerCreated>>,
    layers_query: Query<&TiledName, With<TiledLayer>>,
    mut commands: Commands,
) {
    let layer = layer_created.origin;
    if layers_query.get(layer).is_ok_and(|name| name.0 == "walls") {
        commands.entity(layer).insert(TiledColliderStrategy::LineStrip);
    }
}
```

Colliders are spawned again when this component is inserted, modified or removed.
With the `user_properties` feature, you can also directly add it from Tiled as a [custom property](properties.md) of the layer or object.

---

## Colliders Chunks

By default, all the tiles of a layer are merged in a single set of colliders.
This is the best option for maps which are not modified once loaded, but it means that all the colliders of the layer are computed again whenever one of its tiles is added, modified or removed, which can take a while on large maps.

If you [modify your maps at runtime](dynamic_maps.md), you can split the colliders of tiles layers in chunks: each chunk of tiles gets its own colliders and only the chunks whose tiles are modified are computed again.

The chunk size, in tiles, is set for all the tiles layers of the map in [`TiledPhysicsSettings`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/physics/settings/struct.TiledPhysicsSettings.html):

```rust,no_run
use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        TiledMap(asset_server.load("map.tmx")),
        TiledPhysicsSettings::<TiledPhysicsAvianBackend> {
            // Split tiles layers colliders in chunks of 32x32 tiles
            tiles_chunk_size: TiledColliderChunkSize::Size(UVec2::splat(32)),
            ..default()
        },
    ));
}
```

It can be overridden for a given tiles layer by inserting a [`TiledColliderChunkSize`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/physics/collider/enum.TiledColliderChunkSize.html) component on the layer entity, either to use another size or to disable chunking for this layer (`TiledColliderChunkSize::Disabled`).
You can also only use this component, to enable chunking for a single layer of the map:

```rust,no_run
use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;

fn on_layer_created(
    layer_created: On<TiledEvent<LayerCreated>>,
    layers_query: Query<&TiledName, With<TiledLayer>>,
    mut commands: Commands,
) {
    let layer = layer_created.origin;
    if layers_query.get(layer).is_ok_and(|name| name.0 == "destructible") {
        commands
            .entity(layer)
            .insert(TiledColliderChunkSize::Size(UVec2::splat(16)));
    }
}
```

When chunking is enabled for a layer:

- the layer entity does not hold any collider. Each chunk with at least one collider has its own [`TiledColliderChunk`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/physics/collider/struct.TiledColliderChunk.html) entity, child of the layer, holding the collider of the chunk;
- colliders are not merged across chunks: two adjacent tiles which are not in the same chunk have separate colliders. Depending on your physics engine and collider strategy, bodies sliding over a chunk boundary can catch on it ("ghost collisions"): prefer larger chunks if this is an issue;
- smaller chunks make updates faster, but produce more colliders.

With the `debug` feature, the [`TiledDebugColliderChunksPlugin`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/debug/collider_chunks/index.html) displays the bounding box of each chunk colliders.

---

## Custom Physics Backends

For advanced use cases, you can implement your own physics backend.
Just create a struct that implements the [`TiledPhysicsBackend`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/physics/backend/trait.TiledPhysicsBackend.html) trait, and use it when registering the `TiledPhysicsPlugin<T>` plugin.

A backend only has to create colliders out of a geometry: it declares the component(s) making a collider (the `Collider` associated type) and provides a `colliders()` function returning them for a given geometry and [`TiledColliderStrategy`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/physics/collider/enum.TiledColliderStrategy.html).
`bevy_ecs_tiled` takes care of inserting these colliders on the right entities and of removing them when their source changes.
The [`collider_shapes()`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/physics/geometry/fn.collider_shapes.html) helper converts a geometry into simple shapes (polylines or triangles) according to the requested strategy.

See [this example](https://github.com/adrien-bon/bevy_ecs_tiled/blob/main/examples/physics_custom.rs) for more information.

---

## Physics Events

When a collider entity is spawned, a [`TiledEvent<ColliderCreated>`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/physics/collider/struct.ColliderCreated.html) event is sent.
Since colliders are spawned again when their source changes, you can receive this event several times for the same tiles layer or object.
It contains information about the origin of the collider.

This can be useful to add custom components to your colliders.
For instance, with Avian:

```rust,no_run
use bevy::prelude::*;
use avian2d::prelude::*;
use bevy_ecs_tiled::prelude::*;

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);
    commands
        .spawn(TiledMap(asset_server.load("map.tmx")))
        .observe(|collider_created: On<TiledEvent<ColliderCreated>>, mut commands: Commands| {
            // Filter collider created from Tiled objects
            if collider_created.event().event.source == TiledColliderSource::Object {
                // Add a RigidBody::Static to the collider entity
                commands
                    .entity(collider_created.origin)
                    .insert(RigidBody::Static);
            }
        });
}
```
