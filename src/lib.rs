use std::{
    collections::HashMap,
    sync::{
        Arc, Mutex,
        atomic::{AtomicU32, Ordering},
    },
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
    store: Mutex<HashMap<u32, Arc<Entry>>>,
    recents: Mutex<Vec<Arc<Entry>>>,
    computes: AtomicU32,
}

impl Cache {
    pub fn new() -> Self {
        return Self {
            store: Mutex::new(HashMap::new()),
            recents: Mutex::new(Vec::new()),
            computes: AtomicU32::new(0),
        };
    }
    fn expensive_compute(key: u32) -> u32 {
        std::thread::sleep(std::time::Duration::from_millis(50));
        return key * key;
    }
}

impl Cache {
    pub fn get(&self, key: u32) -> u32 {
        let mut store = self.store.lock().unwrap();
        if let Some(entry) = store.get(&key) {
            return entry.value;
        }

        let val = Self::expensive_compute(key);
        let entry = Arc::new(Entry { key, value: val });

        store.insert(key, Arc::clone(&entry));
        self.recents.lock().unwrap().push(Arc::clone(&entry));
        self.compute();

        return val;
    }

    fn compute(&self) {
        self.computes.fetch_add(1, Ordering::SeqCst);
    }

    pub fn remove(&self, key: u32) -> Option<Arc<Entry>> {
        {
            let mut store = self.store.lock().unwrap();
            let removed = store.remove(&key);

            return removed;
        }
    }

    // to be used by integration test later on
    pub fn get_compute(&self) -> u32 {
        return self.computes.load(Ordering::SeqCst);
    }

    pub fn recent_keys(&self) -> Vec<u32> {
        return self.recents.lock().unwrap().iter().map(|e| e.key).collect();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn computes_on_miss() {
        let cache = Cache::new();
        assert_eq!(cache.get(3), 9);
        assert_eq!(cache.get_compute(), 1);
    }

    #[test]
    fn compute_stays_same_for_existing_value() {
        let cache = Cache::new();
        assert_eq!(cache.get(3), 9);
        assert_eq!(cache.get(3), 9);
        assert_eq!(cache.get_compute(), 1);
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
        assert_eq!(cache.get_compute(), 1);

        cache.remove(3);
        cache.get(3);
        assert_eq!(cache.get_compute(), 2); // gone from store → recomputed
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
        assert_eq!(cache.recent_keys(), vec![5]);
    }

    #[test]
    fn removed_entry_stays_alive_in_recents() {
        let cache = Cache::new();
        cache.get(5);

        let removed = cache.remove(5).unwrap(); // store's Arc handed to us
        assert_eq!(Arc::strong_count(&removed), 2); // us + recents = 2 owners

        // recents still owns a live entry:
        assert_eq!(cache.recent_keys(), vec![5]);
    }

    // ---- concurrency ----

    #[test]
    fn cache_is_shareable_across_threads() {
        let cache = Arc::new(Cache::new());
        let mut handles = vec![];

        for _ in 0..8 {
            let cache = Arc::clone(&cache);
            handles.push(thread::spawn(move || {
                for k in 0..5 {
                    assert_eq!(cache.get(k), k * k);
                }
            }));
        }
        for h in handles {
            h.join().unwrap();
        }

        // every key ends up cached and correct, regardless of interleaving
        for k in 0..5 {
            assert_eq!(cache.get(k), k * k);
        }
    }

    #[test]
    fn concurrent_gets_compute_once_per_key() {
        let cache = Arc::new(Cache::new());
        let mut handles = vec![];

        for _ in 0..8 {
            let cache = Arc::clone(&cache);
            handles.push(thread::spawn(move || {
                for k in 0..5 {
                    cache.get(k);
                }
            }));
        }
        for h in handles {
            h.join().unwrap();
        }

        // 5 distinct keys — ideally computed once each.
        // PREDICT before running. Run it several times.
        assert_eq!(cache.get_compute(), 5);
    }
}
