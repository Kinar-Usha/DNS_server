use std::net::{Ipv4Addr, Ipv6Addr};
use crate::buffer::BytePacketBuffer;
type Error = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Error>;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ResultCode {
    NOERROR = 0,
    FORMERR = 1,
    SERVFAIL = 2,
    NXDOMAIN =3,
    NOTIMP = 4,
    REFUSED = 5,
}

impl ResultCode {
    pub fn from_num(num: u8) -> ResultCode {
        match num {
            1 => ResultCode::FORMERR,
            2 => ResultCode::SERVFAIL,
            3 => ResultCode::NXDOMAIN,
            4 => ResultCode::NOTIMP,
            5 => ResultCode::REFUSED,
            0 | _ => ResultCode::NOERROR,
        }
    }
}

pub struct DnsHeader { 
    pub id: u16,
    pub recursion_desired: bool,
    pub truncated_message: bool,
    pub authoritative_answer:bool,
    pub opcode: u8,
    pub response:bool,

    pub rescode: ResultCode,
    pub checking_disabled:bool,
    pub authed_data: bool,
    pub z: bool,
    pub recursion_available:bool,


    pub questions:u16,
    pub answers: u16,
    pub authoritative_entries: u16,
    pub resource_entries: u16,

}

impl DnsHeader {
    pub fn new() -> DnsHeader{
        DnsHeader { id: 0, recursion_desired: false, truncated_message: false, authoritative_answer: false, opcode: 0, response: false, rescode: ResultCode::NOERROR, checking_disabled: false, authed_data: false, z: false, recursion_available: false, questions: 0, answers: 0, authoritative_entries: 0, resource_entries: 0 }
    }

    pub fn read(&mut self, buffer: &mut BytePacketBuffer) -> Result<()>{
        self.id = buffer.read_u16()?;

        let flags = buffer.read_u16()?;
        let a = (flags >> 8) as u8;
        let b = (flags & 0xFF) as u8;
        self.recursion_desired = (a & (1<<0)) > 0;
        self.truncated_message = (a & (1<<1)) > 0;
        self.authoritative_answer = (a & (1<<2)) >0;
        self.opcode = (a >> 3) & 0x0F;
        self.response = (a & (1<<7))> 0;

        self.rescode = ResultCode::from_num(b & 0x0F);
        self.checking_disabled = (b &(1<< 4)) > 0;
        self.authed_data = (b& (1<<5))> 0;
        self.z = (b & (1<< 6)) > 0;
        self.recursion_available = (b & (1<< 7)) >0;
        
        
        self.questions = buffer.read_u16()?;
        self.answers = buffer.read_u16()?;
        self.authoritative_entries = buffer.read_u16()?;
        self.resource_entries = buffer.read_u16()?;
        
        Ok(())
    }
    
    pub fn write(&self, buffer: &mut BytePacketBuffer) -> Result<()> {
        buffer.write_u16(self.id)?;
        
        
        buffer.write_u8(
            (self.recursion_desired as u8)
                | ((self.truncated_message as u8) << 1)
                | ((self.authoritative_answer as u8) << 2)
                | (self.opcode << 3)
                | ((self.response as u8) << 7) as u8,
        )?;

        buffer.write_u8(
            (self.rescode as u8)
                | ((self.checking_disabled as u8) << 4)
                | ((self.authed_data as u8) << 5)
                | ((self.z as u8) << 6)
                | ((self.recursion_available as u8) << 7),
        )?;
        buffer.write_u16(self.questions)?;
        buffer.write_u16(self.answers)?;
        buffer.write_u16(self.authoritative_entries)?;
        buffer.write_u16(self.resource_entries)?;

        Ok(())
    }
}

