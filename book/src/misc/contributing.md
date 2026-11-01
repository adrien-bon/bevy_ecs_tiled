# Contributing

Thank you for your interest in contributing to `bevy_ecs_tiled`!
Whether you're fixing bugs, adding features, improving documentation, or helping other users, your contributions are greatly appreciated.

## Where to Start

If you're unsure where to begin, check out the [GitHub issues](https://github.com/adrien-bon/bevy_ecs_tiled/issues) page.
You might find it helpful to look at issues:

- [tagged with the `enhancement` label](https://github.com/adrien-bon/bevy_ecs_tiled/issues?q=is%3Aopen+is%3Aissue+label%3Aenhancement)
- [tagged with the `limitation` label](https://github.com/adrien-bon/bevy_ecs_tiled/issues?q=is%3Aopen+is%3Aissue+label%3Alimitation)

If you feel you can tackle one of these, please feel free to submit a PR!

Other valuable ways to contribute include:

- Helping other users by responding to issues or reviewing PRs
- Reporting bugs or suggesting improvements by opening new issues
- Contributing to dependencies, such as [`rs-tiled`](https://github.com/mapeditor/rs-tiled), [`bevy_ecs_tilemap`](https://github.com/StarArawn/bevy_ecs_tilemap), or other crates in the Bevy ecosystem (or Bevy itself!)

## Contribution Guidelines

When submitting a pull request, please ensure that:

- **CI passes:**
  Run `just check` locally to check formatting, linting, and tests.
- **Documentation:**
  Add or update in-code documentation for any new features or changes.
- **Changelog:**
  Update the `CHANGELOG.md` file with a description of your fix or feature.
- **Examples:**
  If you add a new example:
  - Update `examples/README.md` with a description of your example.
  - Add your example to the workspace `Cargo.toml` (and specify any required features).
- **Assets:**
  If you add a new map, update `assets/README.md` with its characteristics.
  If you add a new asset, update the "Assets credits" section of the main `README.md` and ensure you have the right to use it.

If you're not sure about something, feel free to open a draft PR or ask questions in your issue or PR.

## Manual Checks

Some behaviors cannot be checked by automated tests and must be checked manually.

### Tiles Rendering

Tests run without a renderer: they check that tiles are added to or removed from their tilemap `TileStorage`, but not what is actually drawn.
However, how a tile is drawn after it has been modified relies on `bevy_ecs_tilemap` internals: to make sure a moved or replaced tile is removed from its previous location, its `TilePos` component is removed then inserted again (see `sync_tiles` in `src/tiled/tile.rs`).

When changing tiles rendering code, when upgrading `bevy_ecs_tilemap` and before each release, run the `map_dynamic` example with physics colliders displayed:

```sh
cargo run --example map_dynamic --features avian
```

Then check that rendering and colliders always match:

- `[1]` (despawn a tile): the tile and its collider disappear.
- `[2]` (change a tile): the tile texture changes and its collider follows the new tile shape.
- `[3]` (move a tile), several times in a row: the tile is drawn at its new position only, along with its collider.
  Also move a tile onto another one, then away: the tile it replaced is neither drawn nor has a collider anymore.
- `[4]` (spawn a tile): the new tile is drawn, along with its collider.
- `[5]` and `[6]` (despawn an object, change an object shape): the object collider follows.
- `[R]` (reload the map): the map is back to its initial state.

### Performance

A performance test measures, on a large map, the time needed to spawn the map and to modify its tiles.
It is ignored by default: run it in release mode to compare performance before and after a change.

```sh
cargo test --release --all-features --lib perf -- --ignored --nocapture
```

---

Thanks in advance for helping make `bevy_ecs_tiled` better!
