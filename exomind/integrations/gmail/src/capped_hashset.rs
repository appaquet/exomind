use std::{
    borrow::Borrow,
    collections::{HashSet, LinkedList},
    hash::Hash,
};

pub struct CappedHashSet<K: Hash + Eq + Clone> {
    set: HashSet<K>,
    list: LinkedList<K>,
    capacity: usize,
}

impl<K: Hash + Eq + Clone> CappedHashSet<K> {
    pub fn new(capacity: usize) -> CappedHashSet<K> {
        CappedHashSet {
            set: HashSet::new(),
            list: LinkedList::new(),
            capacity,
        }
    }

    pub fn insert(&mut self, item: K) {
        if self.set.insert(item.clone()) {
            self.list.push_front(item);
        }

        self.cap();
    }

    pub fn insert_all(&mut self, items: &[K]) {
        for item in items {
            if self.set.insert(item.clone()) {
                self.list.push_front(item.clone());
            }
        }

        self.cap();
    }

    #[cfg(test)]
    pub fn len(&self) -> usize {
        self.set.len()
    }

    pub fn contains<Q: ?Sized>(&self, item: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.set.contains(item)
    }

    fn cap(&mut self) {
        while self.list.len() > self.capacity {
            if let Some(item) = self.list.pop_back() {
                self.set.remove(&item);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn golden_path() {
        let mut hs = CappedHashSet::new(2);

        hs.insert("hello");
        hs.insert("world");
        assert_eq!(hs.len(), 2);

        assert!(hs.contains("hello"));
        assert!(hs.contains("world"));

        hs.insert("bonjour");
        assert_eq!(hs.len(), 2);
        assert!(!hs.contains("hello"));
        assert!(hs.contains("bonjour"));
        assert!(hs.contains("world"));

        hs.insert("monde");
        assert_eq!(hs.len(), 2);
        assert!(hs.contains("bonjour"));
        assert!(hs.contains("monde"));

        hs.insert("monde");
        assert_eq!(hs.len(), 2);
        assert!(hs.contains("bonjour"));
        assert!(hs.contains("monde"));

        hs.insert_all(&["hello", "world"]);
        assert_eq!(hs.len(), 2);
        assert!(hs.contains("hello"));
        assert!(hs.contains("world"));
    }
}
