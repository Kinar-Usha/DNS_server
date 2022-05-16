use std::net::UdpSocket;
mod buffer;
mod protocol;
mod resolve;

type Error = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Error>;

fn main() -> Result<()> {
    let socket = UdpSocket::bind(("0.0.0.0", 2053))?;

    loop {
        match resolve::handle_query(&socket) {
            Ok(_) => {}
            Err(e) => eprintln!("Error occured ; context: {}", e),
        }
    }
}
