//! Utilities for working with Tiled object and layer names.
//!
//! This module provides types and helpers for filtering and matching names of Tiled objects, layers, or tiles.
//! It is used throughout the plugin to allow selective processing of Tiled entities based on their names,
//! supporting case-insensitive and whitespace-trimmed matching.

use bevy::{platform::collections::HashSet, prelude::*};

/// Specifies a set of allowed names for filtering Tiled objects, layers, or tiles.
///
/// This enum is used to define name filters for various systems (such as physics or spawning).
/// It supports matching all names, a specific list of names, or none.
///
/// # Example
/// ```rust,no_run
/// use bevy_ecs_tiled::prelude::*;
///
/// let name_to_check = String::from("some name");
/// let matching_allowed_names = TiledName::Names(vec!["some name".to_string()]);
/// let non_matching_allowed_names = TiledName::Names(vec!["some other name".to_string()]);
///
/// assert_eq!(TiledNameFilter::from(&matching_allowed_names).contains(&name_to_check), true);
/// assert_eq!(TiledNameFilter::from(&non_matching_allowed_names).contains(&name_to_check), false);
/// assert_eq!(TiledNameFilter::from(&TiledName::All).contains(&name_to_check), true);
/// assert_eq!(TiledNameFilter::from(&TiledName::None).contains(&name_to_check), false);
/// ```
#[derive(Default, Clone, PartialEq, Reflect, Debug)]
pub enum TiledName {
    /// Matches all names.
    #[default]
    All,
    /// Matches only the provided names.
    ///
    /// Names are compared case-insensitively and with leading/trailing whitespace trimmed.
    Names(Vec<String>),
    /// Matches no names.
    None,
}

/// A filter for efficiently checking if a given name matches a [`TiledName`] specification.
///
/// This type is used internally to perform fast lookups and comparisons.
/// Construct it from a [`TiledName`] using `TiledNameFilter::from`.
///
/// # Example
/// ```rust,no_run
/// use bevy_ecs_tiled::prelude::*;
///
/// let name_to_check = String::from("some name");
/// let matching_allowed_names = TiledName::Names(vec!["some name".to_string()]);
/// let non_matching_allowed_names = TiledName::Names(vec!["some other name".to_string()]);
///
/// assert_eq!(TiledNameFilter::from(&matching_allowed_names).contains(&name_to_check), true);
/// assert_eq!(TiledNameFilter::from(&non_matching_allowed_names).contains(&name_to_check), false);
/// assert_eq!(TiledNameFilter::from(&TiledName::All).contains(&name_to_check), true);
/// assert_eq!(TiledNameFilter::from(&TiledName::None).contains(&name_to_check), false);
/// ```
#[derive(Clone, Debug)]
pub enum TiledNameFilter {
    /// Matches all names.
    All,
    /// Matches only the provided names (case-insensitive, trimmed).
    Names(HashSet<String>),
    /// Matches no names.
    None,
}

impl From<&TiledName> for TiledNameFilter {
    /// Creates a [`TiledNameFilter`] from a [`TiledName`], normalizing names for case-insensitive and trimmed matching.
    fn from(value: &TiledName) -> Self {
        match value {
            TiledName::All => TiledNameFilter::All,
            TiledName::Names(names) => {
                let names = names
                    .iter()
                    .map(|x| x.trim().to_lowercase())
                    .filter(|x| !x.is_empty())
                    .collect();
                TiledNameFilter::Names(names)
            }
            TiledName::None => TiledNameFilter::None,
        }
    }
}

impl TiledNameFilter {
    /// Returns `true` if the provided name matches the filter.
    ///
    /// Matching is case-insensitive and ignores leading/trailing whitespace.
    pub fn contains(&self, name: &str) -> bool {
        match self {
            TiledNameFilter::All => true,
            TiledNameFilter::Names(names) => names.contains(&name.trim().to_lowercase()),
            TiledNameFilter::None => false,
        }
    }
}
