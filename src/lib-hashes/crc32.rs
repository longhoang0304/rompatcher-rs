pub struct CRC32;

impl CRC32 {
    const POLY: u32 = 0xEDB8_8320;
    const CRC32_TABLE: [u32; 256] = {
        let mut table = [0u32; 256];
        let mut i = 0;

        while i < 256 {
            let mut crc = i as u32;
            let mut j = 0;
            while j < 8 {
                if (crc & 1) == 1 {
                    crc = (crc >> 1) ^ Self::POLY;
                } else {
                    crc >>= 1;
                }
                j += 1;
            }
            table[i] = crc;
            i += 1;
        }
        table
    };

    pub fn hash(data: &[u8]) -> u32 {
        // all 1s
        let mut crc = 0xFFFF_FFFFu32;

        // iterate left-to-right through the ROM data
        for &byte in data {
            let table_index = ((crc as u8) ^ byte) as usize;
            crc = (crc >> 8) ^ Self::CRC32_TABLE[table_index];
        }

        // invert the bits at the very end
        crc ^ 0xFFFF_FFFF
    }

    pub fn validate(data: &[u8], exp_crc32: u32) -> bool {
        let crc32 = Self::hash(data);

        exp_crc32 == crc32
    }
}
