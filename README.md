# DNS Server

A high-performance, multithreaded DNS recursive resolver written in Rust. Fully compatible with `dig` and designed for production deployment.

## Overview

This DNS server is a recursive resolver that can handle complex DNS queries by traversing the hierarchical DNS system. It supports concurrent query processing through an async/await architecture powered by tokio, enabling it to handle multiple simultaneous requests efficiently.

### Supported Record Types
- **A** - IPv4 address records
- **AAAA** - IPv6 address records
- **NS** - Nameserver records
- **MX** - Mail exchange records
- **CNAME** - Canonical name (alias) records

## Architecture

### Multithreaded Design
The DNS server evolved from a single-threaded architecture to a fully asynchronous, multithreaded design using Rust's tokio runtime. This enables efficient handling of concurrent DNS queries.

**Key architectural components:**

- **Tokio Async Runtime**: The `#[tokio::main]` macro sets up the async runtime, allowing the server to handle many concurrent connections without blocking threads.

- **Socket Sharing with Arc<>**: The UDP socket is wrapped in an `Arc<>` (Atomic Reference Counted pointer), enabling safe sharing across multiple async tasks without explicit locking. Each spawned task receives a clone of the Arc, allowing concurrent access to the shared socket.

- **Task Spawning**: Each incoming DNS query is handled by spawning a new async task via `tokio::spawn()`. This allows the server to process multiple queries concurrently without blocking:
  ```rust
  tokio::spawn(async move {
      match resolve::handle_query(&socket_clone).await {
          Ok(_) => { /* query handled */ }
          Err(e) => { /* log error, continue */ }
      }
  });
  ```

**Why this approach?**
- **Non-blocking I/O**: Async/await allows the server to handle thousands of concurrent queries without thread context switching overhead
- **Memory efficient**: Tokio manages a thread pool internally, requiring far fewer threads than traditional thread-per-request models
- **Resilient**: Each query runs in an isolated task, so individual query failures don't crash the server

### DNS Caching Architecture

The server includes an efficient **LRU (Least Recently Used) caching layer** to improve query performance:

```
┌─────────────────────────────────────┐
│    Incoming DNS Query (dig)         │
└────────────────┬────────────────────┘
                 │
                 ▼
        ┌────────────────────┐
        │ Cache Lookup       │
        │ (O(1) hash lookup) │
        └────┬───────────┬───┘
             │           │
          HIT│           │MISS
             ▼           ▼
        ┌─────────┐  ┌──────────────────┐
        │ Return  │  │ Recursive Lookup │
        │ Cached  │  │ (traverse DNS)   │
        │ Result  │  └────────┬─────────┘
        └─────────┘           │
                              ▼
                        ┌──────────────────┐
                        │ Store in Cache   │
                        │ (with TTL)       │
                        └────────┬─────────┘
                                 ▼
                        ┌──────────────────┐
                        │ Return Response  │
                        └──────────────────┘
```

**Key features:**
- **Thread-safe**: Uses Arc<Mutex<>> for safe concurrent access
- **TTL-aware**: Cached entries expire according to DNS TTL values
- **LRU eviction**: Automatically removes least recently used entries when capacity is exceeded
- **Configurable size**: Cache size is controlled via CLI argument
- **Hit rate tracking**: Monitors cache performance with hit/miss statistics
- **Periodic logging**: Logs cache statistics every 60 seconds

**Cache trait design** (`src/cache/mod.rs`):
- Backend-agnostic through the `DnsCache` trait
- Current implementation: LRU cache (`src/cache/lru.rs`)
- Future implementations: SQLite, Redis, Memcached (drop-in replacements)

## Performance & Deployment

### Concurrent Query Handling
The server efficiently processes multiple simultaneous DNS requests:
- Each query runs in its own isolated async task
- The single UDP socket is safely shared via Arc<>, eliminating unnecessary allocations
- No global locks or mutexes needed for query processing

### Production Readiness
This server is suitable for production deployment:
- **Graceful error handling**: Query errors (timeouts, malformed packets, resolution failures) are logged but don't crash the server
- **Individual query isolation**: One query's failure has no impact on other concurrent queries
- **Resource efficiency**: Can handle thousands of concurrent queries with minimal memory overhead
- **Non-blocking operations**: Uses tokio's async I/O primitives for responsive query handling

### Reliability Features
- **Query timeout handling**: The resolver respects timeouts during recursion
- **Partial failure tolerance**: If a nameserver is unavailable, the resolver tries alternates
- **Robust error recovery**: Malformed packets are rejected gracefully with appropriate DNS error responses

## Building & Running

### Prerequisites

