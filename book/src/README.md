# Introduction

{{ #include intro.md }}

---

**Disclaimer:** both this book and the whole crate have been heavilly inspired by the [`bevy_ecs_ldtk` crate](https://github.com/Trouv/bevy_ecs_ldtk), which is basically the equivalent of `bevy_ecs_tiled` but for the [LDTK](https://ldtk.io/) map editor.
Thanks for the inspiration! :)

---

## Purpose of this book

This book aims to give you a better understanding of how [`bevy_ecs_tiled`](https://github.com/adrien-bon/bevy_ecs_tiled) works, what you can achieve with it and how you can do it.

It will focus on high-level concepts, design concerns and basic tutorials.
If you need an API reference, please have a look at the [`bevy_ecs_tiled` API reference](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/).
The [examples](https://github.com/adrien-bon/bevy_ecs_tiled/tree/main/examples/README.md) directory is also a good place to start.

Finally, this book assume you already have some sort of knowledge about [Bevy](https://bevyengine.org/) and [Tiled](https://www.mapeditor.org/) map editor.
There are already some good documentations available on these topics and some resources are referenced [in the dedicated section](misc/useful-links.md).

## Architecture of this book

This book is divided in several categories:

- **Design and explanation**: how does the plugin work and why some technical choices have been made;
- **How-To's**: in-depth tutorials about a specific aspect of  the plugin;
- **Migration guides**: migration guides for specific versions;
- **Miscellaneous**: other topics that did not fit in other categories.

Good reading ! :)
