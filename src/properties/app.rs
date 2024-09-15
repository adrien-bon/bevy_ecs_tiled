//! App traits related to Tiled custom properties.

use crate::prelude::*;
use bevy::{app::App, ecs::bundle::Bundle};

#[allow(clippy::needless_doctest_main)]
pub trait TiledApp {
    /// Register a Tiled object.
    ///
    /// This function triggers the spawn of the corresponding [TiledObject](../prelude/derive.TiledObject.html) whenever it is encountered in a Tiled map.
    ///
    /// Example:
    /// ```rust,no_run
    /// use bevy::prelude::*;
    /// use bevy_ecs_tiled::prelude::*;
    ///
    /// // `ObjectGraphics` definition
    /// #[derive(TiledObject, Component, Default)]
    /// struct ObjectGraphics {
    ///     color: bevy::color::Color,
    ///     is_visible: bool,
    /// }
    ///
    /// fn main() {
    ///     App::new()
    ///         // Where 'ObjectGraphicsInfos' is the custom type name as it appears in Tiled
    ///         .register_tiled_object::<ObjectGraphics>("ObjectGraphicsInfos");
    /// }
    /// ```
    fn register_tiled_object<T: TiledObject + Bundle>(&mut self, ident: &str) -> &mut Self;

    /// Register a Tiled custom tile.
    ///
    /// This function triggers the spawn of the corresponding [TiledCustomTile](../prelude/derive.TiledCustomTile.html) whenever it is encountered in a Tiled map.
    ///
    /// Example:
    /// ```rust,no_run
    /// use bevy::prelude::*;
    /// use bevy_ecs_tiled::prelude::*;
    ///
    /// // `TileMovement` definition
    /// #[derive(TiledCustomTile, Component, Default)]
    /// struct TileMovement {
    ///     movement_cost: i32,
    ///     has_road: bool,
    /// }
    ///
    /// fn main() {
    ///     App::new()
    ///         // Where 'TileMovementInfos' is the custom type name as it appears in Tiled
    ///         .register_tiled_custom_tile::<TileMovement>("TileMovementInfos");
    /// }
    /// ```
    fn register_tiled_custom_tile<T: TiledCustomTile + Bundle>(&mut self, ident: &str)
        -> &mut Self;
}

impl TiledApp for App {
    fn register_tiled_object<T: TiledObject + Bundle>(&mut self, ident: &str) -> &mut Self {
        match self
            .world_mut()
            .get_non_send_resource_mut::<TiledObjectRegistry>()
        {
            Some(mut registry) => {
                registry.insert(ident.to_string(), Box::new(PhantomTiledObject::<T>::new()));
                self
            }
            None => {
                self.world_mut()
                    .insert_non_send_resource(TiledObjectRegistry::default());
                self.register_tiled_object::<T>(ident)
            }
        }
    }

    fn register_tiled_custom_tile<T: TiledCustomTile + Bundle>(
        &mut self,
        ident: &str,
    ) -> &mut Self {
        match self
            .world_mut()
            .get_non_send_resource_mut::<TiledCustomTileRegistry>()
        {
            Some(mut registry) => {
                registry.insert(
                    ident.to_string(),
                    Box::new(PhantomTiledCustomTile::<T>::new()),
                );
                self
            }
            None => {
                self.world_mut()
                    .insert_non_send_resource(TiledCustomTileRegistry::default());
                self.register_tiled_custom_tile::<T>(ident)
            }
        }
    }
}
