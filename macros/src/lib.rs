//! This crate contains derive macros for the [`bevy_ecs_tiled`](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/) Bevy plugin.
//!
//! It should be viewed through the [embedded documentation of bevy_ecs_tiled](https://docs.rs/bevy_ecs_tiled/latest/bevy_ecs_tiled/properties/prelude/index.html) crate.

mod tiled_class;
mod tiled_custom_tile;
mod tiled_enum;
mod tiled_object;

/// Derive macro for Tiled objects.
///
/// This derive macro is used to declare in Rust either a Bevy `Component` or a Bevy `Bundle`, which corresponds to a "custom type" from Tiled.
///
/// [TiledObject] must be declared using the [register_tiled_object()](../app/trait.TiledApp.html#tymethod.register_tiled_object) function and only work for Tiled objects.
/// To do the same with tiles, see [TiledCustomTile] derive macro.
///
/// Example:
/// ```rust,no_run
/// #[derive(TiledObject, Component, Default)]
/// struct ObjectGraphics {
///     color: bevy::color::Color,
///     is_visible: bool,
/// }
/// ```
///
/// ---
/// Required additional traits:
/// - `Bundle` trait, in case you are only using Tiled "custom types" in your structure (ie. only [TiledClass] fields).
/// - `Component` trait, in case you are only using Tiled "standard types" in your structure (ie. no [TiledClass] fields).
/// - `Default` trait, so you can provide a default value in case a property is not set explicitely set in Tiled.
///
/// Note that `Component` and `Bundle` traits are mutually exclusive.
///
/// ---
/// Available attributes:
/// - `tiled_rename`: name of the Tiled type, in case it's different from the structure field.
/// - `tiled_skip`: skip the following field and do not try to get it's value from Tiled custom properties.
/// - `tiled_observer`: name of an observer (a function) which will be triggered once the object is actually added to the world.
/// The observer is triggered using the [TiledObjectCreated](../events/struct.TiledObjectCreated.html) event.
#[proc_macro_derive(TiledObject, attributes(tiled_rename, tiled_skip, tiled_observer))]
pub fn derive_tiled_objects(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    tiled_object::expand_tiled_objects_derive(syn::parse(input).unwrap())
}

/// Derive macro for Tiled custom tiles.
///
/// This derive macro is used to declare in Rust either a Bevy `Component` or a Bevy `Bundle`, which corresponds to a "custom type" from Tiled.
///
/// [TiledCustomTile] must be declared using the [register_tiled_custom_tile()](../app/trait.TiledApp.html#tymethod.register_tiled_custom_tile) function and only work for Tiled custom tiles.
/// To do the same with objects, see [TiledObject] derive macro.
///
/// Example:
/// ```rust,no_run
/// #[derive(TiledCustomTile, Component, Default)]
/// struct TileMovement {
///     movement_cost: i32,
///     has_road: bool,
/// }
/// ```
///
/// ---
/// Required additional traits:
/// - `Bundle` trait, in case you are only using Tiled "custom types" in your structure (ie. only [TiledClass] fields).
/// - `Component` trait, in case you are only using Tiled "standard types" in your structure (ie. no [TiledClass] fields).
/// - `Default` trait, so you can provide a default value in case a property is not set explicitely set in Tiled.
///
/// Note that `Component` and `Bundle` traits are mutually exclusive.
///
/// ---
/// Available attributes:
/// - `tiled_rename`: name of the Tiled type, in case it's different from the structure field.
/// - `tiled_skip`: skip the following field and do not try to get it's value from Tiled custom properties.
/// Instead use the struct default value.
/// - `tiled_observer`: name of an observer (a function) which will be triggered once the tile is actually added to the world.
/// The observer is triggered using the [TiledCustomTileCreated](../events/struct.TiledCustomTileCreated.html) event.
#[proc_macro_derive(TiledCustomTile, attributes(tiled_rename, tiled_skip, tiled_observer))]
pub fn derive_tiled_custom_tiles(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    tiled_custom_tile::expand_tiled_custom_tiles_derive(syn::parse(input).unwrap())
}

/// Derive macro for Tiled custom types.
///
/// This derive macro is used to declare in Rust a Bevy `Component`, which corresponds to a "custom type" from Tiled.
///
/// [TiledClass] should be contained in either a [TiledObject] or a [TiledCustomTile] struct.
///
/// Example:
/// ```rust,no_run
/// #[derive(TiledClass, Component, Default)]
/// struct PlayerStats {
///     health: i32,
///     mana: i32,
/// }
/// ```
///
/// ---
/// Required additional traits:
/// - `Default` trait, so you can provide a default value in case a property is not set explicitely set in Tiled.
/// - `Component` trait, note that it means you can only use Tiled "standard types" in your structure (ie. no [TiledClass] nesting)
///
/// ---
/// Available attributes:
/// - `tiled_rename`: name of the Tiled type, in case it's different from the structure field.
/// - `tiled_skip`: skip the following field and do not try to get it's value from Tiled custom properties.
#[proc_macro_derive(TiledClass, attributes(tiled_rename, tiled_skip))]
pub fn derive_tiled_classes(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    tiled_class::expand_tiled_class_derive(syn::parse(input).unwrap())
}

/// Derive macro for Tiled enums
///
/// This derive macro is used to declare in Rust an enum, which corresponds to a Tiled "string enum".
/// Note that Tiled "number enum" are not supported.
///
/// [TiledEnum] should be contained in either a [TiledObject], a [TiledCustomTile] or a [TiledClass] struct.
///
/// Example:
/// ```rust,no_run
/// #[derive(TiledEnum, Default)]
/// enum BiomeType {
///     #[default]
///     Unknown,
///     Plain,
///     Desert,
///     Forest,
///     Mountain,
/// }
/// ```
///
/// ---
/// Required traits:
/// - `Default` trait, so you can provide a default value in case a property is not set explicitely set in Tiled.
///
/// ---
/// Available attributes:
/// - `tiled_rename`: name of the Tiled enum, in case it's different from the Rust enum variant.
#[proc_macro_derive(TiledEnum, attributes(tiled_rename))]
pub fn derive_tiled_enums(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    tiled_enum::expand_tiled_enum_derive(syn::parse(input).unwrap())
}
