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
| `basic` | None | This example shows the basic usage of the plugin. |
| `map_events` | None | This example shows how to use map loading events. |
| `map_reload` | None | This example demonstrates how to load and unload maps. |
| `map_settings` | None | This example cycles through different map settings that can be applied. |
| `map_spawn_delay` | None | This example will delay map spawn from asset loading to demonstrate both are decoupled. |
| `orientation_orthogonal` | None | This example cycles through different kinds of orthogonal maps. |
| `orientation_isometric` | None | This example cycles through different kinds of isometric maps. |
| `orientation_hexagonal` | None | This example cycles through different kinds of hexagonal maps. |
| `physics_custom` | `physics` | This example shows how to use a custom physics backend. |
| `physics_events` | `physics` | This example shows how to use physics events. |
| `physics_avian_settings` | `avian` | This example shows how to use Avian2D physics backend. |
| `physics_avian_controller` | `avian` | This example shows a simple player-controlled object using Avian2D physics. You can move the object using arrow keys. |
| `infinite_rapier` | `rapier_debug` | This example shows how to use Rapier physics backend. |
| `physics_rapier_controller` | `rapier_debug` | This example shows a simple player-controlled object using Rapier physics. You can move the object using arrow keys. |
