use std::time::SystemTime;
use crate::rp_cores::cores::rp_parser::{RPParser, RPParseRecord, RPParseError};
use crate::rp_cores::cores::rp_patcher::{RPPatchError, RPPatchEvent, RPPatcher};
use crate::rp_cores::ips::ips_record::{IPSDataRecord, IPSRLERecord, IPSRecord};
use crate::utils::byte_reader::ByteReader;
use crate::utils::read_until::read_until;

pub struct IPS;

impl IPS {
    const HEADER: [u8; 5] = *b"PATCH";
    const FOOTER: [u8; 3] = *b"EOF";

    fn patch_data_record(rom: &mut [u8], patch_record: &IPSDataRecord) -> Result<(), RPPatchError> {
        let start = patch_record.offset as usize;
        let size = patch_record.size as usize;
        let end = start + size;

        // Check for overflow using the calculated indices
        if end > rom.len() {
            return Err(RPPatchError::OverflowPatchRecordEof(
                patch_record.offset,
                patch_record.size,
                rom.len() as u32
            ));
        }

        rom[start..end].copy_from_slice(&patch_record.payload);

        Ok(())
    }

    fn patch_rle_record(rom: &mut [u8], patch_record: &IPSRLERecord) -> Result<(), RPPatchError> {
        Self::patch_data_record(rom, &patch_record.into())
    }
}

impl RPParser<IPSRecord> for IPS {
    fn parse_record(patch: &[u8]) -> Result<RPParseRecord<IPSRecord>, RPParseError> {
        let mut reader = ByteReader::new(patch);

        let offset = reader.u24_be()?;
        let size = reader.u16_be()?;

        let record = if size != 0 {
            let payload = reader.take(size as usize)?;
            IPSRecord::new_with_data(offset, size, payload)
        } else {
            let rle_size = reader.u16_be()?;
            let value = reader.u8()?;
            IPSRecord::new_with_rle(offset, size, rle_size, value)
        };

        Ok(RPParseRecord::new(record, reader.consumed()))
    }

    fn parse(patch: &[u8]) -> Result<Vec<IPSRecord>, RPParseError> {
        let header = patch
            .get(..Self::HEADER.len())
            .ok_or(RPParseError::UnexpectedEof)?;
        if header != Self::HEADER {
            return Err(RPParseError::InvalidHeader);
        }

        let body = &patch[Self::HEADER.len()..];
        let mut body = read_until(body, &Self::FOOTER)
            .map_err(|_| RPParseError::MissingFooter)?;

        let mut records: Vec<IPSRecord> = Vec::new();
        while !body.is_empty() {
            let parsed = Self::parse_record(body)?;
            records.push(parsed.record);
            body = &body[parsed.len..];
        }

        Ok(records)
    }
}

impl RPPatcher<IPSRecord> for IPS {
    fn patch_record(rom: &mut [u8], patch_record: &IPSRecord) -> Result<RPPatchEvent<IPSRecord>, RPPatchError> {
        match patch_record {
            IPSRecord::Data(data_record) => Self::patch_data_record(rom, data_record)?,
            IPSRecord::RLE(rle_record) => Self::patch_rle_record(rom, rle_record)?,
        }

        Ok(
            RPPatchEvent {
                timestamp: SystemTime::now(),
                patch_record: Box::new(patch_record.clone())
            }
        )
    }

    fn patch(rom: &[u8], patch_records: &[IPSRecord]) -> Result<Vec<RPPatchEvent<IPSRecord>>, RPPatchError> {
        let mut patched_rom = Vec::from(rom);
        let mut events: Vec<RPPatchEvent<IPSRecord>> = Vec::new();

        for record in patch_records {
            let event = Self::patch_record(&mut patched_rom, record)?;
            events.push(event)
        }

        Ok(events)
    }
}