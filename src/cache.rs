use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};
use std::time::{Duration, Instant};

use crate::package::PackageCandidate;

struct CacheEntry {
    results: Vec<PackageCandidate>,
    timestamp: Instant,
}

struct CacheInner {
    entries: HashMap<String, CacheEntry>,
}

pub struct SearchCache {
    inner: Mutex<CacheInner>,
    ttl: Duration,
}

impl SearchCache {
    pub fn new(ttl_secs: u64) -> Self {
        Self {
            inner: Mutex::new(CacheInner {
                entries: HashMap::new(),
            }),
            ttl: Duration::from_secs(ttl_secs),
        }
    }

    pub fn get(&self, key: &str) -> Option<Vec<PackageCandidate>> {
        let guard = self.inner.lock().ok()?;
        let entry = guard.entries.get(key)?;
        if entry.timestamp.elapsed() < self.ttl {
            Some(entry.results.clone())
        } else {
            None
        }
    }

    pub fn insert(&self, key: String, results: Vec<PackageCandidate>) {
        if let Ok(mut guard) = self.inner.lock() {
            guard.entries.insert(
                key,
                CacheEntry {
                    results,
                    timestamp: Instant::now(),
                },
            );
        }
    }

    pub fn clear(&self) {
        if let Ok(mut guard) = self.inner.lock() {
            guard.entries.clear();
        }
    }
}

static GLOBAL_CACHE: LazyLock<SearchCache> = LazyLock::new(|| SearchCache::new(300));

pub fn cached_search(
    package: &str,
    backends: &[&dyn crate::backends::PackageManager],
) -> Vec<PackageCandidate> {
    if let Some(cached) = GLOBAL_CACHE.get(package) {
        return cached;
    }

    let mut results = Vec::new();
    for backend in backends {
        if let Ok(mut candidates) = backend.search(package) {
            results.append(&mut candidates);
        }
    }

    GLOBAL_CACHE.insert(package.to_string(), results.clone());
    results
}

pub fn clear_cache() {
    GLOBAL_CACHE.clear();
}
