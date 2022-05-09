use crate::protocol::{QueryType,DnsQuestion,DnsPacket,DnsHeader, ResultCode};
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

fn recursive_lookup(qname: &str,qtype : QueryType) -> Result<DnsPacket> {
    let mut ns = "198.41.0.4".parse::<Ipv4Addr>().unwrap();

    loop {
        println!("Looking up {:?} {}  with ns {} ", qtype, qname, ns);

        let ns_copy = ns;
        let server = (ns_copy, 53);
        let response = lookup(qname, qtype, server)?;
        if response.header.rescode == ResultCode::NOERROR  && !response.answers.is_empty() {
            return Ok(response);
        }
        if response.header.rescode == ResultCode::NXDOMAIN {
            return Ok(response);
        }

        if let Some(new_ns) = response.get_resolved_ns(qname) {
            ns = new_ns;
            continue;
        }
        let new_ns_name = match response.get_unresolved_ns(qname) {
            Some(x)=> x,
            None => return Ok(response),
        };
        let recursive_response = recursive_lookup(&new_ns_name, QueryType::A)?;

        if let Some(new_ns) = recursive_response.get_random_a() {
            ns = new_ns;
        }
        else {
            return Ok(response);
        }
    }
}


pub fn handle_query(socket: &UdpSocket) -> Result<()> {
    let mut request_buffer = BytePacketBuffer::new();
    let (_, src) = socket.recv_from(&mut request_buffer.buf)?;
    let mut request = DnsPacket::from_buffer(&mut request_buffer)?;
    
    let mut packet = DnsPacket::new();
    packet.header.id = request.header.id;
    packet.header.recursion_desired = true;
    packet.header.recursion_available = true;
    packet.header.response = true;

    if let Some(question) = request.questions.pop() {
        println!("Query = {:?}", question);

        if let Ok(result) = recursive_lookup (&question.name, question.qtype){
            packet.questions.push(question.clone());
            packet.header.rescode = result.header.rescode;

            for rec in result.answers {
                println!("Answer: {:?} ", rec);
                packet.answers.push(rec);
            }
            for rec in result.authorities {
                println!("Authorities : {:?}",rec);
                packet.authorities.push(rec);
            }

            for rec in result.resources {
                println!("Resource: {:?}", rec);
                packet.resources.push(rec);
            }
        
        }
        else {
            packet.header.rescode = ResultCode::SERVFAIL;
        }
    }
    else {
        packet.header.rescode = ResultCode::FORMERR;
    }

    let mut result_buffer = BytePacketBuffer::new();
    packet.write(&mut result_buffer)?;



    let len = result_buffer.pos();
    let buffer = result_buffer.get_range(0,  len)?;

    socket.send_to(buffer, src)?;
    
    
    Ok(())
}