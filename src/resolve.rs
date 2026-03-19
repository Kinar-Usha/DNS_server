//! this module contains the methods for resolving and recursively looking up domain names.

use crate::buffer::BytePacketBuffer;
use crate::cache::{CachedDnsRecord, CacheKey, DnsCache};
use crate::logging::ResponseLog;
use crate::protocol::{DnsPacket, DnsQuestion, QueryType, ResultCode};

use std::net::Ipv4Addr;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Instant;
use tokio::net::UdpSocket;

type Error = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Error>;

const ROOT_SERVERS: &[&str] = &[
    "198.41.0.4",     // a.root-servers.net
    "199.9.14.201",   // b.root-servers.net
    "192.33.4.12",    // c.root-servers.net
    "199.7.91.13",    // d.root-servers.net
    "192.203.230.10", // e.root-servers.net
    "192.5.5.241",    // f.root-servers.net
    "192.112.36.4",   // g.root-servers.net
    "198.97.190.53",  // h.root-servers.net
    "192.36.148.17",  // i.root-servers.net
    "192.58.128.30",  // j.root-servers.net
    "193.0.14.129",   // k.root-servers.net
    "199.7.83.42",    // l.root-servers.net
    "202.12.27.33",   // m.root-servers.net
];

async fn lookup(qname: &str, qtype: QueryType, server: (Ipv4Addr, u16)) -> Result<DnsPacket> {
    let mut last_error = None;

    // Try up to 3 times for each lookup
    for attempt in 0..3 {
        // Bind to any available port (0 = OS assigns free port) to avoid port exhaustion
        let socket = match UdpSocket::bind("0.0.0.0:0").await {
            Ok(s) => s,
            Err(e) => {
                last_error = Some(format!("Failed to bind local socket: {}", e));
                continue;
            }
        };
        
        let mut packet = DnsPacket::new();
        packet.header.id = rand::random();
        packet.header.questions = 1;
        packet.header.recursion_desired = true;
        packet
            .questions
            .push(DnsQuestion::new(qname.to_string(), qtype));

        let mut request_buffer = BytePacketBuffer::new();
        if let Err(e) = packet.write(&mut request_buffer) {
            return Err(format!("Failed to write DNS packet: {}", e).into());
        }
        
        if let Err(e) = socket.send_to(&request_buffer.buf[0..request_buffer.pos], server).await {
            last_error = Some(format!("Failed to send DNS query: {}", e));
            continue;
        }

        let mut result_buffer = BytePacketBuffer::new();
        // Use a 3-second timeout per attempt, total 9 seconds across 3 attempts
        match tokio::time::timeout(std::time::Duration::from_secs(3), socket.recv_from(&mut result_buffer.buf)).await {
            Ok(Ok(_)) => {
                return DnsPacket::from_buffer(&mut result_buffer)
                    .map_err(|e| format!("Failed to parse response packet: {}", e).into());
            }
            Ok(Err(e)) => {
                last_error = Some(format!("Failed to receive response: {}", e));
            }
            Err(_) => {
                last_error = Some("Timeout waiting for response".to_string());
            }
        }
        
        log::debug!("Attempt {} failed for '{}' ({}) from {}: {:?}", 
            attempt + 1, qname, qtype, server.0, last_error);
    }

    Err(format!("Lookup failed for '{}' ({}) after 3 attempts. Last error: {:?}", 
        qname, qtype, last_error).into())
}