#[derive(PartialEq,Eq,Debug,Clone,Hash,Copy)]
pub enum QueryType {
    UNKNOWN(u16),
    A,
    NS,
    CNAME,
    MX,
    AAAA,

}
impl  QueryType {
    pub fn to_num(&self) -> u16{
        match *self {
            QueryType::UNKNOWN(x) => x,
            QueryType::A => 1,
            QueryType::NS =>2,
            QueryType::CNAME =>5,
            QueryType::MX => 15,
            QueryType::AAAA =>28,
        }
    }

    pub fn from_num(num: u16) -> QueryType{
        match num {
            1 => QueryType::A,
            2 => QueryType::NS,
            5 => QueryType::CNAME,
            15 => QueryType::MX,
            28 => QueryType::AAAA,
            _ => QueryType::UNKNOWN(num),
        }
    }
}

#[derive(Debug,Clone,PartialEq,Eq)]
pub struct DnsQuestion {
    pub name: String,
    pub qtype: QueryType,
}

impl DnsQuestion{
    pub fn new(name: String, qtype: QueryType) -> DnsQuestion{
        DnsQuestion { name: name, qtype: qtype }
    }
    pub fn read(&mut self, buffer: &mut BytePacketBuffer) -> Result<()> {
        buffer.read_qname(&mut self.name)?;
        self.qtype =QueryType::from_num(buffer.read_u16()?);
        let _ =buffer.read_u16()?;

        Ok(())
    }

