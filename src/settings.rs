use bevy::{prelude::*, utils::HashSet};

#[derive(Default, Clone, Component)]
pub struct TiledMapSettings {
    /// Name your Tiled object layers one of these to automatically add all
    /// shapes as colliders.
    pub collision_layer_names: CollisionLayerNames,
}

/// Specify the object layer names to process for collision shapes.
#[derive(Default, Clone)]
pub enum CollisionLayerNames {
    #[default]
    All,
    /// Only the named object layers will be processed.
    ///
    /// Names are case-insensitive and leading/trailing whitespace
    /// will be trimmed.
    Named(Vec<String>),
    None,
}

impl TiledMapSettings {
    pub fn collision_layer_names(&self) -> Option<HashSet<String>> {
        match self.collision_layer_names {
            CollisionLayerNames::All => return None,
            CollisionLayerNames::Named(ref names) => {
                return Some(HashSet::from_iter(
                    names
                        .clone()
                        .into_iter()
                        .map(|x| x.to_lowercase().trim().to_owned())
                        .filter(|x| !x.is_empty()),
                ));
            }
            CollisionLayerNames::None => return Some(HashSet::new()),
        }
    }
}
