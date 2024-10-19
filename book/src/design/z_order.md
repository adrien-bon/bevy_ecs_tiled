# Layers Z-ordering

When designing your map under Tiled, you expect that a layer will hide another one which is below in the layer hierarchy.
This is very useful when using isometric tiles for instance.

To reproduce this behaviour under Bevy, we add an arbitrary offset on the Z transform to each layers of the hierarchy.

If we call this offset `OFFSET`:

- the top-level layer will have a Z transform of `0`
- the second one will have a Z transform of `- OFFSET`
- the next one of `- 2*OFFSET`
- etc...

By default this offset has a value of `+100`.
It can be changed by tweaking the [`TiledMapSettings`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/components/struct.TiledMapSettings.html) component.
Since `bevy_ecs_tilemap` also play with the Z-transform to adjust how tiles from a given layers are rendered, you probably don't want to have a "too low" value.
