# FAQ

## What kind of maps are supported ?

Currently, we support :

- orthogonal maps
- (mostly) isometric "diamond" maps
- hexagonal "flat-top" maps
- hexagonal "pointy-top" maps

Isometric "diamond" maps currently have an issue with colliders not having the proper shape (see [GH issue #32](https://github.com/adrien-bon/bevy_ecs_tiled/issues/32)).

Isometric "staggered" maps are not supported at all (see [GH issue #31](https://github.com/adrien-bon/bevy_ecs_tiled/issues/31)).

## Is it possible to automatically add physics colliders ?

Yes, see the [dedicated guide](guides/physics.md).

We currently support both [Avian](https://github.com/Jondolf/avian) and [Rapier](https://github.com/dimforge/bevy_rapier) physics backend.


## Is it possible to use Tiled "custom properties" ?

Yes, see the [dedicated guide](guides/properties.md).

## I found a bug! What should I do ?

Please have a look to [already openned issues](https://github.com/adrien-bon/bevy_ecs_tiled/issues) and if it does not already exists, please fill a new one !

## I want to add a new feature that's not yet in the crate!

Great news!
Please have a look to [the dedicated section](misc/contributing.md)
