pub struct ByteReader<'a> {
    data: &'a [u8],
    pos: usize,
}

#[derive(Debug)]
pub struct UnexpectedEof;

impl<'a> ByteReader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    pub fn consumed(&self) -> usize {
        self.pos
    }

    pub fn take(&mut self, n: usize) -> Result<&'a [u8], UnexpectedEof> {
        let end = self.pos.checked_add(n).ok_or(UnexpectedEof)?;
        let slice = self.data.get(self.pos..end).ok_or(UnexpectedEof)?;
        self.pos = end;
        Ok(slice)
    }

    pub fn u8(&mut self) -> Result<u8, UnexpectedEof> {
        Ok(self.take(1)?[0])
    }

    pub fn u16_be(&mut self) -> Result<u16, UnexpectedEof> {
        let b = self.take(2)?;
        Ok(u16::from_be_bytes([b[0], b[1]]))
    }

    pub fn u24_be(&mut self) -> Result<u32, UnexpectedEof> {
        let b = self.take(3)?;
        Ok(u32::from_be_bytes([0, b[0], b[1], b[2]]))
    }
}
