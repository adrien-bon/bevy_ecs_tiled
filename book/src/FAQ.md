# FAQ

## What's the current status of the crate ?

While the crate is already definitely usable, it is still under active development.

Expect bugs, missing features and breaking changes !

However, I plan to follow semver and to provide a migration guide each time there are breaking changes (for instance, upon a new Bevy release).

## What kind of maps are supported ?

We should support nearly all maps from Tiled :

- orthogonal
- "flat-top" hexagonal
- "pointy-top" hexagonal
- "diamond" isometric

There is however an exception: we do not (and don't plan to) support isometric "staggered" maps ([#31](https://github.com/adrien-bon/bevy_ecs_tiled/issues/31)).

Also, some of the feature are currently not working very well for "diamond" isometric maps: colliders don't have the proper shape ([#32](https://github.com/adrien-bon/bevy_ecs_tiled/issues/32)) and are not always at the right place ([#48](https://github.com/adrien-bon/bevy_ecs_tiled/issues/48)).
But we hope to have better support for them in the future.

## I'm using an isometric map and it seems all messed up !

Make sure you are actually using a "diamond" map and not a "staggered" one, which are not supported.

Also, for isometric maps, you may want to tweak the [`TilemapRenderSettings`](https://docs.rs/bevy_ecs_tilemap/latest/bevy_ecs_tilemap/map/struct.TilemapRenderSettings.html) component from `bevy_ecs_tilemap` to enable Y-sorting and adjust the chunk size.

More information in the [isometric maps example](https://github.com/adrien-bon/bevy_ecs_tiled/blob/main/examples/orientation_isometric.rs#L34)

## I want to add physics to my game, how should I do ?

You can automatically spawn physics colliders on tiles or objects using either [Avian](https://github.com/Jondolf/avian) or [Rapier](https://github.com/dimforge/bevy_rapier) physics backend.
Basically, all you have to do is to add another plugin to your app.
The crate handle the rest.

You can select on which objects or tiles you colliders are spawn based upon their name and using Tiled builtin collision editor.
Everything is explained in the [dedicated guide](guides/physics.md).

## I want to update my Bevy entities directly from Tiled, how should I do ?

Let's say you are building a top-down turn-based RPG.
You probably want to give each of your tile some kind of information to determine if it can be crossed and what's the associated movement cost.

Using [Tiled custom properties](https://doc.mapeditor.org/en/stable/manual/custom-properties/), we can define this information in Tiled editor: the crate will automatically insert the corresponding Bevy `Component` on the tile entity when you load the map, so you can use it in your game logic.
See the [dedicated guide](guides/properties.md) for more information.

## How to enable map hot-reload ?

You need to enable Bevy `file_watcher` feature.
`bevy_ecs_tiled` will then be able to automatically reload a map that was updated with Tiled.

## I found a bug ! / Feature 'X' is missing ! / How do I do 'Y' ?

This crate is still a work in-progress so it's likely you'll find bugs, miss some feature or just wonder how to achieve something.

You can have a look to [already openned issues](https://github.com/adrien-bon/bevy_ecs_tiled/issues) and if it does not already exist, please fill a new one !
I try to address every issue as quickly as possible.

Also, contributions are more than welcome !
If you want to contribute, please have a look to [contribution guide](misc/contributing.md) and feel free to open a PR ! :)
