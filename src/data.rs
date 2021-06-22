use serde::{Deserialize, Serialize};
use ulid::Ulid;
use ulid::serde::ulid_as_u128;

#[derive(Serialize, Deserialize, PartialEq, Copy, Clone, Debug, Ord, PartialOrd, Eq)]
pub struct Revision(
    #[serde(with = "ulid_as_u128")]
    pub(crate) Ulid
);

impl std::fmt::Display for Revision {
    fn fmt (&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Serialize, Deserialize, Copy, Clone)]
pub struct ValueEntry<V: Sized + Copy + Clone> {
    pub(crate) revision: Revision,
    pub(crate) prev_rev: Option<Revision>,
    pub(crate) value: V,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct KeyValueRevisionPersistence<K: Sized + Copy + Clone, V: Sized + Copy + Clone> {
    pub(crate) key: K,
    pub(crate) value_entry: ValueEntry<V>,
}

#[derive(Ord, PartialOrd, PartialEq, Eq)]
pub(crate) struct KeyAtRevision<K: Sized> {
    pub(crate) key: K,
    pub(crate) revision: Revision,
}

pub(crate) struct ValueWithPrevRevRef<V> {
    pub(crate) prev_rev: Option<Revision>,
    pub(crate) value: V,
}
