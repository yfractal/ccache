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

    pub fn get_through_shard(
        &self,
        key: &K,
        read_shard: &RwLockReadGuard<'a, HashMap<K, V>>,
    ) -> Option<V> {
        read_shard.get(key).cloned()
    }

    pub fn write_guard(&'a self, key: &K) -> RwLockWriteGuard<'a, HashMap<K, V>> {
        let idx = self.shard_idx(key);

        unsafe { self._write_shard(idx) }
    }

    pub fn read_guard(&'a self, key: &K) -> RwLockReadGuard<'a, HashMap<K, V>> {
        let idx = self.shard_idx(&key);

        unsafe { self._read_shard(idx) }
    }

    fn shard_idx(&self, key: &K) -> usize {
        let mut hasher = self.hasher.build_hasher();
        key.hash(&mut hasher);
        let hash = hasher.finish() as usize;
        hash % 128
    }

    unsafe fn _write_shard(&'a self, i: usize) -> RwLockWriteGuard<'a, HashMap<K, V>> {
        debug_assert!(i < self.shards.len());

        self.shards.get_unchecked(i).write().unwrap()
    }

    unsafe fn _read_shard(&'a self, i: usize) -> RwLockReadGuard<'a, HashMap<K, V>> {
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
        map.write_guard(&1).insert(1, 2);
        mep_alais.write_guard(&2).insert(2, 3);
        let read_shard = map.read_guard(&1);
        let rv = read_shard.get(&1).unwrap();
        assert_eq!(rv, &2);
    }
}
