use std::collections::HashMap;
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use core::hash::{BuildHasher, Hash, Hasher};
use std::collections::hash_map::RandomState;

pub struct PartitionedHashMap<K, V, S> {
    shards: Box<[RwLock<HashMap<K, V, S>>]>,
    hasher: S,
}

impl<'a, K: 'a + Eq + Hash + core::fmt::Debug, V: Clone> PartitionedHashMap<K, V, RandomState> {
    pub fn new() -> Self {
        let hasher = RandomState::default();

        let shards = (0..128)
            .map(|_| RwLock::new(HashMap::with_capacity_and_hasher(0, hasher.clone())))
            .collect();

        Self { shards, hasher }
    }

    pub fn insert(&self, key: K, value: V) {
        let mut hasher = self.hasher.build_hasher();

        key.hash(&mut hasher);

        let hash = hasher.finish() as usize;

        let idx = hash % 128;

        let mut shard = unsafe { self.yield_write_shard(idx) };
        shard.insert(key, value);
    }

    pub fn get(&self, key: K) -> Option<V> {
        let mut hasher = self.hasher.build_hasher();

        key.hash(&mut hasher);

        let hash = hasher.finish() as usize;

        let idx = hash % 128;
        let shard = unsafe { self.yield_read_shard(idx) };

        shard.get(&key).cloned()
    }

    unsafe fn yield_write_shard(&'a self, i: usize) -> RwLockWriteGuard<'a, HashMap<K, V>> {
        debug_assert!(i < self.shards.len());

        self.shards.get_unchecked(i).write().unwrap()
    }

    unsafe fn yield_read_shard(&'a self, i: usize) -> RwLockReadGuard<'a, HashMap<K, V>> {
        debug_assert!(i < self.shards.len());

        self.shards.get_unchecked(i).read().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        let map = PartitionedHashMap::new();
        let mep_alais = &map;
        map.insert(1, 2);
        mep_alais.insert(2, 3);
        assert_eq!(map.get(1).unwrap(), 2);
    }
}
