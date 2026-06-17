use std::time::SystemTime;
use crate::rp::cores::rp_parser::{RPParser, RPParseRecord, RPParseError};
use crate::rp::cores::rp_patcher::{RPPatchError, RPPatchEvent, RPPatchResult, RPPatcher};
use crate::rp::ips::ips_record::{IPSDataRecord, IPSRLERecord, IPSRecord};
use crate::utils::byte_reader::ByteReader;

pub struct IPS {
    pub records: Vec<IPSRecord>,
}

impl IPS {
    const FOOTER: [u8; 3] = *b"EOF";

    // The byte just past the end of the region a record writes to. IPS offsets
    // are 24-bit, so a record may legitimately reach beyond the input ROM.
    fn record_end(record: &IPSRecord) -> usize {
        match record {
            IPSRecord::Data(d) => d.offset as usize + d.payload.len(),
            IPSRecord::RLE(r) => r.offset as usize + r.rle_size as usize,
        }
    }

    // prevent overflow
    fn resolve_range(
        rom_len: usize,
        offset: u32,
        len: usize,
        size: u16,
    ) -> Result<(usize, usize), RPPatchError> {
        let start = offset as usize;
        let end = start
            .checked_add(len)
            .filter(|&end| end <= rom_len)
            .ok_or(RPPatchError::OverflowPatchRecordEof(offset, size, rom_len as u32))?;

        Ok((start, end))
    }

    fn patch_data_record(rom: &mut [u8], patch_record: &IPSDataRecord) -> Result<(), RPPatchError> {
        let (start, end) = Self::resolve_range(
            rom.len(),
            patch_record.offset,
            patch_record.payload.len(),
            patch_record.size,
        )?;

        rom[start..end].copy_from_slice(&patch_record.payload);

        Ok(())
    }

    fn patch_rle_record(rom: &mut [u8], patch_record: &IPSRLERecord) -> Result<(), RPPatchError> {
        let (start, end) = Self::resolve_range(
            rom.len(),
            patch_record.offset,
            patch_record.rle_size as usize,
            patch_record.rle_size,
        )?;

        rom[start..end].fill(patch_record.value);

        Ok(())
    }
}

impl RPParser<IPS, IPSRecord> for IPS {
    const HEADER: &'static [u8] = b"PATCH";
    
    fn parse_record(patch: &[u8]) -> Result<RPParseRecord<IPSRecord>, RPParseError> {
        let mut reader = ByteReader::new(patch);

        let offset = reader.u24_be_take()?;
        let size = reader.u16_be_take()?;

        let record = if size != 0 {
            let payload = reader.take(size as usize)?;
            IPSRecord::new_with_data(offset, size, payload)
        } else {
            let rle_size = reader.u16_be_take()?;
            let value = reader.u8_take()?;
            IPSRecord::new_with_rle(offset, rle_size, value)
        };

        Ok(RPParseRecord::new(record, reader.consumed()))
    }

    fn parse(patch: &[u8]) -> Result<IPS, RPParseError> {
        Self::verify_header(patch)?;

        let mut body = &patch[Self::HEADER.len()..];
        let mut records = Vec::new();

        loop {
            // read until EOF is met
            // if an offset is equal to EOF then welp
            // we are in a bad luck situation
            let head = body
                .get(..Self::FOOTER.len())
                .ok_or(RPParseError::MissingFooter)?;

            if head == Self::FOOTER {
                break;
            }

            let parsed = Self::parse_record(body)?;
            records.push(parsed.record);
            body = &body[parsed.len..];
        }

        Ok(IPS { records })
    }
}

impl RPPatcher<IPS, IPSRecord> for IPS {
    fn patch_record(rom: &mut [u8], patch_record: &IPSRecord) -> Result<RPPatchEvent<IPSRecord>, RPPatchError> {
        match patch_record {
            IPSRecord::Data(data_record) => Self::patch_data_record(rom, data_record)?,
            IPSRecord::RLE(rle_record) => Self::patch_rle_record(rom, rle_record)?,
        }

        Ok(RPPatchEvent {
            timestamp: SystemTime::now(),
            patch_record: Box::new(patch_record.clone()),
        })
    }

    fn patch(rom: &[u8], patch: &IPS) -> Result<RPPatchResult<IPSRecord>, RPPatchError> {
        // IPS has no output-size field: the ROM grows to the furthest record,
        // with any gap zero-filled.
        let output_len = patch
            .records
            .iter()
            .map(Self::record_end)
            .max()
            .unwrap_or(0)
            .max(rom.len());

        let mut patched_rom = rom.to_vec();
        patched_rom.resize(output_len, 0);
        let mut events = Vec::new();

        for record in &patch.records {
            let event = Self::patch_record(&mut patched_rom, record)?;
            events.push(event);
        }

        Ok(RPPatchResult {
            events,
            patched_rom,
        })
    }
}
