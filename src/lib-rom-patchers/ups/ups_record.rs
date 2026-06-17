#[derive(Clone, Debug)]
pub struct UPSRecord {
    pub offset: usize,
    pub payload: Vec<u8>,
}


impl UPSRecord {
    pub fn new(offset: usize, payload: Vec<u8>) -> UPSRecord {
        UPSRecord {
            offset,
            payload,
        }
    }
}