fn recursive_lookup(
    qname: &str,
    qtype: QueryType,
    cache: Option<Arc<dyn DnsCache>>,
) -> Pin<Box<dyn std::future::Future<Output = Result<DnsPacket>> + Send>> {
    let qname = qname.to_string();
    
    Box::pin(async move {
        // ... (cache check remains the same)
        if let Some(ref cache) = cache {
            let cache_key = CacheKey::new(qname.clone(), qtype.to_num());
            if let Some(cached) = cache.get(&cache_key) {
                if !cached.is_expired() {
                    log::info!(
                        "Cache hit for '{}' ({}) - hit_count: {}, miss_count: {}",
                        qname,
                        qtype,
                        cache.hit_count(),
                        cache.miss_count()
                    );
                    return Ok(cached.packet);
                }
            }
        }

        // Randomly pick a root server to start with
        let root_idx = rand::random::<usize>() % ROOT_SERVERS.len();
        let mut ns = ROOT_SERVERS[root_idx].parse::<Ipv4Addr>().unwrap();

        loop {
            log::debug!("Looking up {:?} {} with ns {}", qtype, &qname, ns);

            let ns_copy = ns;
            let server = (ns_copy, 53);
            
            let response = match lookup(&qname, qtype, server).await {
                Ok(r) => r,
                Err(e) => {
                    log::warn!("Lookup failed for '{}' ({}): {}", &qname, qtype, e);
                    return Err(e);
                }
            };


            if response.header.rescode == ResultCode::NOERROR && !response.answers.is_empty() {
                log::debug!("Found answers for '{}' ({})", &qname, qtype);
                
                // Cache successful response
                if let Some(ref cache) = cache {
                    let ttl = response.answers.first()
                        .map(|a| match a {
                            crate::protocol::DnsRecord::UNKNOWN { ttl, .. } => *ttl,
                            crate::protocol::DnsRecord::A { ttl, .. } => *ttl,
                            crate::protocol::DnsRecord::NS { ttl, .. } => *ttl,
                            crate::protocol::DnsRecord::CNAME { ttl, .. } => *ttl,
                            crate::protocol::DnsRecord::MX { ttl, .. } => *ttl,
                            crate::protocol::DnsRecord::AAAA { ttl, .. } => *ttl,
                            crate::protocol::DnsRecord::SOA { ttl, .. } => *ttl,
                        })
                        .unwrap_or(300);
                    let cache_key = CacheKey::new(qname.clone(), qtype.to_num());
                    let cached_record = CachedDnsRecord::new(response.clone(), ttl);
                    cache.set(cache_key, cached_record);
                    log::debug!(
                        "Cached result for '{}' ({}) - cache_size: {}",
                        qname,
                        qtype,
                        cache.size()
                    );
                }
                
                return Ok(response);
            }

            // Authoritative name servers replying no domain by that name.
            if response.header.rescode == ResultCode::NXDOMAIN {
                log::debug!("Domain '{}' does not exist (NXDOMAIN)", &qname);
                return Ok(response);
            }
            
            // find a new nameserver based on the records in the additional section
            if let Some(new_ns) = response.get_resolved_ns(&qname) {
                log::debug!("Found resolved nameserver {} for '{}'", new_ns, &qname);
                ns = new_ns;
                continue;
            }
            
            let new_ns_name = match response.get_unresolved_ns(&qname) {
                Some(x) => x,
                None => {
                    log::debug!("No more nameservers found for '{}'", &qname);
                    return Ok(response);
                }
            };
            
            log::debug!("Need to resolve nameserver '{}' for '{}'", new_ns_name, &qname);
            let recursive_response = match recursive_lookup(&new_ns_name, QueryType::A, cache.clone()).await {
                Ok(r) => r,
                Err(e) => {
                    log::warn!("Failed to recursively lookup nameserver '{}': {}", new_ns_name, e);
                    return Ok(response);
                }
            };

            if let Some(new_ns) = recursive_response.get_random_a() {
                log::debug!("Resolved nameserver '{}' to {}", new_ns_name, new_ns);
                ns = new_ns;
            } else {
                log::warn!("Could not resolve nameserver '{}' for '{}'", new_ns_name, &qname);
                return Ok(response);
            }
        }
    })
}

