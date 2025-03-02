# Why using Tiled ?

[Tiled](https://www.mapeditor.org/) may feel a bit outdated in terms of "look and feel", especially when compared with more modern map editor tools like [LDTK](https://ldtk.io/).
However it has **a lot** of features which make it very interesting.

If we compare with [LDTK](https://ldtk.io/), they both have a set of powerful features like:

- auto-tiling
- adding gameplay informations to map tiles and objects
- working with worlds

But [Tiled](https://www.mapeditor.org/) also have a set of unique features:

- support for both isometric and hexagonal maps
- native support for animated tiled

Since I specifically wanted to work with hexagonal maps the choice was obvious for me !

However, if it's not your case and you just want to use orthogonal map, you could give a shot at using [LDTK](https://ldtk.io/) instead, especially using the [`bevy_ecs_ldtk` crate](https://github.com/Trouv/bevy_ecs_ldtk).
Or stay with [Tiled](https://www.mapeditor.org/), it also works :)
