use std::time::SystemTime;
use crate::rp::cores::rp_parser::{RPParser, RPParseRecord, RPParseError};
use crate::rp::cores::rp_patcher::{RPPatchError, RPPatchEvent, RPPatcher};
use crate::rp::ups::ups_record::{UPSRecord};
use crate::utils::byte_reader::ByteReader;
use crate::utils::read_until::read_until;

pub struct UPS;

impl UPS {
    const HEADER: [u8; 4] = *b"UPS1";
}

impl RPParser<UPSRecord> for IPS {
    fn parse_record(patch: &[u8]) -> Result<RPParseRecord<UPSRecord>, RPParseError> {
        todo!()
    }

    fn parse(patch: &[u8]) -> Result<Vec<UPSRecord>, RPParseError> {
        todo!()
    }
}

impl RPPatcher<UPSRecord> for IPS {
    fn patch_record(rom: &mut [u8], patch_record: &UPSRecord) -> Result<RPPatchEvent<UPSRecord>, RPPatchError> {
        todo!()
    }

    fn patch(rom: &[u8], patch_records: &[UPSRecord]) -> Result<Vec<RPPatchEvent<UPSRecord>>, RPPatchError> {
        todo!()
    }
}