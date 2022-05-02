use std::hash::BuildHasherDefault;

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