**On Linux/macOS:**
- Rust (install via [rustup](https://rustup.rs))
- Standard build tools

**On Windows (using WSL - recommended):**
- Windows Subsystem for Linux (WSL2)
- Within WSL:
  - Rust (install via [rustup](https://rustup.rs))
  - Build tools: `sudo apt-get install build-essential pkg-config`

### Building for Production

```bash
# Development build
cargo build

# Production release build (optimized)
cargo build --release

# Run the server
./target/release/dns_server
```

### Running the Server

#### Basic Usage

```bash
# Start with default configuration (port 2053, cache size 1000)
./target/release/dns_server

# With custom logging
RUST_LOG=debug ./target/release/dns_server
RUST_LOG=info ./target/release/dns_server
```

#### Configuration Options

```bash
# Custom port and cache size
./target/release/dns_server --port 5353 --cache-size 5000

# Short flags
./target/release/dns_server -p 5353 -c 5000

# Full configuration
./target/release/dns_server \
  --bind 0.0.0.0 \
  --port 2053 \
  --cache-size 10000 \
  --log-level debug

# Disable caching (useful for testing)
./target/release/dns_server --cache-size 0

# Get help
./target/release/dns_server --help
```

**Configuration parameters:**
- `--bind <ADDR>` - Bind address (default: 0.0.0.0)
- `--port <PORT>` - UDP port (default: 2053)
- `-p, --port <PORT>` - Short form
- `--cache-size <SIZE>` - Number of DNS records to cache (default: 1000)
- `-c, --cache-size <SIZE>` - Short form
- `--log-level <LEVEL>` - Log level: trace, debug, info, warn, error (default: debug)
- `-l, --log-level <LEVEL>` - Short form
- `--threads <COUNT>` - Number of worker threads (default: auto-detect)
- `-t, --threads <COUNT>` - Short form
- `--help` - Show help message
- `-h` - Short help

### Caching Examples

```bash
# Production setup with large cache (10k entries, 100MB+)
./target/release/dns_server --cache-size 10000 --log-level info

# Development with debug logging
./target/release/dns_server --cache-size 100 --log-level debug

# High performance: large cache for frequently accessed domains
./target/release/dns_server --cache-size 50000 --log-level warn

# Testing: disable cache to measure resolution time
./target/release/dns_server --cache-size 0
```

**Port Requirements:**
- The server binds to `0.0.0.0:2053` by default
- Ensure port 2053 is available and firewall rules allow UDP traffic
- On Unix-like systems, you may need elevated privileges to bind to ports below 1024

### Testing the Server

```bash
# Query a domain
dig @127.0.0.1 -p 2053 google.com

# On PowerShell
dig "@127.0.0.1" -p 2053 google.com

# Query a specific record type
dig @127.0.0.1 -p 2053 google.com AAAA
dig @127.0.0.1 -p 2053 google.com MX
```

## Caching Performance

The built-in LRU cache dramatically improves query performance for frequently accessed domains:

### Performance Metrics

**Without cache (every query performs recursive lookup):**
- Initial query: 200-500ms
- All queries require nameserver traversal

**With cache (hits on repeated queries):**
- Initial query: 200-500ms (recursive lookup + cache)
- Cached query: <5ms (immediate response)
- Result: **40-100x faster** for cached queries

### Cache Hit Rate

Monitor cache effectiveness through periodic statistics:

```
Cache Stats: 45 hits, 5 misses, 32 entries, 90.00% hit rate
```

- **High hit rate (>80%)**: Cache is working well, queries are being reused
- **Low hit rate (<20%)**: Many unique domains, increase cache size if memory allows

### Configuring Cache Size

The optimal cache size depends on your use case:

```bash
# Small deployments (< 1000 unique domains/day)
./target/release/dns_server --cache-size 1000

# Medium deployments (1000-10000 unique domains/day)
./target/release/dns_server --cache-size 5000

# Large deployments (>10000 unique domains/day)
./target/release/dns_server --cache-size 50000
```

**Memory usage estimate:**
- Each cached entry: ~500-1000 bytes
- 1000 entries: ~1 MB
- 10000 entries: ~10 MB
- 50000 entries: ~50 MB

### Monitoring Cache Health

Enable debug logging to monitor cache behavior in detail:

```bash
RUST_LOG=debug ./target/release/dns_server --cache-size 1000
```

Key log messages:
- `Cache hit for '<domain>' (<qtype>)` - Cache hit occurred
- `Cached result for '<domain>' (<qtype>) - cache_size: N` - Entry added
- `Cache Stats: X hits, Y misses, Z entries, A% hit rate` - Statistics (every 60s)

See [CACHE_TESTING.md](CACHE_TESTING.md) for detailed testing procedures.

## Concurrency Testing

### Single Query Test
```bash
dig "@127.0.0.1" -p 2053 twitch.tv
```

### Multiple Concurrent Queries
Test the server's ability to handle multiple simultaneous requests:

```bash
# Send 10 concurrent queries (bash/sh)
for i in {1..10}; do 
  dig "@127.0.0.1" -p 2053 example.com &
done
wait

# Send 50 concurrent queries for stress testing
for i in {1..50}; do 
  dig "@127.0.0.1" -p 2053 "domain$i.com" &
done
wait
```

### Stress Testing with Different Domains
```bash
# Test with variety of domains and record types
dig "@127.0.0.1" -p 2053 google.com A &
dig "@127.0.0.1" -p 2053 google.com AAAA &
dig "@127.0.0.1" -p 2053 google.com MX &
dig "@127.0.0.1" -p 2053 github.com &
dig "@127.0.0.1" -p 2053 cloudflare.com &
wait
```

### Monitoring Server Performance
Watch server logs while running concurrent queries:
```bash
# Terminal 1: Start server with debug logging
RUST_LOG=debug ./target/release/dns_server

# Terminal 2: Run concurrent queries
for i in {1..20}; do 
  dig "@127.0.0.1" -p 2053 example.com &
done
wait
```

Expected behavior:
- All queries complete successfully
- No query blocks others (concurrent execution)
- Error messages are logged but server continues running
- Server handles connection interruptions gracefully

## Example Output

### Client Query
```text
$ dig "@127.0.0.1" -p 2053 twitch.tv
; <<>> DiG 9.16.26 <<>> @127.0.0.1 -p 2053 twitch.tv
; (1 server found)
;; global options: +cmd
;; Got answer:
;; ->>HEADER<<- opcode: QUERY, status: NOERROR, id: 53972
;; flags: qr rd ra; QUERY: 1, ANSWER: 4, AUTHORITY: 4, ADDITIONAL: 4

;; QUESTION SECTION:
;twitch.tv.                     IN      A

;; ANSWER SECTION:
twitch.tv.              3600    IN      A       151.101.130.167
twitch.tv.              3600    IN      A       151.101.66.167
twitch.tv.              3600    IN      A       151.101.2.167
twitch.tv.              3600    IN      A       151.101.194.167

;; AUTHORITY SECTION:
twitch.tv.              172800  IN      NS      ns-1450.awsdns-53.org.
twitch.tv.              172800  IN      NS      ns-1778.awsdns-30.co.uk.
twitch.tv.              172800  IN      NS      ns-219.awsdns-27.com.
twitch.tv.              172800  IN      NS      ns-664.awsdns-19.net.

;; Query time: 500 msec
;; SERVER: 127.0.0.1#2053(127.0.0.1)
;; WHEN: Mon May 16 15:46:05 India Standard Time 2022
;; MSG SIZE  rcvd: 303
```

### Server Output
```text
✓ DNS server listening on 0.0.0.0:2053
Query = DnsQuestion { name: "twitch.tv", qtype: A }
Looking up A twitch.tv  with ns 198.41.0.4 
Looking up A twitch.tv  with ns 192.42.173.30 
Looking up A ns-219.awsdns-27.com  with ns 198.41.0.4 
Looking up A ns-219.awsdns-27.com  with ns 192.5.6.30 
Looking up A ns-219.awsdns-27.com  with ns 205.251.192.28 
Looking up A twitch.tv  with ns 205.251.192.219 
Answer: A { domain: "twitch.tv", addr: 151.101.130.167, ttl: 3600 } 
Answer: A { domain: "twitch.tv", addr: 151.101.66.167, ttl: 3600 } 
Answer: A { domain: "twitch.tv", addr: 151.101.2.167, ttl: 3600 }
Answer: A { domain: "twitch.tv", addr: 151.101.194.167, ttl: 3600 }
Authorities : NS { domain: "twitch.tv", host: "ns-1450.awsdns-53.org", ttl: 172800 }
Authorities : NS { domain: "twitch.tv", host: "ns-1778.awsdns-30.co.uk", ttl: 172800 }
Authorities : NS { domain: "twitch.tv", host: "ns-219.awsdns-27.com", ttl: 172800 }
Authorities : NS { domain: "twitch.tv", host: "ns-664.awsdns-19.net", ttl: 172800 }
```

## Code Documentation

Generate and view the Rust code documentation:
```bash
cargo doc --open
```

## References

- [RFC 1034](https://datatracker.ietf.org/doc/html/rfc1034) - Domain Names - Concepts and Facilities
- [RFC 1035](https://datatracker.ietf.org/doc/html/rfc1035) - Domain Names - Implementation and Specification
