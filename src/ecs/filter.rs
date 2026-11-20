//! Filtering of Tiled names, used to select which layers, objects or types are processed.

use bevy::prelude::*;

/// Component holding the name of a Tiled item as it appears in the Tiled editor.
///
/// Apply for Layers, Tilesets and Objects.
#[derive(Component, Reflect, Clone, Debug, Default, PartialEq, Eq, Hash)]
#[component(immutable)]
#[reflect(Component, Debug, Default, PartialEq, Hash)]
pub struct TiledName(pub String);

/// A filter for efficiently checking if a given name matches a filter specification.
///
/// # Example
/// ```rust
/// use bevy_ecs_tiled::{prelude::*, regex};
///
/// let names_filter = TiledFilter::from(["some", "name"]);
/// let regex_filter = TiledFilter::from(
///     regex::RegexSet::new([
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
    RegexSet(regex::RegexSet),
    /// Matches no names.
    None,
}

impl From<regex::RegexSet> for TiledFilter {
    fn from(rs: regex::RegexSet) -> Self {
        Self::RegexSet(rs)
    }
}

impl<S: Into<String>> From<Vec<S>> for TiledFilter {
    /// Builds a [`TiledFilter::Names`] from a list of names.
    fn from(names: Vec<S>) -> Self {
        names.into_iter().collect()
    }
}

impl<S: Into<String>, const N: usize> From<[S; N]> for TiledFilter {
    /// Builds a [`TiledFilter::Names`] from an array of names.
    fn from(names: [S; N]) -> Self {
        names.into_iter().collect()
    }
}

impl<S: Into<String>> FromIterator<S> for TiledFilter {
    /// Builds a [`TiledFilter::Names`] from an iterator of names.
    fn from_iter<I: IntoIterator<Item = S>>(names: I) -> Self {
        Self::Names(names.into_iter().map(Into::into).collect())
    }
}

impl TiledFilter {
    /// Returns `true` if the provided name matches the filter.
    pub fn matches(&self, name: &str) -> bool {
        match self {
            Self::All => true,
            Self::Names(names) => {
                // Normalize both sides: stored names may have been provided with uppercase letters
                let name = name.trim();
                names.iter().any(|n| eq_ignore_case(n.trim(), name))
            }
            Self::RegexSet(set) => set.is_match(name),
            Self::None => false,
        }
    }
}

/// Case-insensitive comparison of two strings, without allocating.
fn eq_ignore_case(a: &str, b: &str) -> bool {
    // Most names are ASCII
    if a.is_ascii() && b.is_ascii() {
        return a.eq_ignore_ascii_case(b);
    }
    a.chars()
        .flat_map(char::to_lowercase)
        .eq(b.chars().flat_map(char::to_lowercase))
}

pub(crate) fn plugin(app: &mut App) {
    app.register_type::<TiledFilter>();
    app.register_type::<TiledName>();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_filter_is_case_insensitive() {
        let filter = TiledFilter::from(["Player", "  Enemy "]);
        assert!(filter.matches("player"));
        assert!(filter.matches("PLAYER"));
        assert!(filter.matches(" Player "));
        assert!(filter.matches("enemy"));
        assert!(!filter.matches("npc"));
    }

    #[test]
    fn names_filter_can_be_collected() {
        let filter = ["Player", "enemy"].into_iter().collect::<TiledFilter>();
        assert!(filter.matches("player"));
        let filter = TiledFilter::from(vec![String::from("npc")]);
        assert!(filter.matches("NPC"));
    }

    #[test]
    fn names_filter_handles_non_ascii_names() {
        let filter = TiledFilter::from(vec!["Écluse"]);
        assert!(filter.matches("écluse"));
        assert!(filter.matches("ÉCLUSE"));
        assert!(!filter.matches("ecluse"));
    }

    #[test]
    fn names_filter_built_from_variant_is_case_insensitive() {
        let filter = TiledFilter::Names(vec!["Player".to_string()]);
        assert!(filter.matches("player"));
    }
}
