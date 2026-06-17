use std::time::SystemTime;
use crate::rp::cores::rp_parser::{RPParser, RPParseRecord, RPParseError};
use crate::rp::cores::rp_patcher::{RPPatchError, RPPatchEvent, RPPatchResult, RPPatcher};
use crate::rp::ups::ups_record::{UPSRecord};
use crate::utils::byte_reader::ByteReader;

pub struct UPS {
    pub records: Vec<UPSRecord>,
    pub inp_rom_size: usize,
    pub out_rom_size: usize,
    pub inp_rom_crc32: u32,
    pub out_rom_crc32: u32,
    pub patch_crc32: u32,
}

impl UPS {
}

impl RPParser<UPS, UPSRecord> for UPS {
    const HEADER: &'static [u8] = b"UPS1";

    fn parse_record(patch: &[u8]) -> Result<RPParseRecord<UPSRecord>, RPParseError> {
        let reader = ByteReader::new(patch);
        let (offset, vli_read_bytes) = reader.vli_get::<usize>(0)?;
        let payload_offset = vli_read_bytes + 1;
        let payload = reader.get_until_byte(payload_offset, 0x00)?;
        let size = vli_read_bytes + payload.len();

        return Ok(
            RPParseRecord::new(
                UPSRecord::new(offset, payload),
                size,
            )
        )
    }

    fn parse(patch: &[u8]) -> Result<UPS, RPParseError> {
        Self::verify_header(patch)?;

        let size_offset = patch.len() - Self::HEADER.len();
        let sizes = &patch[size_offset..];
        let mut size_reader = ByteReader::new(sizes);
        let inp_rom_size = size_reader.vli_take::<usize>()?;
        let out_rom_size = size_reader.vli_take::<usize>()?;

        let check_sums_offset = patch.len() - 12;
        let check_sums = &patch[check_sums_offset..];

        let mut checksum_reader = ByteReader::new(check_sums);
        let inp_rom_crc32 = checksum_reader.u32_le_take()?;
        let out_rom_crc32 = checksum_reader.u32_le_take()?;
        let patch_crc32 = checksum_reader.u32_le_take()?;

        let records_offset = size_reader.consumed();
        let mut records_segment = &patch[records_offset..];
        let mut records = Vec::new();
        loop {
            let rp_record = Self::parse_record(&records_segment)?;

            records.push(rp_record.record);

            records_segment = &records_segment[rp_record.len + 1..];
            // reached the end
            if records_segment.len() == 12 {
                break;
            }
        }

        Ok(
            UPS {
                records,
                inp_rom_size,
                out_rom_size,
                inp_rom_crc32,
                out_rom_crc32,
                patch_crc32,
            }
        )
    }
}

impl RPPatcher<UPS, UPSRecord> for UPS {
    fn patch_record(
        rom: &mut [u8],
        patch_record: &UPSRecord
    ) -> Result<RPPatchEvent<UPSRecord>, RPPatchError> {
        let patch_src = &patch_record.payload;
        for (rom_byte, patch_byte) in rom.iter_mut().zip(patch_src.iter()) {
            *rom_byte ^= *patch_byte;
        }

        Ok(
            RPPatchEvent {
                timestamp: SystemTime::now(),
                patch_record: Box::new(patch_record.clone())
            }
        )
    }

    fn patch(rom: &[u8], patch: &UPS) -> Result<RPPatchResult<UPSRecord>, RPPatchError> {
        let mut patched_rom = Vec::from(rom);
        let mut events: Vec<RPPatchEvent<UPSRecord>> = Vec::new();
        let mut cursor = 0;

        let patch_records = &patch.records;

        for record in patch_records {
            let patched_rom = patched_rom
                .get_mut(cursor + record.offset..)
                .ok_or(RPPatchError::UnexpectedEof)?;
            let event = Self::patch_record(patched_rom, record)?;

            events.push(event);

            cursor += record.offset + record.payload.len() + 1; // +1 for 0x00 byte
        }

        Ok(
            RPPatchResult {
                events,
                patched_rom,
            }
        )
    }
}