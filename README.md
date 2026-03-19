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

```bash
# Start with default logging
./target/release/dns_server

# Start with debug logging
RUST_LOG=debug ./target/release/dns_server

# Start with trace-level logging (very verbose)
RUST_LOG=trace ./target/release/dns_server
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
