use std::sync::Arc;
use std::time::Duration;
use tokio::net::UdpSocket;
use tokio::time::interval;
mod buffer;
mod cache;
mod config;
mod logging;
mod protocol;
mod resolve;

use cache::lru::LruDnsCache;
use cache::DnsCache;
use config::Config;

#[tokio::main]
async fn main() {
    // Parse and validate CLI arguments
    let config = Config::parse_and_validate();
    
    // Initialize flexi_logger to log to console AND file
    let _logger = flexi_logger::Logger::try_with_str(&config.log_level)
        .unwrap()
        .log_to_file(
            flexi_logger::FileSpec::default()
                .directory("logs")
                .basename("dns_server"),
        )
        .duplicate_to_stderr(flexi_logger::Duplicate::All)
        .rotate(
            flexi_logger::Criterion::Age(flexi_logger::Age::Day),
            flexi_logger::Naming::Timestamps,
            flexi_logger::Cleanup::KeepLogFiles(7),
        )
        .start()
        .unwrap();

    // Print startup configuration
    config.print_startup_info();

    // Initialize LRU cache with configured size
    let cache = Arc::new(LruDnsCache::new(config.cache_size));
    log::info!("DNS cache initialized with size: {}", config.cache_size);

    let addr = config.socket_addr();
    let socket = match UdpSocket::bind(&addr).await {
        Ok(s) => {
            eprintln!("✓ DNS server listening on {}", addr);
            Arc::new(s)
        }
        Err(e) => {
            eprintln!("✗ Failed to bind socket to {}: {}", addr, e);
            eprintln!("  Make sure the port {} is not already in use and you have permission to bind it.", config.port);
            return;
        }
    };

    // Spawn cache stats logging task
    let cache_clone = cache.clone();
    tokio::spawn(async move {
        let mut stats_interval = interval(Duration::from_secs(60));
        loop {
            stats_interval.tick().await;
            cache_clone.get_stats().log();
        }
    });

    // Spawn cache cleanup task (runs every 30 seconds)
    let cache_cleanup = cache.clone();
    tokio::spawn(async move {
        let mut cleanup_interval = interval(Duration::from_secs(30));
        loop {
            cleanup_interval.tick().await;
            let removed = cache_cleanup.cleanup();
            if removed > 0 {
                log::info!("Cache cleanup: removed {} expired entries, cache size now: {}", removed, cache_cleanup.size());
            }
        }
    });

    // Use a semaphore to limit concurrent query tasks
    let semaphore = Arc::new(tokio::sync::Semaphore::new(1000));

    loop {
        let mut request_buffer = buffer::BytePacketBuffer::new();
        let (len, src) = match socket.recv_from(&mut request_buffer.buf).await {
            Ok(res) => res,
            Err(e) => {
                log::error!("Failed to receive DNS query from client: {}", e);
                continue;
            }
        };
        request_buffer.pos = 0; // Reset position to 0 for reading

        let socket = socket.clone();
        let cache = cache.clone();
        let sem = semaphore.clone();

        tokio::spawn(async move {
            let _permit = match sem.acquire().await {
                Ok(p) => p,
                Err(_) => return,
            };

            match resolve::handle_query(&socket, request_buffer, src, Some(cache)).await {
                Ok(_) => {
                    log::debug!("DNS query processed and response sent successfully");
                }
                Err(e) => {
                    log::error!("Query failed: {}", e);
                }
            }
        });
    }
}
