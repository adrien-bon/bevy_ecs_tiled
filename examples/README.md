# Examples

This directory provides a collection of examples demonstrating how to use the `bevy_ecs_tiled` crate in various scenarios.

To run an example, use the following command, replacing `XXX` with the example name:

```bash
cargo run --example XXX
```

> **Note:**  
> Some examples require one or more feature flags to be enabled.  
> If you attempt to run such an example without the required features, Cargo will display an error message with the correct command to use.

For a more in-depth explanation of crate concepts, see the [dedicated book](https://adrien-bon.github.io/bevy_ecs_tiled/) or the [demo_platformer](./demo_platformer/) example..

## Keyboard Controls

Most examples support the following controls:

- **WASD**: Move the camera
- **Z/X**: Zoom out / zoom in
- **A/E**: Rotate the map (not just the camera)

> **Note for non-QWERTY layouts:**  
> Some keys may differ.  
> For example, on an AZERTY keyboard, use `A` instead of `Q` and `Z` instead of `W`.

## Examples List

| Name | Required features | Description |
|------|-------------------|-------------|
| `map_anchor` | `debug` | This example shows the basic usage of `TilemapAnchor`. |
| `map_basic` | None | This example shows the basic usage of the plugin to load a Tiled map. |
| `map_events` | None | This example shows how to use map loading events. |
| `map_parallax` | None | This example demonstrates parallax scrolling with Tiled map layers. |
| `map_reload` | None | This example demonstrates how to load and unload maps. |
| `map_settings` | None | This example cycles through different map settings that can be applied. |
| `map_spawn_delay` | None | This example will delay map spawn from asset loading to demonstrate both are decoupled. |
| `orientation_hexagonal` | `debug` | This example cycles through different kinds of hexagonal maps. |
| `orientation_isometric` | `debug` | This example cycles through different kinds of isometric maps. |
| `orientation_orthogonal` | `debug` | This example cycles through different kinds of orthogonal maps. |
| `physics_avian_controller` | `avian_debug` | This example shows a simple player-controlled object using Avian2D physics. You can move the object using arrow keys. |
| `physics_avian_orientation` | `avian_debug` | This example shows Avian2D physics backend with various map orientation. |
| `physics_avian_settings` | `avian_debug` | This example shows how to use Avian2D physics backend. |
| `physics_custom` | `physics` | This example shows how to use a custom physics backend. |
| `physics_rapier_controller` | `rapier_debug` | This example shows a simple player-controlled object using Rapier physics. You can move the object using arrow keys. |
| `physics_rapier_orientation` | `rapier_debug` | This example shows Rapier physics backend with various map orientation. |
| `physics_rapier_settings` | `rapier_debug` | This example shows how to use Rapier physics backend. |
| `properties_basic` | `user_properties` | This example shows how to map custom tiles and objects properties from Tiled to Bevy Components. |
| `spacing` | None | This example shows how spacing and margins are automatically loaded. It also shows . |
| `world_basic` | None | This example shows the basic usage of the plugin to load a Tiled world. |
| `world_chunking` | `debug` | This example shows how to load Tiled World files and demonstrates chunking the loaded maps. |

