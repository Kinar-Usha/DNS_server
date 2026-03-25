//! DNS caching layer with support for multiple backends

pub mod lru;

use crate::protocol::DnsPacket;
use std::time::{SystemTime, UNIX_EPOCH};

/// Represents a cached DNS record with TTL support
#[derive(Clone, Debug, PartialEq)]
pub struct CachedDnsRecord {
    /// The cached DNS response packet
    pub packet: DnsPacket,
    /// Timestamp when the record was cached (seconds since epoch)
    pub cached_at: u64,
    /// Time-to-live in seconds
    pub ttl: u32,
}

impl CachedDnsRecord {
    /// Create a new cached record
    pub fn new(packet: DnsPacket, ttl: u32) -> Self {
        let cached_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            packet,
            cached_at,
            ttl,
        }
    }

    /// Check if the record has expired
    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        now >= self.cached_at + self.ttl as u64
    }
}

/// Cache key combining domain name and query type
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct CacheKey {
    pub domain: String,
    pub qtype: u16, // QueryType as u16
}

impl CacheKey {
    pub fn new(domain: String, qtype: u16) -> Self {
        Self { domain, qtype }
    }
}

/// Cache statistics for monitoring
#[derive(Clone, Debug)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub size: usize,
    pub hit_rate: f64,
}

impl CacheStats {
    pub fn new(hits: u64, misses: u64, size: usize) -> Self {
        let total = hits + misses;
        let hit_rate = if total > 0 {
            (hits as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        Self {
            hits,
            misses,
            size,
            hit_rate,
        }
    }

    pub fn log(&self) {
        log::info!(
            "Cache Stats: {} hits, {} misses, {} entries, {:.2}% hit rate",
            self.hits, self.misses, self.size, self.hit_rate
        );
    }
}

/// Trait for DNS cache backends
/// Implementations must be thread-safe (Send + Sync)
pub trait DnsCache: Send + Sync {
    /// Get a cached record by key
    fn get(&self, key: &CacheKey) -> Option<CachedDnsRecord>;

    /// Store a record in the cache
    fn set(&self, key: CacheKey, record: CachedDnsRecord);

    /// Clear all cache entries
    fn clear(&self);

    /// Remove expired entries from cache and return count removed
    fn cleanup(&self) -> usize;

    /// Get the number of cache hits
    fn hit_count(&self) -> u64;

    /// Get the number of cache misses
    fn miss_count(&self) -> u64;

    /// Get the current number of entries in the cache
    fn size(&self) -> usize;

    /// Get cache statistics
    fn get_stats(&self) -> CacheStats {
        CacheStats::new(self.hit_count(), self.miss_count(), self.size())
    }
}
