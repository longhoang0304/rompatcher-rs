#[derive(Clone, Debug)]
pub struct IPSDataRecord {
    pub offset: u32,
    pub size: u16,
    pub payload: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct IPSRLERecord {
    pub offset: u32,
    pub size: u16,
    pub rle_size: u16,
    pub value: u8,
}

impl From<&IPSRLERecord> for IPSDataRecord {
    fn from(rle: &IPSRLERecord) -> IPSDataRecord {
        IPSDataRecord {
            offset: rle.offset,
            size: rle.size,
            payload: vec![rle.value; rle.rle_size as usize],
        }
    }
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

    pub fn new_with_rle(offset: u32, size: u16, rle_size: u16, value: u8) -> Self {
        IPSRecord::RLE(IPSRLERecord { offset, size, rle_size, value })
    }
}