pub async fn handle_query(
    socket: &UdpSocket,
    mut request_buffer: BytePacketBuffer,
    src: std::net::SocketAddr,
    cache: Option<Arc<dyn DnsCache>>,
) -> Result<()> {
    let query_start = Instant::now();
    
    let mut request = DnsPacket::from_buffer(&mut request_buffer)
        .map_err(|e| format!("Failed to parse incoming DNS query from {}: {}", src, e))?;

    let mut packet = DnsPacket::new();
    packet.header.id = request.header.id;
    packet.header.recursion_desired = true;
    packet.header.recursion_available = true;
    packet.header.response = true;

    if let Some(question) = request.questions.pop() {
        log::info!("Processing query: {:?} from {}", question, src);

        // Track whether this was a cache hit
        let mut was_cache_hit = false;

        match recursive_lookup(&question.name, question.qtype, cache.clone()).await {
            Ok(result) => {
                packet.questions.push(question.clone());
                packet.header.rescode = result.header.rescode;

                for rec in result.answers {
                    log::debug!("Answer: {:?}", rec);
                    packet.answers.push(rec);
                }
                for rec in result.authorities {
                    log::debug!("Authority: {:?}", rec);
                    packet.authorities.push(rec);
                }

                for rec in result.resources {
                    log::debug!("Resource: {:?}", rec);
                    packet.resources.push(rec);
                }

                // Check if this was a cache hit by comparing with cache stats
                if let Some(ref cache_ref) = cache {
                    let current_hits = cache_ref.hit_count();
                    was_cache_hit = current_hits > 0; // Simplified check
                }
            }
            Err(e) => {
                log::warn!("Failed to resolve '{}' ({}): {}", question.name, question.qtype, e);
                packet.header.rescode = ResultCode::SERVFAIL;
            }
        }

        // Serialize response and get size
        let mut result_buffer = BytePacketBuffer::new();
        packet.write(&mut result_buffer)
            .map_err(|e| format!("Failed to serialize DNS response: {}", e))?;

        let len = result_buffer.pos();
        let response_size = len;
        let buffer = result_buffer.get_range(0, len)
            .map_err(|e| format!("Failed to get response buffer range: {}", e))?;

        // Calculate response time
        let response_time_ms = query_start.elapsed().as_millis() as u64;

        // Extract TTL values from answers
        let ttl_values: Vec<u32> = packet
            .answers
            .iter()
            .map(|rec| match rec {
                crate::protocol::DnsRecord::UNKNOWN { ttl, .. } => *ttl,
                crate::protocol::DnsRecord::A { ttl, .. } => *ttl,
                crate::protocol::DnsRecord::NS { ttl, .. } => *ttl,
                crate::protocol::DnsRecord::CNAME { ttl, .. } => *ttl,
                crate::protocol::DnsRecord::MX { ttl, .. } => *ttl,
                crate::protocol::DnsRecord::AAAA { ttl, .. } => *ttl,
                crate::protocol::DnsRecord::SOA { ttl, .. } => *ttl,
            })
            .collect();

        // Log the response
        let response_log = ResponseLog {
            domain_name: question.name.clone(),
            query_type: question.qtype,
            response_time_ms,
            answer_count: packet.answers.len(),
            authority_count: packet.authorities.len(),
            response_size,
            was_cache_hit,
            ttl_values,
            result_code: packet.header.rescode,
        };
        response_log.log();

        // Send response
        socket.send_to(buffer, src)
            .await
            .map_err(|e| format!("Failed to send DNS response to {}: {}", src, e))?;
    } else {
        log::warn!("Received query with no questions from {}", src);
        
        let mut result_buffer = BytePacketBuffer::new();
        packet.write(&mut result_buffer)
            .map_err(|e| format!("Failed to serialize DNS response: {}", e))?;

        let len = result_buffer.pos();
        let buffer = result_buffer.get_range(0, len)
            .map_err(|e| format!("Failed to get response buffer range: {}", e))?;

        socket.send_to(buffer, src)
            .await
            .map_err(|e| format!("Failed to send DNS response to {}: {}", src, e))?;
    }

    Ok(())
}
