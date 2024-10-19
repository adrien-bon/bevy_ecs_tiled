//! This module contains utilities to work with Tiled names.
use bevy::utils::HashSet;

/// A struct to specify names when using [ObjectNameFilter]
///
/// Example:
/// ```rust,no_run
/// use bevy_ecs_tiled::prelude::*;
///
/// let name_to_check = String::from("some name");
/// let matching_allowed_names = ObjectNames::Names(vec!("some name".to_string()));
/// let non_matching_allowed_names = ObjectNames::Names(vec!("some other name".to_string()));
///
/// assert_eq!(ObjectNameFilter::from(&matching_allowed_names).contains(&name_to_check), true);
/// assert_eq!(ObjectNameFilter::from(&non_matching_allowed_names).contains(&name_to_check), false);
/// assert_eq!(ObjectNameFilter::from(&ObjectNames::All).contains(&name_to_check), true);
/// assert_eq!(ObjectNameFilter::from(&ObjectNames::None).contains(&name_to_check), false);
/// ```
#[derive(Default, Clone, PartialEq)]
pub enum ObjectNames {
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

/// Allow to check if a provided [ObjectNames] matches a given name.
///
/// Example:
/// ```rust,no_run
/// use bevy_ecs_tiled::prelude::*;
///
/// let name_to_check = String::from("some name");
/// let matching_allowed_names = ObjectNames::Names(vec!("some name".to_string()));
/// let non_matching_allowed_names = ObjectNames::Names(vec!("some other name".to_string()));
///
/// assert_eq!(ObjectNameFilter::from(&matching_allowed_names).contains(&name_to_check), true);
/// assert_eq!(ObjectNameFilter::from(&non_matching_allowed_names).contains(&name_to_check), false);
/// assert_eq!(ObjectNameFilter::from(&ObjectNames::All).contains(&name_to_check), true);
/// assert_eq!(ObjectNameFilter::from(&ObjectNames::None).contains(&name_to_check), false);
/// ```
#[derive(Clone)]
pub enum ObjectNameFilter {
    /// Matches all names.
    All,
    /// Matches only provided names.
    Names(HashSet<String>),
    /// Does not match any name.
    None,
}

impl From<&ObjectNames> for ObjectNameFilter {
    /// Initialize a [ObjectNameFilter] from an [ObjectNames].
    fn from(value: &ObjectNames) -> Self {
        match value {
            ObjectNames::All => ObjectNameFilter::All,
            ObjectNames::Names(names) => {
                let names = names
                    .iter()
                    .map(|x| x.trim().to_lowercase())
                    .filter(|x| !x.is_empty())
                    .collect();
                ObjectNameFilter::Names(names)
            }
            ObjectNames::None => ObjectNameFilter::None,
        }
    }
}

impl ObjectNameFilter {
    /// Determine if provided [str] matches the filter.
    pub fn contains(&self, name: &str) -> bool {
        match self {
            ObjectNameFilter::All => true,
            ObjectNameFilter::Names(names) => names.contains(name),
            ObjectNameFilter::None => false,
        }
    }
}
