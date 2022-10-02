use std::hash::{Hash, Hasher};

use indexmap::IndexMap;

/// Function for use with the `derivative` crate
pub fn hash_indexmap<K, V, S, H>(map: &IndexMap<K, V, S>, state: &mut H)
where
    H: Hasher,
{
    map.len().hash(state);
}
