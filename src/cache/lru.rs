//! LRU cache backend for DNS records

use super::{CachedDnsRecord, CacheKey, DnsCache};
use lru::LruCache;
use std::num::NonZeroUsize;
use std::sync::{Arc, Mutex};

/// LRU-based DNS cache implementation
pub struct LruDnsCache {
    cache: Arc<Mutex<LruCache<CacheKey, CachedDnsRecord>>>,
    hits: Arc<Mutex<u64>>,
    misses: Arc<Mutex<u64>>,
}

impl LruDnsCache {
    /// Create a new LRU cache with the specified maximum size
    pub fn new(capacity: usize) -> Self {
        let capacity = NonZeroUsize::new(capacity).unwrap_or(NonZeroUsize::new(1000).unwrap());
        
        Self {
            cache: Arc::new(Mutex::new(LruCache::new(capacity))),
            hits: Arc::new(Mutex::new(0)),
            misses: Arc::new(Mutex::new(0)),
        }
    }
}

impl DnsCache for LruDnsCache {
    fn get(&self, key: &CacheKey) -> Option<CachedDnsRecord> {
        let mut cache = self.cache.lock().unwrap();

        // Clean up expired entries while we're at it
        cache.iter().for_each(|(_, record)| {
            if record.is_expired() {
                //TODO: Mark for removal (can't modify while iterating)
            }
        });

        if let Some(record) = cache.get(key).cloned() {
            if !record.is_expired() {
                *self.hits.lock().unwrap() += 1;
                return Some(record);
            }
        }

        *self.misses.lock().unwrap() += 1;
        None
    }

    fn set(&self, key: CacheKey, record: CachedDnsRecord) {
        let mut cache = self.cache.lock().unwrap();
        cache.put(key, record);
    }
    fn clear(&self) {
        self.cache.lock().unwrap().clear();
    }

    fn cleanup(&self) -> usize {
        let mut cache = self.cache.lock().unwrap();
        let initial_size = cache.len();

        // Collect keys of expired entries (can't remove while iterating)
        let expired_keys: Vec<_> = cache
            .iter()
            .filter(|(_, record)| record.is_expired())
            .map(|(key, _)| key.clone())
            .collect();

        // Remove expired entries
        for key in &expired_keys {
            cache.pop(key);
        }

        let removed = initial_size - cache.len();
        if removed > 0 {
            log::debug!("Cache cleanup: removed {} expired entries", removed);
        }
        removed
    }

    fn hit_count(&self) -> u64 {
        *self.hits.lock().unwrap()
    }

    fn miss_count(&self) -> u64 {
        *self.misses.lock().unwrap()
    }

    fn size(&self) -> usize {
        self.cache.lock().unwrap().len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::DnsPacket;

    #[test]
    fn test_cache_basic_operations() {
        let cache = LruDnsCache::new(100);
        let key = CacheKey::new("example.com".to_string(), 1);
        let packet = DnsPacket::new();
        let record = CachedDnsRecord::new(packet, 300);

        // Initially miss
        assert_eq!(cache.get(&key), None);
        assert_eq!(cache.miss_count(), 1);

        // Store and retrieve
        cache.set(key.clone(), record.clone());
        assert_eq!(cache.size(), 1);

        let retrieved = cache.get(&key);
        assert!(retrieved.is_some());
        assert_eq!(cache.hit_count(), 1);
    }

    #[test]
    fn test_cache_expiration() {
        let cache = LruDnsCache::new(100);
        let key = CacheKey::new("example.com".to_string(), 1);
        let packet = DnsPacket::new();
        let record = CachedDnsRecord::new(packet, 0); // Immediate expiration

        cache.set(key.clone(), record);
        
        // Should be expired immediately
        assert_eq!(cache.get(&key), None);
    }

    #[test]
    fn test_cache_lru_eviction() {
        let cache = LruDnsCache::new(2);
        let packet = DnsPacket::new();

        // Fill the cache
        for i in 0..3 {
            let key = CacheKey::new(format!("example{}.com", i), 1);
            let record = CachedDnsRecord::new(packet.clone(), 300);
            cache.set(key, record);
        }

        // Cache should have evicted the oldest entry
        assert_eq!(cache.size(), 2);
    }

    #[test]
    fn test_cleanup_removes_expired() {
        let cache = LruDnsCache::new(100);
        let packet = DnsPacket::new();

        // Add 5 expired entries (TTL=0) and 5 valid entries (TTL=3600)
        for i in 0..5 {
            let key = CacheKey::new(format!("expired{}.com", i), 1);
            let record = CachedDnsRecord::new(packet.clone(), 0);
            cache.set(key, record);
        }

        for i in 0..5 {
            let key = CacheKey::new(format!("valid{}.com", i), 1);
            let record = CachedDnsRecord::new(packet.clone(), 3600);
            cache.set(key, record);
        }

        assert_eq!(cache.size(), 10);

        // Run cleanup
        let removed = cache.cleanup();
        assert_eq!(removed, 5);
        assert_eq!(cache.size(), 5);
    }

    #[test]
    fn test_cleanup_keeps_valid() {
        let cache = LruDnsCache::new(100);
        let key = CacheKey::new("example.com".to_string(), 1);
        let packet = DnsPacket::new();
        let record = CachedDnsRecord::new(packet, 3600);

        cache.set(key.clone(), record);
        assert_eq!(cache.size(), 1);

        // Cleanup should not remove valid entries
        let removed = cache.cleanup();
        assert_eq!(removed, 0);
        assert_eq!(cache.size(), 1);

        // Entry should still be retrievable
        assert!(cache.get(&key).is_some());
    }

    #[test]
    fn test_cleanup_empty_cache() {
        let cache = LruDnsCache::new(100);

        // Should not panic on empty cache
        let removed = cache.cleanup();
        assert_eq!(removed, 0);
        assert_eq!(cache.size(), 0);
    }
}
