#[derive(Clone, Debug)]
pub struct UPSRecord {
    pub offset: u64,
    pub size: u64,
    pub data: Vec<u8>,
}


impl UPSRecord {
    pub fn new(offset: u64, size: u64, data: Vec<u8>) -> UPSRecord {
        UPSRecord {
            offset,
            size,
            data,
        }
    }
}
