# Z-ordering

In 2D games, the Z-axis determines the rendering order: elements with a higher Z value appear in front of those with a lower Z value.  
For a deeper dive into Bevy's coordinate system and Z-ordering, see [the Bevy cheatbook](https://bevy-cheatbook.github.io/fundamentals/coords.html).

## Layer Ordering in Tiled and Bevy

When designing your map in Tiled, you expect that layers higher in the stack visually cover those below.  
This is especially important for isometric maps, which often use multiple tile layers for proper visual stacking.

To reproduce this behavior in Bevy, `bevy_ecs_tiled` assigns an incremental Z offset to each layer in the hierarchy.

If we call this offset `OFFSET`:

- The topmost layer (in Tiled) will have a Z transform of `0`
- The next layer down will have a Z transform of `-1 × OFFSET`
- The next one: `-2 × OFFSET`
- And so on...

By default, `OFFSET` is set to `+100`.  
You can adjust this value by adding or modifying the [`TiledMapLayerZOffset`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/tiled/map/struct.TiledMapLayerZOffset.html) component on your map entity.

> **Tip:**  
> `bevy_ecs_tilemap` also applies a small Z offset to each chunk for correct rendering.  
> If your layer offset is too small, you may see unexpected rendering order issues.  
> A larger offset (like the default `100`) helps avoid this.

## Objects on a Layer

All objects on a given layer share the same Z offset as their parent layer.  
This can cause issues if two objects (such as sprites) overlap at the same position:  
Bevy cannot reliably determine which one should be drawn on top, leading to ["Z-fighting"](https://en.wikipedia.org/wiki/Z-fighting)—a flickering effect as the renderer alternates between the two.

### How to Avoid Z-fighting

Currently, there is no built-in way in `bevy_ecs_tiled` to automatically resolve Z-fighting between objects on the same layer.  
However, you can work around this by listening to map events and manually adjusting the Z offset of your objects after they are spawned.

See the [map events example](https://github.com/adrien-bon/bevy_ecs_tiled/blob/dev/examples/map_events.rs) for a practical demonstration of this approach.
