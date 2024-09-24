# Examples

This directory provides a list of examples which demonstrate how to use the crate.

To run an example, you can run the following command, where XXX is the example name:

```bash
cargo run --example XXX
```

Be advised that some of the examples require to enable one or multiple feature.
In that case, cargo will return an error and print the proper command to run.

Please note that if you want a more in-depth explanation of some of the crate concepts, there is a [dedicated book](https://adrien-bon.github.io/bevy_ecs_tiled/) to cover that.

## Keyboard controls

In most of the examples, you can use the following action keys:

- **WASD**: to move around the camera
- **Z/X**: to zoom out / zoom in
- **A/E**: to rotate the map (not only the camera)

Note: some of on the key can be differents for "non-QWERTY" keyboard layout.

For instance, on an "AZERTY" keyboard, you must use `A` instead of `Q` and `Z` instead of `W`.

## Examples list

| Name | Required features | Description |
|------|-------------------|-------------|
| `finite` | None | This example shows a finite orthogonal map with an external tileset. |
| `finite_embedded` | None | This example shows a finite orthogonal map with an embedded tileset. |
| `infinite` | None | This example shows an infinite orthogonal map with an external tileset. |
| `infinite_embedded` | None | This example shows an infinite orthogonal map with an embedded tileset. |
| `reload` | None | This example demonstrates how to load and unload maps. |
| `finite_rapier` | `rapier_debug` | This example shows a finite orthogonal map with an external tileset and Rapier physics. |
| `infinite_rapier` | `rapier_debug` | This example shows an infinite orthogonal map with an external tileset and Rapier physics. |
| `hex_map` | None | This example cycle through all four kinds of hexagonal maps and display debug informations about Tiled objects. |
| `controller_rapier` | `rapier_debug` | This example shows a simple player-controlled object using Rapier physics. You can move the object using arrow keys. |
| `user_properties` | `user_properties` | This example shows how to map custom tiles and objects properties from Tiled to Bevy Components. |
| `user_properties_rapier` | `user_properties`, `rapier_debug` | This example shows how to map custom tiles / objects properties from Tiled to Bevy Components and manually spawn Rapier colliders from them. |
| `isometric_map`| None | This example cycle through two kinds of isometric maps and display debug informations about Tiled objects. |
| `finite_avian` | `avian` | This example shows a finite orthogonal map with an external tileset and Avian2D physics. |
| `infinite_avian` | `avian` | This example shows an infinite orthogonal map with an external tileset and Avian2D physics. |
| `controller_avian` | `avian` | This example shows a simple player-controlled object using Avian2D physics. You can move the object using arrow keys. |
| `user_properties_avian` | `user_properties`, `avian` | This example shows how to map custom tiles / objects properties from Tiled to Bevy Components and manually spawn Avian colliders from them. |
| `delayed_spawn` | None | This example will delay map spawn from asset loading to demonstrate both are decoupled. |
| `multiple_tilesets` | None | This example shows a finite orthogonal map with multiple external tilesets. |
| `custom_physics` | `physics` | This example shows how to use a custom physics backend. |
| `map_events` | None | This example shows how to use map loading events. |
