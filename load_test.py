#!/usr/bin/env python3
"""
DNS Server Load Testing Script

Measures DNS server performance under concurrent load:
- Throughput (queries per second)
- Response latency (min, max, average, p99)
- Cache hit rates
- Error rates

Requirements:
    pip install dnspython
"""

import asyncio
import dns.rdatatype
import dns.resolver
import time
import statistics
import sys
from dataclasses import dataclass
from typing import List, Tuple
from datetime import datetime
import json


@dataclass
class QueryResult:
    domain: str
    qtype: str
    success: bool
    response_time_ms: float
    error: str = None
    cached: bool = False


class DNSLoadTester:
    def __init__(self, host: str = "127.0.0.1", port: int = 2053, timeout: int = 5):
        self.host = host
        self.port = port
        self.timeout = timeout
        self.resolver = dns.resolver.Resolver()
        self.resolver.nameservers = [host]
        self.resolver.port = port
        self.resolver.timeout = timeout
        self.resolver.lifetime = timeout
        self.results: List[QueryResult] = []

    async def query_dns(self, domain: str, qtype: str = "A") -> QueryResult:
        """Perform a single DNS query and measure time"""
        start = time.perf_counter()
        
        try:
            answers = self.resolver.resolve(domain, qtype)
            elapsed_ms = (time.perf_counter() - start) * 1000
            
            return QueryResult(
                domain=domain,
                qtype=qtype,
                success=True,
                response_time_ms=elapsed_ms,
                cached=elapsed_ms < 5  # Heuristic: responses < 5ms are likely cached
            )
        except Exception as e:
            print(f"Error querying {domain} ({qtype}): {e}")
            elapsed_ms = (time.perf_counter() - start) * 1000
            return QueryResult(
                domain=domain,
                qtype=qtype,
                success=False,
                response_time_ms=elapsed_ms,
                error=str(e)
            )

    async def run_concurrent_queries(self, domains: List[str], qtype: str = "A", concurrency: int = 10) -> List[QueryResult]:
        """Run multiple DNS queries concurrently"""
        semaphore = asyncio.Semaphore(concurrency)
        
        async def bounded_query(domain: str) -> QueryResult:
            async with semaphore:
                return await self.query_dns(domain, qtype)
        
        tasks = [bounded_query(domain) for domain in domains]
        return await asyncio.gather(*tasks)

    def generate_test_domains(self, count: int = 100) -> List[str]:
        """Generate test domain names using real popular domains"""
        popular_domains = [
            "google.com", "youtube.com", "facebook.com", "wikipedia.org",
            "twitter.com", "amazon.com", "instagram.com", "linkedin.com",
            "reddit.com", "netflix.com", "microsoft.com", "apple.com",
            "yahoo.com", "github.com", "cloudflare.com", "openai.com",
            "zoom.us", "twitch.tv", "bing.com", "office.com",
            "pinterest.com", "ebay.com", "booking.com", 
            "duckduckgo.com", "stackoverflow.com", "spotify.com", "adobe.com",
            "nytimes.com", "cnn.com", "bbc.co.uk", "theguardian.com",
            "imdb.com", "quora.com", "medium.com", "discord.com",
            "dropbox.com", "vimeo.com", "etsy.com", "paypal.com",
            "salesforce.com", "slack.com", "hubspot.com", "shopify.com",
            "trello.com", "canva.com", "notion.so", "figma.com",
            "stripe.com", "digitalocean.com"
        ]
        
        # If we need more domains than available, cycle through them
        from itertools import cycle, islice
        return list(islice(cycle(popular_domains), count))

    def print_statistics(self, results: List[QueryResult], test_name: str):
        """Print detailed statistics about query results"""
        successful = [r for r in results if r.success]
        failed = [r for r in results if not r.success]
        cached = [r for r in results if r.cached]
        
        success_rate = (len(successful) / len(results)) * 100 if results else 0
        cache_hit_rate = (len(cached) / len(successful)) * 100 if successful else 0
        
        if successful:
            times = [r.response_time_ms for r in successful]
            avg_time = statistics.mean(times)
            min_time = min(times)
            max_time = max(times)
            p99_time = statistics.quantiles(times, n=100)[98] if len(times) > 1 else min_time
            total_time = sum(times)
            qps = (len(results) / total_time) * 1000 if total_time > 0 else 0
        else:
            avg_time = min_time = max_time = p99_time = qps = 0

        duration = sum([r.response_time_ms for r in results]) / 1000

        print(f"\n{'='*60}")
        print(f"Test: {test_name}")
        print(f"{'='*60}")
        print(f"Total Queries:        {len(results)}")
        print(f"Successful:           {len(successful)} ({success_rate:.1f}%)")
        print(f"Failed:               {len(failed)}")
        print(f"Cached Hits:          {len(cached)} ({cache_hit_rate:.1f}% of successful)")
        print(f"\nPerformance Metrics:")
        print(f"  Min Response Time:   {min_time:.2f} ms")
        print(f"  Max Response Time:   {max_time:.2f} ms")
        print(f"  Avg Response Time:   {avg_time:.2f} ms")
        print(f"  P99 Response Time:   {p99_time:.2f} ms")
        print(f"  Throughput:          {qps:.0f} queries/sec")
        print(f"{'='*60}\n")

        return {
            "test_name": test_name,
            "total_queries": len(results),
            "successful": len(successful),
            "success_rate": success_rate,
            "failed": len(failed),
            "cached_hits": len(cached),
            "cache_hit_rate": cache_hit_rate,
            "min_response_ms": min_time,
            "max_response_ms": max_time,
            "avg_response_ms": avg_time,
            "p99_response_ms": p99_time,
            "throughput_qps": qps,
            "total_duration_sec": duration
        }


