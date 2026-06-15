pub enum IPSRecord {
    Data(u32, u16, Vec<u8>),
    RLE(u32, u16, u16, u8),
}


impl IPSRecord {
    pub fn new_with_data(offset: u32, len: u16, data: &[u8]) -> Self {
        IPSRecord::Data(offset, len, Vec::from(data))
    }

    pub fn new_with_rle(offset: u32, len: u16, repeat_len: u16, repeat_byte: u8) -> Self {
        IPSRecord::RLE(offset, len, repeat_len, repeat_byte)
    }
}