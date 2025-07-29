//! Utilities for filtering Tiled names.
//!
//! This module provides types and helpers for filtering and matching names of Tiled objects, layers, tiles or types.
//! It is used throughout the plugin to allow selective processing of Tiled entities based on their names.

use crate::prelude::*;
use bevy::prelude::*;

/// A filter for efficiently checking if a given name matches a filter specification.
///
/// # Example
/// ```rust,no_run
/// use bevy_ecs_tiled::prelude::*;
///
/// let names_filter = TiledFilter::from(vec!["some", "name"]);
/// let regex_filter = TiledFilter::from(
///     RegexSet::new([
///         r"^some",
///         r"name$"
///     ]).unwrap());
///
/// assert!(names_filter.matches("some"));
/// assert!(!names_filter.matches("some name"));
/// assert!(regex_filter.matches("some"));
/// assert!(regex_filter.matches("some name"));
/// ```
#[derive(Default, Reflect, Clone, Debug)]
#[reflect(opaque, Debug)]
pub enum TiledFilter {
    /// Matches all names.
    #[default]
    All,
    /// Matches only the provided names.
    ///
    /// Matching is case-insensitive and ignores leading/trailing whitespace.
    Names(Vec<String>),
    /// Matches only the provided regex.
    ///
    /// See <https://docs.rs/regex/latest/regex/index.html#syntax>
    RegexSet(RegexSet),
    /// Matches no names.
    None,
}

impl From<RegexSet> for TiledFilter {
    fn from(rs: RegexSet) -> Self {
        Self::RegexSet(rs)
    }
}

impl From<Vec<&str>> for TiledFilter {
    fn from(names: Vec<&str>) -> Self {
        Self::Names(names.iter().map(|s| s.to_string()).collect())
    }
}

impl TiledFilter {
    /// Returns `true` if the provided name matches the filter.
    pub fn matches(&self, name: &str) -> bool {
        match self {
            Self::All => true,
            Self::Names(names) => names.contains(&name.trim().to_lowercase()),
            Self::RegexSet(set) => set.is_match(name),
            Self::None => false,
        }
    }
}

pub(crate) fn plugin(app: &mut App) {
    app.register_type::<TiledFilter>();
}
