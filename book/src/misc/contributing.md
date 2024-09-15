# Contributing

If you would like to contribute but you're unsure where to start, here is a short wishlist and some notes on how to get started.

First, you can have a look at [GH issues](https://github.com/adrien-bon/bevy_ecs_tiled/issues).
More specifically, the ones that are:

- [tagged with `enhancement` label](https://github.com/adrien-bon/bevy_ecs_tiled/issues?q=is%3Aopen+is%3Aissue+label%3Aenhancement)
- [tagged with `limitation` label](https://github.com/adrien-bon/bevy_ecs_tiled/issues?q=is%3Aopen+is%3Aissue+label%3Alimitation)

If you feel like you can tackle on of these, please, feel free to submit a PR!

Helping other users, respond to issues, help review PRs or just openning new issues is also very helpful !

Also, another way of helping is to contribute on crates we rely on, namely [`rs-tiled`](https://github.com/mapeditor/rs-tiled) and [`bevy_ecs_tilemap`](https://github.com/StarArawn/bevy_ecs_tilemap), or anything else in the Bevy ecosystem.

## Contribution guidelines

If you submit a PR, please make sure that:

- the CI is green (you can locally run the `./ci_check.sh` script)
- you add proper in-code documentation
- you update the `CHANGELOG.md` file with a description of your fix
- if you add a new example, update the `examples/README.md` file with a description of your example and the `Cargo.toml` file with your example name (and any feature it may need)
- if you add a new map, update the `assets/README.md` file with your map characteristics
- if you add a new asset, update the "Assets credits" section of the `README.md` file and make sure that you actually have the right to use it!

Thanks in advance! :)
