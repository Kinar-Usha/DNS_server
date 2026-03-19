# DNS Cache Testing Guide

## Manual Testing

### Start the Server with Caching

```bash
# Build release binary
cargo build --release

# Run server with cache enabled (100 entry cache, info logging)
RUST_LOG=info ./target/release/dns_server --cache-size 100 --log-level info
```

You should see output like:
```
╔════════════════════════════════════════╗
║      DNS Server Configuration         ║
╠════════════════════════════════════════╣
║ Listen Address: 0.0.0.0:2053           ║
║ Cache Size:     100                    ║
║ Worker Threads: auto                   ║
║ Log Level:      info                   ║
╚════════════════════════════════════════╝
DNS cache initialized with size: 100
✓ DNS server listening on 0.0.0.0:2053
```

### Test 1: Basic Cache Hit

```bash
# First query - should perform recursive lookup
dig @127.0.0.1 -p 2053 google.com

# Second query - should hit cache
dig @127.0.0.1 -p 2053 google.com
```

In the server logs, you should see:
- First query: "Looking up ... with ns" messages (recursive lookup)
- Second query: "Cache hit for 'google.com' (A)" message

### Test 2: Different Query Types

```bash
# A record query (default)
dig @127.0.0.1 -p 2053 example.com

# NS record query
dig @127.0.0.1 -p 2053 example.com NS

# Repeated queries should hit cache
dig @127.0.0.1 -p 2053 example.com
dig @127.0.0.1 -p 2053 example.com NS
```

### Test 3: Concurrent Cache Access

```bash
# Run 10 concurrent queries to the same domain
for i in {1..10}; do
  dig @127.0.0.1 -p 2053 github.com +short > /dev/null 2>&1 &
done
wait
```

The server should handle all concurrent queries without errors due to Arc<Mutex<>> thread safety.

### Test 4: Cache Statistics

```bash
# Watch the server logs for periodic cache stats (every 60 seconds)
# You should see output like:
# INFO  dns_server::cache: Cache Stats: 10 hits, 5 misses, 15 entries, 66.67% hit rate
```

### Test 5: Cache Eviction Under Load

```bash
# Fill cache with 150 different queries (exceeds 100 entry cache)
for i in {1..150}; do
  dig @127.0.0.1 -p 2053 "test$i.example.com" +short > /dev/null 2>&1
done
```

The LRU cache should evict oldest entries when capacity is exceeded.

## Expected Behavior

### Cache Hits
- Same domain + query type should return cached response
- Log entry: "Cache hit for '<domain>' (<qtype>)"
- Should be significantly faster than initial lookup

### Cache Misses
- New domain or query type should trigger recursive lookup
- Log entry: "Looking up ... with ns" messages
- Result is cached after recursive lookup completes

### TTL Handling
- Cached records include TTL from the authoritative server
- Expired records are not returned; cache miss is forced
- Default TTL fallback is 300 seconds for answers with TTL

### Concurrency
- Multiple simultaneous queries to same domain should all hit cache
- No race conditions or deadlocks
- All responses are consistent

### Eviction
- When cache size exceeds capacity, LRU (least recently used) entries are removed
- With cache size 100 and many queries, cache remains bounded
- Hit rate may fluctuate depending on query distribution

## Performance Impact

### Before Caching
- Every query requires recursive lookup (multiple DNS queries)
- Typical lookup time: 200-500ms per query

### After Caching
- First query: ~200-500ms (recursive lookup + cache)
- Cached queries: <5ms (immediate cache hit)
- Result: 40-100x faster for cached queries

## Monitoring Cache Health

Enable debug logging to see all cache operations:

```bash
RUST_LOG=debug ./target/release/dns_server --cache-size 100 --log-level debug
```

Key log messages:
- `Cache hit for '<domain>' (<qtype>)` - Cache hit occurred
- `Cached result for '<domain>' (<qtype>) - cache_size: N` - Entry added to cache
- `Cache Stats: X hits, Y misses, Z entries, A% hit rate` - Periodic statistics
