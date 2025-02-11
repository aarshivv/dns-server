pub struct BytePacketBuffer {
    pub buf: [u8; 512],
    pub pos: usize,
}

impl BytePacketBuffer {
    pub fn new() -> BytePacketBuffer {
        Self { buf: [0; 512], pos: 0 }
    }

    fn pos(&self) -> usize {
        self.pos
    }

    fn step(&mut self, steps: usize) -> Result<(), String> {
        self.pos += steps;

        Ok(())
    }

    fn seek(&mut self, pos: usize) -> Result<(), String> {
        self.pos = pos;

        Ok(())
    }

    fn read(&mut self) -> Result<u8, String> {
        if self.pos >= 512 {
            return Err("End of buffer".into())
        }

        let res = self.buf[self.pos];
        self.pos += 1;

        Ok(res)
    }

    fn get(&mut self, pos: usize) -> Result<u8, String> {
        if pos >= 512 {
            return Err("End of buffer".into())
        }

        Ok(self.buf[pos])
    }

    fn get_range(&mut self, start: usize, len: usize) -> Result<&[u8], String> {
        if start + len >= 512 {
            return Err("End of buffer".into())
        }

        Ok(&self.buf[start..start + len as usize])
    }

    fn read_u16(&mut self) -> Result<u16, String> {
        let res = ((self.read()? as u16) << 8) | (self.read()? as u16);

        Ok(res)
    }

    fn read_u32(&mut self) -> Result<u32, String> {
        let res = ((self.read()? as u32) << 24) | ((self.read()? as u32) << 16) | ((self.read()? as u32) << 8) | ((self.read()? as u32) << 0);

        Ok(res)
    }

    fn read_qname(&mut self, outstr: &mut String) -> Result<(), String> {
        let mut pos = self.pos;

        let mut jumped = false;
        let max_jumps = 5;
        let mut jumps_performed = 0;

        let mut delim = "";

        loop {
            if jumps_performed > max_jumps {
                return Err(format!("Limit of {max_jumps} jumps exceeded"));
            }

            let len = self.get(pos)?;

            if (len & 0xC0) == 0xC0 {}
        }
    }
}