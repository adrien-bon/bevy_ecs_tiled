# Add Physics Colliders

Tiled allows you to add objects to your map—either directly on an object layer or attached to a tile.  
`bevy_ecs_tiled` can use these objects to automatically spawn physics colliders when loading the map.

Two physics backends are supported out of the box:  

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

1. **Enable the appropriate feature:**  
   Either `avian` or `rapier`, depending on your backend choice.  
   (You can also enable only the `physics` feature and provide your own backend.)

2. **Add the [`TiledPhysicsPlugin`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/physics/struct.TiledPhysicsPlugin.html)**  
   with the backend of your choice.

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

Colliders will be spawned for every object in your map automatically.

> **Note:**  
> You can actually use several physics backends at the same time if you register the `TiledPhysicsPlugin<T>` plugin multiple times.

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
            objects_filter: ObjectNames::Names(vec!["hitbox".into(), "collision".into()]),
            tiles_layer_filter: ObjectNames::Names(vec!["collision".into()]),
            ..default()
        },
    ));
}
```

---

## Colliders Aggregation

Tiled uses simple shapes (rectangles, ellipses, polygons) to define colliders.  
To reduce the number of physics colliders and improve performance, `bevy_ecs_tiled` tries to merge these simple shape colliders into a single complex one whenever possible.

- **Tiles:** Merging is actually performed at the tilemap level.  
  Eventhough you can use several shapes to define a tile's collider, these shape will be merged into a single collider and all tile colliders for a given tilemap will also be merged together.  
  The collider entity is spawned at the tilemap level.  
- **Regular objects:** If an object references a tile, it will inherit that tile's collider and if the tile has several shapes, they will be merged.  
  The collider entity is spawned at the object level and each object always gets its own collider.

> **Note:**  
> Complex polygons cannot always be merged, so you may still end up with multiple colliders per tile, which can affect performance.

---

## Custom Physics Backends

For advanced use cases, you can implement your own physics backend.  
Just create a struct that implements the [`TiledPhysicsBackend`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/physics/backend/trait.TiledPhysicsBackend.html) trait (i.e., provide a `spawn_colliders` function), and use it when registering the `TiledPhysicsPlugin<T>` plugin.

See [this example](https://github.com/adrien-bon/bevy_ecs_tiled/blob/main/examples/physics_custom_backend.rs) for more information.

---

## Physics Events

When a collider entity is spawned, a [`TiledEvent<ColliderCreated>`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/physics/collider/struct.ColliderCreated.html) event is sent.
It contains information about the origin of the collider.

This can be useful to add custom components to your colliders.  
For instance, with Avian:

```rust,no_run
use bevy::prelude::*;
use avian2d::prelude::*;
use bevy_ecs_tiled::prelude::*;

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);
    commands.spawn(
        TiledMap(asset_server.load("map.tmx"))
            .observe(
                |trigger: Trigger<TiledEvent<ColliderCreated>>, mut commands: Commands| {
                    // Automatically add a RigidBody::Static component to colliders created from Tiled objects
                    if trigger.event().event != TiledCollider::TilesLayer {
                        return;
                    }
                    // Add a RigidBody::Static component to the collider entity
                    commands
                        .entity(trigger.event().target)
                        .insert(RigidBody::Static);
                },
            ),
    );
}
```
