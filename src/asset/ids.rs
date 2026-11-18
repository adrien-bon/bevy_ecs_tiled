//! IDs of the items of Tiled maps and worlds.
//!
//! These IDs are those defined in Tiled: use them to look up the corresponding raw [`tiled`] data
//! in a [`TiledMapAsset`](crate::asset::map::TiledMapAsset) or a world asset.

#[allow(unused_imports)]
use crate::prelude::*;
use bevy::prelude::*;

/// Implements conversions from and to [`u32`] and [`std::fmt::Display`] for an ID newtype.
macro_rules! impl_id {
    ($id:ident) => {
        impl From<u32> for $id {
            fn from(id: u32) -> Self {
                Self(id)
            }
        }

        impl From<$id> for u32 {
            fn from(id: $id) -> Self {
                id.0
            }
        }

        impl std::fmt::Display for $id {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.0.fmt(f)
            }
        }
    };
}

/// ID of a layer, as defined in Tiled.
///
/// This ID is unique within a map, whatever the layer type.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord, Reflect)]
#[reflect(Clone, Debug, Default, PartialEq, Hash)]
pub struct LayerId(pub u32);

impl_id!(LayerId);

/// ID of a tileset in a [`TiledMapAsset`].
///
/// For tilesets referenced by the map, it is the index of the tileset in the Tiled map tilesets.
/// Tilesets only referenced by object templates get an ID after the map tilesets.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord, Reflect)]
#[reflect(Clone, Debug, Default, PartialEq, Hash)]
pub struct TilesetId(pub u32);

impl_id!(TilesetId);

/// ID of an object, as defined in Tiled.
///
/// This ID is unique within a map.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord, Reflect)]
#[reflect(Clone, Debug, Default, PartialEq, Hash)]
pub struct ObjectId(pub u32);

impl_id!(ObjectId);

#[cfg(feature = "world")]
/// ID of a map in a [`TiledWorldAsset`], as its index in the `.world` file.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord, Reflect)]
#[reflect(Clone, Debug, Default, PartialEq, Hash)]
pub struct WorldMapId(pub u32);

#[cfg(feature = "world")]
impl_id!(WorldMapId);

pub(crate) fn plugin(app: &mut App) {
    app.register_type::<LayerId>();
    app.register_type::<TilesetId>();
    app.register_type::<ObjectId>();
    #[cfg(feature = "world")]
    app.register_type::<WorldMapId>();
}
