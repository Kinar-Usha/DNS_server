use crate::protocol::{QueryType,DnsQuestion,DnsPacket,DnsHeader};
use crate::buffer::BytePacketBuffer;

use std::net::{Ipv4Addr, Ipv6Addr};
use std::net::UdpSocket;

type Error = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Error>;



fn lookup(qname: &str, qtype : QueryType, server: (Ipv4Addr, u16)) -> Result<DnsPacket> {
    let socket = UdpSocket::bind(("0.0.0.0", 24130))?;
    let mut packet = DnsPacket::new();

    packet.header.id = 6969;
    packet.header.questions =1;
    packet.header.recursion_desired =true;
    packet.questions.push(DnsQuestion::new(qname.to_string(),qtype));


    let mut request_buffer = BytePacketBuffer::new();
    packet.write(&mut request_buffer)?;
    socket.send_to(&request_buffer.buf[0..request_buffer.pos], server)?;

    let mut result_buffer = BytePacketBuffer::new();
    socket.recv_from(&mut result_buffer.buf)?;
    DnsPacket::from_buffer(&mut result_buffer)
}



