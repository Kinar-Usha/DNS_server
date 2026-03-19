//! This module contains methods which help in manipulating the udp packet

type Error = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Error>;

/// UDP packet structure.
pub struct BytePacketBuffer {
    pub buf: [u8; 512],
    pub pos: usize,
}

impl BytePacketBuffer {
    pub fn new() -> BytePacketBuffer {
        BytePacketBuffer {
            buf: [0; 512],
            pos: 0,
        }
    }

    pub fn pos(&self) -> usize {
        self.pos
    }

    pub fn step(&mut self, steps: usize) -> Result<()> {
        self.pos += steps;
        Ok(())
    }

    pub fn seek(&mut self, pos: usize) -> Result<()> {
        self.pos = pos;

        Ok(())
    }

    pub fn read(&mut self) -> Result<u8> {
        if self.pos >= 512 {
            return Err("End of buffer".into());
        }
        let res = self.buf[self.pos];
        self.pos += 1;

        Ok(res)
    }

    pub fn get(&mut self, pos: usize) -> Result<u8> {
        if pos >= 512 {
            return Err("End of buffer".into());
        }
        Ok(self.buf[pos])
    }

    pub fn get_range(&mut self, start: usize, len: usize) -> Result<&[u8]> {
        if start + len >= 512 {
            return Err("End of buffer".into());
        }
        Ok(&self.buf[start..start + len as usize])
    }

    pub fn read_u16(&mut self) -> Result<u16> {
        let res = ((self.read()? as u16) << 8) | (self.read()? as u16);

        Ok(res)
    }

    pub fn read_u32(&mut self) -> Result<u32> {
        let res = ((self.read()? as u32) << 24)
            | ((self.read()? as u32) << 16)
            | ((self.read()? as u32) << 8)
            | ((self.read()? as u32) << 0);

        Ok(res)
    }

    pub fn read_bytes(&mut self, len: usize) -> Result<Vec<u8>> {
        let mut res = Vec::with_capacity(len);
        for _ in 0..len {
            res.push(self.read()?);
        }
        Ok(res)
    }
    /// reading domain name.
    /// Will take something like \[3\]www\[6\]google\[3\]com\[0\] and append www.google.com to outstr.
    pub fn read_qname(&mut self, outstr: &mut String) -> Result<()> {
        //keeping a track of the position locally this allows us to move past the qname while keeping track of the current pos in qname
        let mut pos = self.pos();

        // tracking jumps
        let mut jumped = false;

        // to track [dot] but it is initailly kept empty because we dont want a [dot] at the beginning.
        let mut delim = "";
        let max_jumps = 5;
        let mut jumps_performed = 0;
        loop {
            if jumps_performed > max_jumps {
                return Err(format!("Limit of {} jumps exceeded", max_jumps).into());
            }

            let len = self.get(pos)?;

            if (len & 0xC0) == 0xC0 {
                if !jumped {
                    self.seek(pos + 2)?;
                }
                // Read another byte, calculate offset and perform the jump by
                // updating our local position variable

                let b2 = self.get(pos + 1)? as u16;
                let offset = (((len as u16) ^ 0xC0) << 8) | b2;
                pos = offset as usize;
                jumped = true;
                jumps_performed += 1;
                continue;
            }
            // if just google.com, just read the next byte.
            pos += 1;

            if len == 0 {
                break;
            }

            outstr.push_str(delim);

            let str_buffer = self.get_range(pos, len as usize)?;
            outstr.push_str(&String::from_utf8_lossy(str_buffer).to_lowercase());

            delim = ".";

            pos += len as usize;
        }

        if !jumped {
            self.seek(pos)?;
        }

        Ok(())
    }

    pub fn write(&mut self, val: u8) -> Result<()> {
        if self.pos >= 512 {
            return Err("End of buffer".into());
        }
        self.buf[self.pos] = val;
        self.pos += 1;
        Ok(())
    }

    pub fn write_u8(&mut self, val: u8) -> Result<()> {
        self.write(val)?;

        Ok(())
    }

    pub fn write_u16(&mut self, val: u16) -> Result<()> {
        self.write((val >> 8) as u8)?;
        self.write((val & 0xFF) as u8)?;

        Ok(())
    }

    pub fn write_u32(&mut self, val: u32) -> Result<()> {
        self.write(((val >> 24) & 0xFF) as u8)?;
        self.write(((val >> 16) & 0xFF) as u8)?;
        self.write(((val >> 8) & 0xFF) as u8)?;
        self.write(((val >> 0) & 0xFF) as u8)?;

        Ok(())
    }

    pub fn write_qname(&mut self, qname: &str) -> Result<()> {
        for label in qname.split('.') {
            let len = label.len();
            if len > 0x34 {
                return Err("Single label exceeds 63 characters of length".into());
            }

            self.write_u8(len as u8)?;
            for b in label.as_bytes() {
                self.write_u8(*b)?;
            }
        }

        self.write_u8(0)?;

        Ok(())
    }

    pub fn set(&mut self, pos: usize, val: u8) -> Result<()> {
        self.buf[pos] = val;

        Ok(())
    }

