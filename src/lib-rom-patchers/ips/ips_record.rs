#[derive(Clone, Debug)]
pub struct IPSDataRecord {
    pub(crate) offset: u32,
    pub(crate) size: u16,
    pub(crate) payload: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct IPSRLERecord {
    pub(crate) offset: u32,
    pub(crate) rle_size: u16,
    pub(crate) value: u8,
}

#[derive(Clone, Debug)]
pub enum IPSRecord {
    Data(IPSDataRecord),
    RLE(IPSRLERecord),
}


impl IPSRecord {
    pub fn new_with_data(offset: u32, size: u16, data: &[u8]) -> Self {
        IPSRecord::Data(IPSDataRecord { offset, size, payload: Vec::from(data) } )
    }

    pub fn new_with_rle(offset: u32, rle_size: u16, value: u8) -> Self {
        IPSRecord::RLE(IPSRLERecord { offset, rle_size, value })
    }
}
