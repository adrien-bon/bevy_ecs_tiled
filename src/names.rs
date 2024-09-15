//! This modules contains utilities to work with Tiled names.
use bevy::utils::HashSet;

/// A filter to specify names of Tiled objects.
#[derive(Default, Clone)]
pub enum ObjectNames {
    /// Matches all names.
    #[default]
    All,
    /// Matches only provided names .
    /// 
    /// Names are case-insensitive and leading/trailing whitespace will be trimmed.
    Names(Vec<String>),
    /// Does not match any name.
    None,
}

#[allow(dead_code)]
#[derive(Clone)]
pub(crate) enum ObjectNameFilter {
    All,
    Names(HashSet<String>),
    None,
}

impl From<&ObjectNames> for ObjectNameFilter {
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

#[allow(dead_code)]
impl ObjectNameFilter {
    pub fn contains(&self, name: &str) -> bool {
        match self {
            ObjectNameFilter::All => true,
            ObjectNameFilter::Names(names) => names.contains(name),
            ObjectNameFilter::None => false,
        }
    }
}
