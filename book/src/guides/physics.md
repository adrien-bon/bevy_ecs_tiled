# Add physics colliders

Tiled allows you to add objects to your map: either directly on an object layer or attached to a tile.
`bevy_ecs_tiled` can use these objects to automatically spawn physics colliders when loading the map.

We provide two working physics backends: one using [`avian`](https://github.com/Jondolf/avian) and another one using [`rapier`](https://github.com/dimforge/bevy_rapier).
They are very close in terms of features regarding `bevy_ecs_tiled` so feel free to choose the one you prefer.

The API allows you to either use these backends "as-is", customize them or implement your own backend.

## Create collider objects in Tiled

First step is to have on your map something that we can actually use to spawn a collider :

- objects attached to an object layer: these are the regular objects you can place on the map when working on an object layer.
- objects attached to a tile: for these, you will need to actually edit your tileset and use Tiled [builtin collision editor](https://doc.mapeditor.org/en/stable/manual/editing-tilesets/#tile-collision-editor).

These two kinds of objects, eventhough they are handled a bit differently, will actually produce the same result: using the object associated shape we will spawn the corresponding physic collider.

## Automatically spawn colliders

In order to automatically spawn colliders from Tiled objects, you need two things:

- enable the proper feature: either `avian` or `rapier`, depending on your backend choice. You can eventually only enable the `physics` feature, but you'll have to provide your own physics backend.
- instanciate the [`TiledPhysicsPlugin`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/physics/struct.TiledPhysicsPlugin.html) with the associated [`TiledPhysicsBackend`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/physics/trait.TiledPhysicsBackend.html) of your choice.

For instance, to automatically add colliders using the `Avian` backend:

```rust,no_run
use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;

// You must enable the 'avian' feature from bevy_ecs_tiled

fn main() {
    App::new()
        // Load bevy_ecs_tiled main plugin
        .add_plugins(TiledMapPlugin::default())
        // Load bevy_ecs_tiled physics plugin: this is where we select
        // which physics backend to use (in this case, Avian)
        .add_plugins(TiledPhysicsPlugin::<TiledPhysicsAvianBackend>::default())
        // Load Avian main plugin
        .add_plugins(PhysicsPlugins::default().with_length_unit(100.0))
        // Add our systems and run the app!
        .add_systems(Startup, startup)
        .run();
}

// Just load the map as usual:
// colliders are spawned for every objects of your map
fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);
    commands.spawn(TiledMapHandle(asset_server.load("finite.tmx")));
}
```

## Objects filtering

By default, we will spawn physics colliders for **all** objects encountered in the map, either regular objects or tiles colliders.
Eventhough it's probably the most common behaviour, you can also fine tune for which objects you want to spawn colliders for.

To do so, we provide the [`TiledPhysicsSettings`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/physics/struct.TiledPhysicsSettings.html) component.

This component, which should be added to the map or to the world entity, contains several filters.
Based upon its name, and its parent layer name, we can determine if a particular object should have collider or not.

```rust,no_run
use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;

// Load the map with custom physics settings (and an Avian backend)
fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);
    commands.spawn((
        TiledMapHandle(asset_server.load("finite.tmx")),
        // With this configuration, we will restrict the spawn of colliders to:
        // - regular objects named 'hitbox' or 'collision'
        // - collision objects for tiles in a layer named 'collision'
        // We will not spawn colliders for objects not matching these conditions
        TiledPhysicsSettings::<TiledPhysicsAvianBackend> {
            objects_filter: ObjectNames::Names(vec![String::from("hitbox"), String::from("collision")]),
            tiles_layer_filter: ObjectNames::Names(vec![String::from("collision")]),
            ..default()
        },
    ));
}
```

## Colliders aggregation

Tiled uses simple shapes, such as rectangles or ellipses, to define colliders :

- For tiles, you can use several of these simple shapes to define the whole tile collider
- For regular objects (ie. objects inside an object layer), you can either use one of these simple shapes or colliders from an existing tile.

In order to reduce the number of physics colliders, and improve overall performances, we will try to merge "simple" colliders together and spawn instead a single "complex" one.
However, keep in mind that if you use complex geometric forms (polygons) for your collisions, we won't be able to merge these colliders and you will still have multiples  colliders, potentially hurting performances.

For a given tiles layer we could have a single physics collider which consists of the aggregation of all tiles colliders it contains.
From a physics point of view we actually have a single collider per layer.

On the other hand, regular objects will always have their own physics collider.
Since they have a `Transform` and can be moved independently, it would not make sense to have a single colliders for several objects.
In case of "tile objects", we will still try to merge colliders from the tile.

## Custom physics backend

For more advanced use cases, you can eventually define your own physics backend.
All you need to do is to create a struct that implements the [`TiledPhysicsBackend`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/physics/trait.TiledPhysicsBackend.html) trait, ie. provide an implementation for the `spawn_colliders` function and use that backend when adding the `TiledPhysicsPlugin<T>` to your app.

You can also extend one the provided backend if their implementation have something missing for you.
You can wrap your own implementation around and existing backend (see [this example for Avian](https://github.com/adrien-bon/bevy_ecs_tiled/blob/main/examples/physics_avian_controller.rs) or [this one for Rapier](https://github.com/adrien-bon/bevy_ecs_tiled/blob/main/examples/physics_rapier$_controller.rs)).
