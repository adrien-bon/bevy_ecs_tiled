# Z-ordering

In 2D games, the Z-axis determines the rendering order: elements with a higher Z value appear in front of those with a lower Z value.  
For a deeper dive into Bevy's coordinate system and Z-ordering, see [the Bevy cheatbook](https://bevy-cheatbook.github.io/fundamentals/coords.html).

---

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

---

## Objects on a Layer

Objects on a given layer inherit the Z offset of their parent layer, but each object is given a small additional offset to avoid ["Z-fighting"](https://en.wikipedia.org/wiki/Z-fighting)—a flickering effect that occurs when two objects share the same Z value.

- Each object receives a unique, small Z offset relative to its parent layer.
- This ensures that overlapping objects are rendered in a stable order.

If you need precise control over the Z position of specific objects (for example, to implement custom sorting or to ensure a particular object always appears on top), you can:

- Listen to [map events](https://github.com/adrien-bon/bevy_ecs_tiled/blob/dev/examples/map_events.rs) and manually adjust the Z offset of your objects after they are spawned.
- Use your own logic to set the Z value based on object properties or gameplay needs.
