#[derive(Clone, Debug)]
pub struct UPSRecord {
    pub offset: usize,
    pub data: Vec<u8>,
}


impl UPSRecord {
    pub fn new(offset: usize, data: Vec<u8>) -> UPSRecord {
        UPSRecord {
            offset,
            data,
        }
    }
}
