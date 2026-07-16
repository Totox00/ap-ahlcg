use std::{collections::HashMap, hash::Hash};

#[derive(Debug)]
pub struct Counter<K: Eq + Hash, V: Copy> {
    inner: HashMap<K, (V, bool)>,
}

impl<K: Eq + Hash, V: Copy> Counter<K, V> {
    pub fn new() -> Counter<K, V> {
        Counter { inner: HashMap::new() }
    }

    pub fn add(&mut self, name: K, val: V) {
        if let Some((_, unique)) = self.inner.get_mut(&name) {
            *unique = false;
        } else {
            self.inner.insert(name, (val, true));
        }
    }

    pub fn unique(&self) -> impl Iterator<Item = V> {
        self.inner.values().filter(|(_, unique)| *unique).map(|(val, _)| *val)
    }

    pub fn is_unique(&self, key: &K) -> bool {
        self.inner.get(key).is_some_and(|(_, unique)| *unique)
    }
}
