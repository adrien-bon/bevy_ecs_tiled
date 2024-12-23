# Add physics colliders

Tiled allows you to add objects to your map: either directly on an object layer or attached to a tile.
`bevy_ecs_tiled` can use these objects to automatically spawn physics colliders when loading the map.

We provide two working physics backend: one using [`avian`](https://github.com/Jondolf/avian) and another one using [`bevy_rapier`](https://github.com/dimforge/bevy_rapier).
They are the same in terms of features regarding `bevy_ecs_tiled` so feel free to choose the one you prefer.

The API also allows you to provide a custom physics backend or to customize existing ones.

## Create objects in Tiled

First step is to have on your map something that we can actually use to spawn a collider: objects.

In Tiled, there are two kinds of objects:

- objects attached to an object layer: these are the regular objects you can place on the map when working on an object layer.
- objects attached to a tile: for these, you will need to actually [edit your tileset](https://doc.mapeditor.org/en/stable/manual/editing-tilesets/#tile-collision-editor).

These two kinds of objects, eventhough they are handled a bit differently, will actually produce the same result: using the object shape we will be able to spawn a collider.
So, as you can imagine, a "Point" object will not produce anything physics-wise :)

One important aspect is to remember that you change the name of your objects.

## Automatically spawn colliders

In order to automatically spawn colliders from Tiled objects, you need two things:

- having the proper feature enabled: you will either need `avian` or `rapier`, depending on your backend choice (you can eventually only enable `physics`, but you it means you will provide the backend yourself).
- instanciate the [`TiledPhysicsPlugin`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/physics/struct.TiledPhysicsPlugin.html) with an associated [`TiledPhysicsBackend`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/physics/trait.TiledPhysicsBackend.html).

```rust,no_run
use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;

// This is a minimal example: in a real world scenario, you would probably
// need to load additionnal plugins (TiledMapPlugin and TilemapPlugin for instance)

fn main() {
    App::new()
        // bevy_ecs_tiled physics plugin: this is where we select
        // which physics backend to use (in this case, Avian)
        .add_plugins(TiledPhysicsPlugin::<TiledPhysicsAvianBackend>::default())
        // Add our systems and run the app!
        .add_systems(Startup, startup)
        .run();
}

// Just load the map as usual
fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);
    commands.spawn(TiledMapHandle(asset_server.load("finite.tmx")));
}
```

By default, we will spawn physics colliders for **all** objects encountered in the map.
Eventhough it's probably the most common behaviour, you can also fine tune for which objects you want to spawn colliders for by updating the [`TiledPhysicsSettings`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/physics/struct.TiledPhysicsSettings.html) component.

```rust,no_run
use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;

// Load the map with custom physics settings (and an Avian backend)
fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);
    commands.spawn((
        TiledMapHandle(asset_server.load("finite.tmx")),
        // With this configuration, we will restrict the spawn of collider
        // for tiles in a layer named 'collision' and prevent spawning colliders for Tiled objects.
        TiledPhysicsSettings::<TiledPhysicsAvianBackend> {
            objects_layer_filter: ObjectNames::None,
            tiles_layer_filter: ObjectNames::Names(vec!["collision".to_string()]),
            ..default()
        },
    ));
}
```

## Custom physics backend and colliders event

If you need to, the API will let you to add your own physics behaviour.

You can eventually define your own physics backend.
All you need to do is to create a struct that implements the [`TiledPhysicsBackend`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/physics/trait.TiledPhysicsBackend.html) trait, ie. provide an implementation for the `spawn_collider` function.

You can even use one the provided backend, but if their implementation have something missing for you, you can wrap your own implementation around and existing backend (see [this example for Avian](https://github.com/adrien-bon/bevy_ecs_tiled/blob/main/examples/physics_avian_controller.rs) or [this one for Rapier](https://github.com/adrien-bon/bevy_ecs_tiled/blob/main/examples/physics_rapier$_controller.rs)).

Finally, whatever the backend you are using, a dedicated [`TiledColliderCreated`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/physics/collider/struct.TiledColliderCreated.html) event will be fired after a collider is spawned.
Note that you will have one event per spawned collider.
These events can be used for instance to add a missing component to the collider (or anything you want).

## Special considerations

For Tiled objects (ie. objects attached to an object layer), we will spawn one collider per object.

For objects attached to tiles, we will try to merge all colliders from a given layer together so from a physics point of view, we actually have a single collider.
The idea is to improve performances by reducing the number of entities we spawn.
However, keep in mind that if you use complex geometric forms for your collisions, we won't be able to merge these colliders and you will end up with a lot of entities which can impact performances.
