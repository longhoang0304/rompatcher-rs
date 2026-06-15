use crate::rp_cores::cores::rp_parser::{RPParser, RPParseRecord, RPParseError};
use crate::rp_cores::ips::ips_record::IPSRecord;
use crate::utils::byte_reader::ByteReader;
use crate::utils::read_until::read_until;

pub struct IPS;

impl IPS {
    const HEADER: [u8; 5] = *b"PATCH";
    const FOOTER: [u8; 3] = *b"EOF";
}

impl RPParser<IPSRecord> for IPS {
    fn parse_record(patch: &[u8]) -> Result<RPParseRecord<IPSRecord>, RPParseError> {
        let mut reader = ByteReader::new(patch);

        let offset = reader.u24_be()?;
        let len = reader.u16_be()?;

        let record = if len != 0 {
            let data = reader.take(len as usize)?;
            IPSRecord::new_with_data(offset, len, data)
        } else {
            let repeat_len = reader.u16_be()?;
            let repeat_byte = reader.u8()?;
            IPSRecord::new_with_rle(offset, len, repeat_len, repeat_byte)
        };

        Ok(RPParseRecord::new(record, reader.consumed()))
    }

    fn parse(patch: &[u8]) -> Result<Vec<IPSRecord>, RPParseError> {
        let header = patch
            .get(..Self::HEADER.len())
            .ok_or(RPParseError::InvalidHeader)?;
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