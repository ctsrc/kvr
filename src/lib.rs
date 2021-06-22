pub mod data;
pub mod error;
pub mod magic;

use std::collections::BTreeMap;
use std::collections::btree_map::Range;
use std::ops::RangeBounds;
use std::path::Path;

use bincode::{deserialize_from, serialize};
use serde::Serialize;
use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::data::{KeyAtRevision, KeyValueRevisionPersistence, Revision, ValueEntry, ValueWithPrevRevRef};
use crate::error::{KVRInsertionError, KVRUpdateError, KVRInitializationError};
use crate::magic::Magic;
use serde::de::DeserializeOwned;

/// An in-memory + append-only persistence key-value database with revision history.
pub struct KeyValueRevision<K: Sized + Copy + Clone, V: Sized + Copy + Clone> {
    value_entry_by_key: BTreeMap<K, ValueEntry<V>>,
    is_keeping_historical_entries_in_memory: bool,
    historical_entries: BTreeMap<KeyAtRevision<K>, ValueWithPrevRevRef<V>>,
    /// XXX: This file holds all entries and is only ever appended to, not deleted from.
    ///      On program launch we read from it.
    file: File,
}

impl<K: Ord + Copy + Clone + Serialize + Magic + DeserializeOwned, V: Copy + Clone + Serialize + Magic + DeserializeOwned> KeyValueRevision<K, V> {
    pub async fn try_init (path: impl AsRef<Path>, should_keep_historical_entries_in_memory: bool) -> Result<Self, KVRInitializationError>
    {
        let mut file = match OpenOptions::new().read(true).append(true).create(true).open(path).await {
            Ok(f) => f,
            Err(e) => return Err(KVRInitializationError::FailedToOpenFile(e)),
        };
        let mut buf = vec![];
        match file.read_to_end(&mut buf).await {
            Ok(_) => {},
            Err(e) => return Err(KVRInitializationError::FailedToReadFile(e)),
        }

        // TODO: Have an instance of our key value with known magic data
        //       at the beginning of the file, which we verify to have values.

        // TODO: Some kind of merkle-tree stuff on the data structures that we serialize,
        //       which we use as a signature for the data held in the file, and which we
        //       check up against on this here init.

        // TODO: Read in batches corresponding to a multiple of the number of bytes per serialized struct.
        //       Dunno if bincode already handles this for us when we've specified K: Sized and V: Sized.

        // TODO: Consistent Overhead Byte Stuffing. https://news.ycombinator.com/item?id=26685014

        let mut data = vec![];
        let mut cursor = &*buf;
        while cursor.len() > 0 {
            let entry: KeyValueRevisionPersistence<K, V> = match deserialize_from(&mut cursor) {
                Ok(e) => e,
                Err(e) => return Err(KVRInitializationError::FailedToDeserializeValue(e)),
            };
            data.push(entry);
        }
        let value_entry_by_key = data.iter()
            .map(|KeyValueRevisionPersistence{ key, value_entry }| (*key, *value_entry))
            .collect();

        let historical_entries = if should_keep_historical_entries_in_memory {
            // TODO: Exclude entries that are in the current live set. Alternatively, rename from historical_entries to all_revs or something.
            data.into_iter()
                .map(|KeyValueRevisionPersistence{ key, value_entry }|
                    (KeyAtRevision { key, revision: value_entry.revision },
                     ValueWithPrevRevRef { value: value_entry.value, prev_rev: value_entry.prev_rev }))
                .collect()
        } else {
            BTreeMap::new()
        };

        Ok(Self {
            value_entry_by_key,
            is_keeping_historical_entries_in_memory: should_keep_historical_entries_in_memory,
            historical_entries,
            file,
        })
    }
    pub async fn try_insert (&mut self, key: K, value: V, revision: Revision) -> Result<(), KVRInsertionError> {
        // TODO: Locking.
        // TODO PROBABLY: BTreeMap try_insert instead of locking once BTreeMap try_insert is in stable Rust.
        if let Some(_) = self.value_entry_by_key.get(&key) {
            return Err(KVRInsertionError::KeyExists)
        }
        let value_entry = ValueEntry { value, revision, prev_rev: None };
        if let Some(_) = self.value_entry_by_key.insert(key, value_entry) {
            unreachable!() // XXX: We assert unreachable because of previous check after lock which is still held.
        }
        let data = match serialize(&KeyValueRevisionPersistence { key, value_entry }) {
            Ok(d) => d,
            Err(e) => return Err(KVRInsertionError::SerializationError(e)),
        };
        match self.file.write(&data).await {
            Ok(_) => {},
            Err(e) => return Err(KVRInsertionError::IOError(e)),
        }
        Ok(())
    }
    pub async fn try_update (&mut self, key: K, value: V, revision: Revision, prev_rev: Revision) -> Result<(), KVRUpdateError> {
        // TODO: Locking.
        if self.value_entry_by_key.get(&key).ok_or(KVRUpdateError::KeyDoesNotExist)?.revision != prev_rev {
            return Err(KVRUpdateError::PrevRevMismatch)
        }
        let value_entry = ValueEntry { value, revision, prev_rev: Some(prev_rev) };
        match self.value_entry_by_key.insert(key, value_entry) {
            Some(prev_value_entry_ret) => {
                if prev_value_entry_ret.revision != prev_rev {
                    unreachable!() // XXX: Same as with the assert about unreachable above; b/c lock held.
                }
                if self.is_keeping_historical_entries_in_memory {
                    // TODO: insert prev_value_entry_ret into historical entries in memory
                }
            },
            _ => unreachable!(), // XXX: Same as with the assert about unreachable above; b/c lock held.
        }
        let data = match serialize(&KeyValueRevisionPersistence { key, value_entry }) {
            Ok(d) => d,
            Err(e) => return Err(KVRUpdateError::SerializationError(e)),
        };
        match self.file.write(&data).await {
            Ok(_) => {},
            Err(e) => return Err(KVRUpdateError::IOError(e)),
        }
        Ok(())
    }
    pub fn range<R: RangeBounds<K>> (&self, range: R) -> Range<'_, K, ValueEntry<V>> {
        self.value_entry_by_key.range(range)
    }
    // TODO: Retrieval and read-operations on historical entries
}