    pub fn set_u16(&mut self, pos: usize, val: u16) -> Result<()> {
        self.set(pos, (val >> 8) as u8)?;
        self.set(pos + 1, (val & 0xFF) as u8)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== Positive Tests =====

    #[test]
    fn test_buffer_creation() {
        let buf = BytePacketBuffer::new();
        assert_eq!(buf.pos(), 0);
        assert_eq!(buf.buf.len(), 512);
    }

    #[test]
    fn test_write_and_read_single_byte() {
        let mut buf = BytePacketBuffer::new();
        assert!(buf.write(42).is_ok());
        
        let mut buf2 = BytePacketBuffer::new();
        buf2.buf = buf.buf;
        let val = buf2.read().unwrap();
        assert_eq!(val, 42);
    }

    #[test]
    fn test_write_and_read_u16() {
        let mut buf = BytePacketBuffer::new();
        assert!(buf.write_u16(0x1234).is_ok());
        
        let mut buf2 = BytePacketBuffer::new();
        buf2.buf = buf.buf;
        let val = buf2.read_u16().unwrap();
        assert_eq!(val, 0x1234);
    }

    #[test]
    fn test_write_and_read_u32() {
        let mut buf = BytePacketBuffer::new();
        assert!(buf.write_u32(0x12345678).is_ok());
        
        let mut buf2 = BytePacketBuffer::new();
        buf2.buf = buf.buf;
        let val = buf2.read_u32().unwrap();
        assert_eq!(val, 0x12345678);
    }

    #[test]
    fn test_write_qname() {
        let mut buf = BytePacketBuffer::new();
        assert!(buf.write_qname("example.com").is_ok());
        
        let mut buf2 = BytePacketBuffer::new();
        buf2.buf = buf.buf;
        let mut qname = String::new();
        assert!(buf2.read_qname(&mut qname).is_ok());
        assert_eq!(qname, "example.com");
    }

    #[test]
    fn test_write_qname_single_label() {
        let mut buf = BytePacketBuffer::new();
        assert!(buf.write_qname("localhost").is_ok());
        
        let mut buf2 = BytePacketBuffer::new();
        buf2.buf = buf.buf;
        let mut qname = String::new();
        assert!(buf2.read_qname(&mut qname).is_ok());
        assert_eq!(qname, "localhost");
    }

    #[test]
    fn test_seek() {
        let mut buf = BytePacketBuffer::new();
        assert!(buf.seek(100).is_ok());
        assert_eq!(buf.pos(), 100);
    }

    #[test]
    fn test_step() {
        let mut buf = BytePacketBuffer::new();
        assert!(buf.step(50).is_ok());
        assert_eq!(buf.pos(), 50);
        assert!(buf.step(30).is_ok());
        assert_eq!(buf.pos(), 80);
    }

    #[test]
    fn test_get() {
        let mut buf = BytePacketBuffer::new();
        buf.buf[10] = 0xFF;
        let val = buf.get(10).unwrap();
        assert_eq!(val, 0xFF);
    }

    #[test]
    fn test_get_range() {
        let mut buf = BytePacketBuffer::new();
        buf.buf[0] = 1;
        buf.buf[1] = 2;
        buf.buf[2] = 3;
        let range = buf.get_range(0, 3).unwrap();
        assert_eq!(range, &[1, 2, 3]);
    }

    #[test]
    fn test_set() {
        let mut buf = BytePacketBuffer::new();
        assert!(buf.set(50, 0xAB).is_ok());
        assert_eq!(buf.buf[50], 0xAB);
    }

    #[test]
    fn test_set_u16() {
        let mut buf = BytePacketBuffer::new();
        assert!(buf.set_u16(0, 0x5678).is_ok());
        assert_eq!(buf.buf[0], 0x56);
        assert_eq!(buf.buf[1], 0x78);
    }

    // ===== Negative Tests =====

    #[test]
    fn test_read_beyond_buffer() {
        let mut buf = BytePacketBuffer::new();
        buf.pos = 512;
        assert!(buf.read().is_err());
    }

    #[test]
    fn test_write_beyond_buffer() {
        let mut buf = BytePacketBuffer::new();
        buf.pos = 512;
        assert!(buf.write(42).is_err());
    }

    #[test]
    fn test_get_beyond_buffer() {
        let mut buf = BytePacketBuffer::new();
        assert!(buf.get(512).is_err());
    }

    #[test]
    fn test_get_range_beyond_buffer() {
        let mut buf = BytePacketBuffer::new();
        assert!(buf.get_range(500, 20).is_err());
    }

    #[test]
    fn test_read_u16_beyond_buffer() {
        let mut buf = BytePacketBuffer::new();
        buf.pos = 511; // Only 1 byte left
        assert!(buf.read_u16().is_err());
    }

    #[test]
    fn test_read_u32_beyond_buffer() {
        let mut buf = BytePacketBuffer::new();
        buf.pos = 510; // Only 2 bytes left
        assert!(buf.read_u32().is_err());
    }

    #[test]
    fn test_write_qname_label_too_long() {
        let mut buf = BytePacketBuffer::new();
        // Create a label longer than 63 characters (0x3F)
        let long_label = "a".repeat(100);
        assert!(buf.write_qname(&long_label).is_err());
    }

    #[test]
    fn test_write_qname_buffer_overflow() {
        let mut buf = BytePacketBuffer::new();
        buf.pos = 510; // Very close to end
        assert!(buf.write_qname("example.com").is_err());
    }
}

