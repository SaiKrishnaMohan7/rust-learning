use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    rc::Rc,
};

pub struct Entry {
    key: u32,
    value: u32,
}

impl Drop for Entry {
    fn drop(&mut self) {
        println!("Entry Dropped for key: {}", self.key);
    }
}
pub struct Cache {
    store: RefCell<HashMap<u32, Rc<Entry>>>,
    recents: RefCell<Vec<Rc<Entry>>>,
    computes: Cell<u32>,
}

impl Cache {
    pub fn new() -> Self {
        return Self {
            store: RefCell::new(HashMap::new()),
            recents: RefCell::new(Vec::new()),
            computes: Cell::new(0),
        };
    }
    fn expensive_compute(key: u32) -> u32 {
        return key * key;
    }
}

impl Cache {
    pub fn get(&self, key: u32) -> u32 {
        if let Some(entry) = self.store.borrow().get(&key) {
            return entry.value;
        }
        let val = Self::expensive_compute(key);
        let entry = Rc::new(Entry { key, value: val });
        self.store.borrow_mut().insert(key, Rc::clone(&entry));
        self.recents.borrow_mut().push(Rc::clone(&entry)); // entry co-owned by both Vec and HashMap
        self.compute();

        return val;
    }

    fn compute(&self) {
        self.computes.set(self.computes.get() + 1);
    }

    pub fn remove(&self, key: u32) -> Option<Rc<Entry>> {
        let removed = self.store.borrow_mut().remove(&key);
        if removed.is_some() {
            self.recents.borrow_mut().retain(|e| e.key != key);
        }
        return removed;
    }

    // to be used by integration test later on
    pub fn get_compute(&self) -> u32 {
        return self.computes.get();
    }

    pub fn recent_keys(&self) -> Vec<u32> {
        return self.recents.borrow().iter().map(|e| e.key).collect();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn computes_on_miss() {
        let cache = Cache::new();
        assert_eq!(cache.get(3), 9);
        assert_eq!(cache.computes.get(), 1);
    }

    #[test]
    fn compute_stays_same_for_existing_value() {
        let cache = Cache::new();
        assert_eq!(cache.get(3), 9);
        assert_eq!(cache.get(3), 9);
        assert_eq!(cache.computes.get(), 1);
    }

    #[test]
    fn remove_returns_entry_for_existing_key() {
        let cache = Cache::new();
        cache.get(3);
        let removed = cache.remove(3);
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().value, 9);
    }

    #[test]
    fn remove_returns_none_for_missing_key() {
        let cache = Cache::new();
        assert!(cache.remove(99).is_none());
    }

    #[test]
    fn removed_key_recomputes_on_next_get() {
        let cache = Cache::new();
        cache.get(3);
        assert_eq!(cache.computes.get(), 1);

        cache.remove(3);
        cache.get(3);
        assert_eq!(cache.computes.get(), 2); // gone from store → recomputed
    }

    #[test]
    fn recent_keys_are_in_insertion_order() {
        let cache = Cache::new();
        cache.get(5);
        cache.get(2);
        cache.get(9);
        assert_eq!(cache.recent_keys(), vec![5, 2, 9]);
    }

    #[test]
    fn cache_hit_does_not_duplicate_recent_key() {
        let cache = Cache::new();
        cache.get(5);
        cache.get(5); // hit — should not push again
        assert_eq!(cache.recent_keys(), vec![5]);
    }

    #[test]
    fn recent_keys_after_remove() {
        let cache = Cache::new();
        cache.get(5);
        cache.remove(5);
        assert_ne!(cache.recent_keys(), vec![5]);
    }

    #[test]
    fn removed_entry_stays_alive_in_recents() {
        let cache = Cache::new();
        cache.get(5);

        let removed = cache.remove(5).unwrap(); // store's Rc handed to us
        assert_eq!(Rc::strong_count(&removed), 2); // us + recents = 2 owners

        // recents still owns a live entry:
        assert_eq!(cache.recent_keys(), vec![5]);
    }
}