    pub fn write(&self, buffer: &mut BytePacketBuffer) -> Result<()> {
        buffer.write_qname(&self.name)?;
        let type_num = self.qtype.to_num();
        buffer.write_u16(type_num)?;
        buffer.write_u16(1)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum DnsRecord{
    UNKNOWN {
        domain: String,
        qtype: u16,
        data_len: u16,
        ttl: u32,
    },
    A {
        domain: String,
        addr: Ipv4Addr,
        ttl: u32,
    },
    NS {
        domain: String,
        host: String,
        ttl: u32,
    },
    CNAME {
        domain: String,
        host: String,
        ttl: u32,
    },
    MX {
        domain: String,
        priority: u16,
        host: String,
        ttl: u32,
    },
    AAAA {
        domain: String,
        addr: Ipv6Addr,
        ttl: u32,
    }
}

impl DnsRecord {
    pub fn read(buffer: &mut BytePacketBuffer) -> Result<DnsRecord> {
        let mut domain = String::new();
        buffer.read_qname(&mut domain)?;

        let qtype_num =buffer.read_u16()?;
        let qtype = QueryType::from_num(qtype_num);
        let _ = buffer.read_u16()?;
        let ttl = buffer.read_u32()?;
        let data_len = buffer.read_u16()?;

        match qtype {
            QueryType::A => {
                let raw_addr = buffer.read_u32()?;
                let ipaddr = Ipv4Addr::new(
                    ((raw_addr >> 24) & 0xFF) as u8,
                    ((raw_addr >> 16) & 0xFF) as u8,
                    ((raw_addr >> 8) & 0xFF) as u8,
                    ((raw_addr >> 0) & 0xFF) as u8,
                );
                Ok(DnsRecord::A { domain: domain, addr: ipaddr, ttl: ttl })
            }
            
            QueryType::AAAA =>{
                let raw_addr1 = buffer.read_u32()?;
                let raw_addr2 = buffer.read_u32()?;
                let raw_addr3 = buffer.read_u32()?;
                let raw_addr4 = buffer.read_u32()?;
                let addr = Ipv6Addr::new(
                    ((raw_addr1 >> 16) & 0xFFFF) as u16,
                    ((raw_addr1 >> 0) & 0xFFFF) as u16,
                    ((raw_addr2 >> 16) & 0xFFFF) as u16,
                    ((raw_addr2 >> 0) & 0xFFFF) as u16,
                    ((raw_addr3 >> 16) & 0xFFFF) as u16,
                    ((raw_addr3 >> 0) & 0xFFFF) as u16,
                    ((raw_addr4 >> 16) & 0xFFFF) as u16,
                    ((raw_addr4 >> 0) & 0xFFFF) as u16,
                );
                Ok(DnsRecord::AAAA { domain: domain, addr: addr, ttl: ttl })
            }
            QueryType::NS => {
                let mut ns = String::new();
                buffer.read_qname(&mut ns)?;

                Ok(DnsRecord::NS {
                    domain,
                    host: ns,
                    ttl,
                })
            }
            QueryType::CNAME => {
                let mut cname = String::new();
                buffer.read_qname(&mut cname)?;

                Ok(DnsRecord::CNAME {
                    domain,
                    host: cname,
                    ttl,
                })
            }
            QueryType::MX => {
                let priority = buffer.read_u16()?;
                let mut mx = String::new();
                buffer.read_qname(&mut mx)?;

                Ok(DnsRecord::MX {
                    domain,
                    priority,
                    host: mx,
                    ttl,
                })
            }
            QueryType::UNKNOWN(_) => {
                buffer.step(data_len as usize)?;

                Ok(DnsRecord::UNKNOWN {
                    domain,
                    qtype: qtype_num,
                    data_len,
                    ttl,
                })
            }
        }
    }
    pub fn write(&self, buffer: &mut BytePacketBuffer) -> Result<usize> {
        let start_pos = buffer.pos();
        match *self {
            DnsRecord::A {
                ref domain,
                ref addr,
                ttl,
            } => {
                buffer.write_qname(domain)?;
                buffer.write_u16(QueryType::A.to_num())?;
                buffer.write_u16(1)?;
                buffer.write_u32(ttl)?;
                buffer.write_u16(4)?;


                let octets = addr.octets();
                buffer.write_u8(octets[0])?;
                buffer.write_u8(octets[1])?;
                buffer.write_u8(octets[2])?;
                buffer.write_u8(octets[3])?;
            }
            DnsRecord::NS {
                ref domain,
                ref host,
                ttl,
            } => {
                buffer.write_qname(domain)?;
                buffer.write_u16(QueryType::NS.to_num())?;
                buffer.write_u16(1)?;
                buffer.write_u32(ttl)?;
                let pos = buffer.pos();
                buffer.write_u16(0)?;
                buffer.write_qname(host)?;
                let size = buffer.pos() - (pos+2);
                buffer.set_u16(pos, size as u16)?;
            }
            DnsRecord::CNAME {
                ref domain,
                ref host,
                ttl,
             }  => {
                 buffer.write_qname(domain)?;
                 buffer.write_u16(QueryType::CNAME.to_num())?;
                    buffer.write_u16(1)?;
                    buffer.write_u32(ttl)?;
    
                    let pos = buffer.pos();
                    buffer.write_u16(0)?;
    
                    buffer.write_qname(host)?;
    
                    let size = buffer.pos() - (pos + 2);
                    buffer.set_u16(pos, size as u16)?;
             }
             DnsRecord::MX {
                ref domain,
                priority,
                ref host,
                ttl,
            } => {
                buffer.write_qname(domain)?;
                buffer.write_u16(QueryType::MX.to_num())?;
                buffer.write_u16(1)?;
                buffer.write_u32(ttl)?;

                let pos = buffer.pos();
                buffer.write_u16(0)?;

                buffer.write_u16(priority)?;
                buffer.write_qname(host)?;

                let size = buffer.pos() - (pos + 2);
                buffer.set_u16(pos, size as u16)?;
            }
            DnsRecord::AAAA {
                ref domain,
                ref addr,
                ttl,
            } => {
                buffer.write_qname(domain)?;
                buffer.write_u16(QueryType::AAAA.to_num())?;
                buffer.write_u16(1)?;
                buffer.write_u32(ttl)?;
                buffer.write_u16(16)?;

                for octet in &addr.segments() {
                    buffer.write_u16(*octet)?;
                }
            }
            DnsRecord::UNKNOWN { .. } => {
                println!("Skipping record: {:?}", self);
            }
        }
        Ok(buffer.pos() - start_pos)
    }
}
