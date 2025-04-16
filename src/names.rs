//! This module contains utilities to work with Tiled names.
use bevy::{platform::collections::HashSet, prelude::*};

/// A struct to specify names when using [TiledNameFilter]
///
/// Example:
/// ```rust,no_run
/// use bevy_ecs_tiled::prelude::*;
///
/// let name_to_check = String::from("some name");
/// let matching_allowed_names = TiledName::Names(vec!("some name".to_string()));
/// let non_matching_allowed_names = TiledName::Names(vec!("some other name".to_string()));
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
    /// Matches only provided names.
    ///
    /// Names are case-insensitive and leading/trailing whitespace will be trimmed.
    Names(Vec<String>),
    /// Does not match any name.
    None,
}

/// Allow to check if a provided [TiledName] matches a given name.
///
/// Example:
/// ```rust,no_run
/// use bevy_ecs_tiled::prelude::*;
///
/// let name_to_check = String::from("some name");
/// let matching_allowed_names = TiledName::Names(vec!("some name".to_string()));
/// let non_matching_allowed_names = TiledName::Names(vec!("some other name".to_string()));
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
    /// Matches only provided names.
    Names(HashSet<String>),
    /// Does not match any name.
    None,
}

impl From<&TiledName> for TiledNameFilter {
    /// Initialize a [TiledNameFilter] from an [TiledName].
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
    /// Determine if provided [str] matches the filter.
    pub fn contains(&self, name: &str) -> bool {
        match self {
            TiledNameFilter::All => true,
            TiledNameFilter::Names(names) => names.contains(name),
            TiledNameFilter::None => false,
        }
    }
}
