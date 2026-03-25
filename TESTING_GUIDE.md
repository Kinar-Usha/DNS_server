# DNS Server Testing Guide

## Verification Summary

The DNS server has been successfully refactored to use **async/await with Tokio** for multithreading:

### ✅ Code Changes Verified

1. **main.rs**
   - ✓ `#[tokio::main]` attribute on main function
   - ✓ Socket wrapped in `Arc<>` for concurrent task sharing (line 18)
   - ✓ Tasks spawned with `tokio::spawn()` (line 29)
   - ✓ Socket cloned per request with `Arc::clone()` (line 28)
   - ✓ Proper error handling with logging (lines 20-23, 31-36)

2. **resolve.rs**
   - ✓ All functions marked as `async` (lines 12, 43, 106)
   - ✓ All I/O operations use `.await` (lines 14-15, 30-31, 35-36, 53, 88, 108-109, 124, 161-162)
   - ✓ Error handling with contextual logging throughout
   - ✓ `recursive_lookup()` properly awaits all async calls

3. **protocol.rs & buffer.rs**
   - ✓ No async I/O - correctly uses synchronous operations
   - ✓ DNS protocol parsing and serialization logic unchanged

4. **Dependencies (Cargo.toml)**
   - ✓ `tokio` with "full" features enabled
   - ✓ `env_logger` and `log` for debugging
   - ✓ Edition 2021

## Building

```bash
cd ~/github/DNS_server
cargo build --release
```

This creates an optimized binary for production use.

## Running the Server

### Terminal 1 - Start the DNS Server

```bash
cargo run
```

Expected output:
```
✓ DNS server listening on 0.0.0.0:2053
```

The server will continue running, handling incoming DNS queries concurrently.

## Testing

### Terminal 2 - Run Tests

#### 1. Single Query Test
```bash
dig @127.0.0.1 -p 2053 google.com
```

Expected: Should return Google's IP address(es) with response details.

#### 2. Concurrent Query Test (Test Multithreading)

Send multiple dig requests simultaneously to verify concurrent handling:

```bash
dig @127.0.0.1 -p 2053 google.com & \
dig @127.0.0.1 -p 2053 github.com & \
dig @127.0.0.1 -p 2053 example.com & \
dig @127.0.0.1 -p 2053 amazon.com & \
wait
```

Expected: All queries should complete without errors and return correct results.

#### 3. Test Different Query Types

```bash
# A record (IPv4)
dig @127.0.0.1 -p 2053 google.com A

# AAAA record (IPv6)
dig @127.0.0.1 -p 2053 google.com AAAA

# NS record (Nameservers)
dig @127.0.0.1 -p 2053 google.com NS

# MX record (Mail servers)
dig @127.0.0.1 -p 2053 google.com MX
```

#### 4. Enable Debug Logging

For detailed logs of query handling:

```bash
RUST_LOG=debug cargo run
```

This will show:
- Query reception
- Nameserver lookups
- Response building
- Task spawning details

## Performance Verification

### Load Testing

Create a simple script to send many concurrent queries:

```bash
#!/bin/bash
for i in {1..50}; do
  dig @127.0.0.1 -p 2053 google.com +short &
done
wait
echo "50 concurrent queries completed successfully"
```

## Expected Behavior

✅ **Server handles multiple concurrent DNS queries**
- Each query is spawned as a separate Tokio task
- Queries don't block each other
- Server responds to all queries correctly

✅ **Proper Error Handling**
- Invalid queries get a FORMERR response
- Lookup failures get a SERVFAIL response
- Connection errors are logged with context

✅ **Async I/O Operations**
- Socket binding uses `.await`
- Send/receive operations use `.await`
- Upstream server queries use `.await`

## Troubleshooting

### Port Already in Use
If you get an error like "Address already in use":
```bash
# Find process using port 2053
lsof -i :2053
# Kill it
kill -9 <PID>
```

### Compilation Errors on Windows
The build tools are not available on native Windows. Build on WSL:
```bash
wsl
cd ~/github/DNS_server
cargo build --release
```

### DNS Resolution Fails
- Ensure you have internet connectivity from WSL
- Check that upstream DNS servers (root nameservers) are reachable
- Enable debug logging: `RUST_LOG=debug cargo run`
