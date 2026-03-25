# DNS Server Load Testing Guide

## Python Load Tester (Recommended)

### Setup

```bash
# Install required package
pip install dnspython

# Make script executable
chmod +x load_test.py
```

### Run the Load Test

**In one terminal, start the DNS server:**
```bash
cargo build --release
RUST_LOG=info ./target/release/dns_server --cache-size 1000 --log-level info
```

**In another terminal, run the load test:**
```bash
python3 load_test.py
```

### What It Tests

The Python script runs 7 comprehensive tests:

1. **Cache Warm-up** - Populates cache with real domains (google.com, github.com, etc.)
2. **Cache Hits** - Tests performance when queries hit the cache
3. **Low Concurrency** - 10 unique domains, 5 concurrent queries
4. **Medium Concurrency** - 50 unique domains, 25 concurrent queries
5. **High Concurrency** - 100 unique domains, 50 concurrent queries (stress test)
6. **Mixed Query Types** - Tests both A and NS records simultaneously
7. **Sustained Load** - 3 rounds of repeated queries to same domains

### Output Example

```
============================================================
Test: Cache Hits (Repeated queries)
============================================================
Total Queries:        100
Successful:           100 (100.0%)
Failed:               0
Cached Hits:          95 (95.0% of successful)

Performance Metrics:
  Min Response Time:   0.42 ms
  Max Response Time:   8.32 ms
  Avg Response Time:   1.23 ms
  P99 Response Time:   4.15 ms
  Throughput:          81,301 queries/sec
============================================================
```

### Results File

Results are automatically saved to `load_test_results_YYYYMMDD_HHMMSS.json` with all statistics.

---

## Interpreting Results

### Key Metrics

| Metric | What It Means | Target |
|--------|--------------|--------|
| **Success Rate** | % of queries that completed | >99% |
| **Throughput (QPS)** | Queries per second | >1000 |
| **Avg Response Time** | Average query latency | <10ms |
| **P99 Response Time** | 99th percentile latency | <50ms |
| **Cache Hit Rate** | % of queries hitting cache | >80% (after warm-up) |

### Performance Expectations

**Without Caching:**
- Throughput: 50-100 QPS
- Latency: 100-500ms

**With Caching:**
- Cache Hits: 50,000+ QPS
- Latency: 1-5ms

**Stress Test (High Concurrency):**
- Should maintain >90% success rate under 50 concurrent connections
- Throughput should scale with CPU cores

---

## Customizing Tests

### Python Script

Edit `load_test.py` to modify:

```python
# Change target server
tester = DNSLoadTester(host="127.0.0.1", port=2053, timeout=10)

# Change test domains
test_domains = ["mynewdomain.com", "example.org"]

# Change concurrency levels
results = await tester.run_concurrent_queries(domains, concurrency=100)

# Add new test
domains = tester.generate_test_domains(count=500)
results = await tester.run_concurrent_queries(domains, concurrency=100)
```

### Common Adjustments

**For high-load testing:**
```python
# Increase concurrency to 200
concurrency=200

# Generate more test domains
tester.generate_test_domains(count=1000)
```

**For network testing:**
```python
# Increase timeout
tester.timeout = 30  # 30 seconds
```

---

## Troubleshooting

### Connection Refused
```bash
# Ensure server is running on port 2053
lsof -i :2053

# Or check with dig
dig @127.0.0.1 -p 2053 google.com
```

### Slow Response Times
- Check server CPU usage: `top`
- Check DNS recursive resolver availability
- Try smaller concurrency level first

### Many Failures
- Increase timeout:
  ```python
  tester = DNSLoadTester(timeout=30)
  ```
- Verify DNS resolver can reach root nameservers

### DNS Resolution Failures
```bash
# Test if dig works
dig @127.0.0.1 -p 2053 google.com

# Try with longer timeout
dig @127.0.0.1 -p 2053 +timeout=20 google.com
```

---

## Performance Tuning

### Server-side Optimization
```bash
# Increase cache size for better hit rates
./target/release/dns_server --cache-size 10000

# Run with multiple worker threads
./target/release/dns_server --worker-threads 8
```

### Load Tester Optimization
```python
# Decrease concurrency if server is bottlenecked
concurrency=25

# Increase for max throughput testing
concurrency=200
```

---

## Next Steps

After running load tests:

1. **Analyze hit rates** - Adjust cache size if hits < 80%
2. **Monitor latency** - If P99 > 50ms, check DNS resolver config
3. **Check throughput** - Compare against target QPS
4. **Profile CPU** - Use `perf` or system monitor during load test
5. **Stress test** - Gradually increase concurrency to find breaking point

```bash
# Example: stress test with increasing load
for concurrency in 10 25 50 100 200; do
  echo "Testing with concurrency: $concurrency"
  python3 load_test.py --concurrency $concurrency
done
```
