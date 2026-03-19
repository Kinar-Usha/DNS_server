use std::sync::Arc;
use tokio::net::UdpSocket;
mod buffer;
mod protocol;
mod resolve;

#[tokio::main]
async fn main() {
    env_logger::Builder::from_default_env()
        .init();

    let addr = "0.0.0.0:2053";
    let socket = match UdpSocket::bind(addr).await {
        Ok(s) => {
            eprintln!("✓ DNS server listening on {}", addr);
            Arc::new(s)
        }
        Err(e) => {
            eprintln!("✗ Failed to bind socket to {}: {}", addr, e);
            eprintln!("  Make sure the port 2053 is not already in use and you have permission to bind it.");
            return;
        }
    };

    loop {
        match resolve::handle_query(&socket).await {
            Ok(_) => {
                log::debug!("Query handled successfully");
            }
            Err(e) => {
                eprintln!("⚠ Query failed: {}", e);
            }
        }
    }
}