async def main():
    """Run comprehensive load tests"""
    print(f"\n🚀 DNS Server Load Testing - Started at {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
    print(f"Target: http://127.0.0.1:2053")
    
    tester = DNSLoadTester(host="127.0.0.1", port=2053, timeout=10)
    all_stats = []

    # Test 1: Cache warm-up (real domains)
    print("\n📍 Test 1: Cache Warm-up (Real domains)")
    real_domains = ["google.com", "github.com", "stackoverflow.com", "cloudflare.com", "amazon.com"]
    results = await tester.run_concurrent_queries(real_domains * 2, concurrency=5)
    stats = tester.print_statistics(results, "Cache Warm-up")
    all_stats.append(stats)

    # Test 2: Cache hits (repeated domains)
    print("📍 Test 2: Cache Hits (Repeated queries)")
    repeated_domains = ["google.com", "github.com"] * 50
    results = await tester.run_concurrent_queries(repeated_domains, concurrency=20)
    stats = tester.print_statistics(results, "Cache Hits")
    all_stats.append(stats)

    # Test 3: Low concurrency baseline
    print("📍 Test 3: Low Concurrency (10 unique domains, 5 concurrent)")
    domains = tester.generate_test_domains(count=10)
    results = await tester.run_concurrent_queries(domains, concurrency=5)
    stats = tester.print_statistics(results, "Low Concurrency")
    all_stats.append(stats)

    # Test 4: Medium concurrency
    print("📍 Test 4: Medium Concurrency (50 unique domains, 25 concurrent)")
    domains = tester.generate_test_domains(count=50)
    results = await tester.run_concurrent_queries(domains, concurrency=25)
    stats = tester.print_statistics(results, "Medium Concurrency")
    all_stats.append(stats)

    # Test 5: High concurrency (stress test)
    print("📍 Test 5: High Concurrency (100 unique domains, 50 concurrent)")
    domains = tester.generate_test_domains(count=100)
    results = await tester.run_concurrent_queries(domains, concurrency=50)
    stats = tester.print_statistics(results, "High Concurrency")
    all_stats.append(stats)

    # # Test 6: Mixed query types
    # print("📍 Test 6: Mixed Query Types (A and NS records, 30 concurrent)")
    # a_records = tester.generate_test_domains(count=25)
    # ns_records = tester.generate_test_domains(count=25)
    
    # results_a = await tester.run_concurrent_queries(a_records, qtype="A", concurrency=15)
    # results_ns = await tester.run_concurrent_queries(ns_records, qtype="NS", concurrency=15)
    # combined_results = results_a + results_ns
    # stats = tester.print_statistics(combined_results, "Mixed Query Types")
    # all_stats.append(stats)

    # Test 7: Sustained load (follow-up queries to test cache)
    print("📍 Test 7: Sustained Load (re-query same 25 domains)")
    domains = tester.generate_test_domains(count=25)
    for i in range(3):
        print(f"   Round {i+1}/3...")
        results = await tester.run_concurrent_queries(domains, concurrency=25)
        stats = tester.print_statistics(results, f"Sustained Load Round {i+1}")
        all_stats.append(stats)

    # Summary
    print("\n" + "="*60)
    print("📊 LOAD TEST SUMMARY")
    print("="*60)
    
    total_queries = sum(s["total_queries"] for s in all_stats)
    total_successful = sum(s["successful"] for s in all_stats)
    total_failed = sum(s["failed"] for s in all_stats)
    avg_success_rate = sum(s["success_rate"] for s in all_stats) / len(all_stats)
    max_throughput = max(s["throughput_qps"] for s in all_stats)
    avg_response_time = sum(s["avg_response_ms"] for s in all_stats) / len(all_stats)
    
    print(f"Total Queries:           {total_queries}")
    print(f"Total Successful:        {total_successful} ({(total_successful/total_queries*100):.1f}%)")
    print(f"Total Failed:            {total_failed}")
    print(f"Average Success Rate:    {avg_success_rate:.1f}%")
    print(f"Peak Throughput:         {max_throughput:.0f} queries/sec")
    print(f"Average Response Time:   {avg_response_time:.2f} ms")
    print("="*60)

    # Save results to JSON
    output_file = f"load_test_results_{datetime.now().strftime('%Y%m%d_%H%M%S')}.json"
    with open(output_file, 'w') as f:
        json.dump(all_stats, f, indent=2)
    
    print(f"\n✅ Results saved to: {output_file}")
    print(f"✅ Load testing completed at {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}\n")


if __name__ == "__main__":
    try:
        asyncio.run(main())
    except KeyboardInterrupt:
        print("\n\n⚠️  Load test interrupted by user")
        sys.exit(0)
    except Exception as e:
        print(f"\n\n❌ Error: {e}")
        sys.exit(1)
