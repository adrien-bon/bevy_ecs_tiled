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
  Run `./tools/ci_check.sh` locally to check formatting, linting, and tests.
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

---

Thanks in advance for helping make `bevy_ecs_tiled` better!
