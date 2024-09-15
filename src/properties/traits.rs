//! Traits related to Tiled custom properties.
//!
//! These traits should be used exclusively through the [derive macros](../prelude/index.html) from `bevy_ecs_tiled_macro`.

use crate::prelude::*;
use bevy::{prelude::*, utils::HashMap};
use std::marker::PhantomData;
use tiled::Properties;

pub trait TiledObject {
    fn initialize(commands: &mut Commands, create_event: &TiledObjectCreated);
}

pub trait TiledCustomTile {
    fn initialize(commands: &mut Commands, create_event: &TiledCustomTileCreated);
}

pub trait TiledClass {
    fn create(properties: &Properties) -> Self;
}

pub trait TiledEnum {
    fn get_identifier(ident: &str) -> Self;
}

pub(crate) type TiledObjectRegistry = HashMap<String, Box<dyn PhantomTiledObjectTrait>>;
pub(crate) type TiledCustomTileRegistry = HashMap<String, Box<dyn PhantomTiledCustomTileTrait>>;

pub(crate) struct PhantomTiledObject<T: TiledObject + Bundle> {
    marker: PhantomData<T>,
}

#[allow(clippy::new_without_default)]
impl<T: TiledObject + Bundle> PhantomTiledObject<T> {
    pub fn new() -> Self {
        Self {
            marker: PhantomData,
        }
    }
}

pub(crate) trait PhantomTiledObjectTrait {
    fn initialize(&self, commands: &mut Commands, create_event: &TiledObjectCreated);
}

impl<T: TiledObject + Bundle> PhantomTiledObjectTrait for PhantomTiledObject<T> {
    fn initialize(&self, commands: &mut Commands, create_event: &TiledObjectCreated) {
        T::initialize(commands, create_event);
    }
}

pub(crate) struct PhantomTiledCustomTile<T: TiledCustomTile + Bundle> {
    marker: PhantomData<T>,
}

#[allow(clippy::new_without_default)]
impl<T: TiledCustomTile + Bundle> PhantomTiledCustomTile<T> {
    pub fn new() -> Self {
        Self {
            marker: PhantomData,
        }
    }
}

pub(crate) trait PhantomTiledCustomTileTrait {
    fn initialize(&self, commands: &mut Commands, create_event: &TiledCustomTileCreated);
}

impl<T: TiledCustomTile + Bundle> PhantomTiledCustomTileTrait for PhantomTiledCustomTile<T> {
    fn initialize(&self, commands: &mut Commands, create_event: &TiledCustomTileCreated) {
        T::initialize(commands, create_event);
    }
}
