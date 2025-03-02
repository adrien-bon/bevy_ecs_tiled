# Z-ordering

Since we are working in 2D, the Z-axis will tell which element is in front or behind each other.
You can have a look to [the Bevy cheatbook for a more in-depth explanation](https://bevy-cheatbook.github.io/fundamentals/coords.html).

## Layers

When designing your map under Tiled, you expect that a layer will hide another one which is below in the layer hierarchy.
This is very useful when using isometric tiles for instance, because they usually have several tile layers.

To reproduce this behaviour under Bevy, we add an arbitrary offset on the Z-axis to each layers of the hierarchy.

If we call this offset `OFFSET`:

- the top-level layer will have a Z transform of `0`
- the second one will have a Z transform of `-1x OFFSET`
- the next one of `-2x OFFSET`
- the next one of `-3x OFFSET`
- etc...

By default this offset has a value of `+100`.
It can be changed by tweaking the [`TiledMapLayerZOffset`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/components/map/struct.TiledMapLayerZOffset.html) component.

Since `bevy_ecs_tilemap` also adds a small Z-axis offset to adjust how chunk are rendered, you probably don't want to have a "too small" value.

## Objects on a layer

For a given layer, all objects have the same Z offset.
It can be problematic if two objects displaying something (a Sprite for instance) have the same location.

You may observe a ["Z-fighting" issue](https://en.wikipedia.org/wiki/Z-fighting): since both sprite are at the same depth, there is no way for Bevy to properly determine which one is on top of the other.
Both will be drawn which is likely to produce some flickering.

Currently, there is not an official way to fix that.
You can work-around this issue by using map events and tweak the Z offset of your objects, as shown in the [map events example](https://github.com/adrien-bon/bevy_ecs_tiled/blob/dev/examples/map_events.rs).
