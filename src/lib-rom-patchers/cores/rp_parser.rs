use std::fmt;
use std::error::Error;

use crate::utils::byte_reader::{ByteReaderError};

pub struct RPParseRecord<P> {
    pub(crate) record: P,
    pub(crate) len: usize,
}

impl<P> RPParseRecord<P> {
    pub fn new(record: P, len: usize) -> Self {
        RPParseRecord {
            record,
            len
        }
    }
}

#[derive(Debug)]
pub enum RPParseError {
    /// Ended before a full field could be read.
    UnexpectedEof,
    /// Magic header was missing or did not match the expected value.
    InvalidHeader,
    /// Required footer marker was not found.
    MissingFooter,
    /// Parsed value is too big for current variable.
    ValueOverflow,
}

impl fmt::Display for RPParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            RPParseError::UnexpectedEof => "unexpected end of patch file",
            RPParseError::InvalidHeader => "invalid patch file header",
            RPParseError::MissingFooter => "patch file footer not found",
            RPParseError::ValueOverflow => "read value overflowed, cannot be held by current variable",
        };
        f.write_str(message)
    }
}

impl Error for RPParseError {}

impl From<ByteReaderError> for RPParseError {
    fn from(br_err: ByteReaderError) -> Self {
        match br_err {
            ByteReaderError::UnexpectedEof => RPParseError::UnexpectedEof,
            ByteReaderError::ValueOverflow => RPParseError::ValueOverflow,
        }
    }
}

pub trait RPParser<B, P> {
    const HEADER: &'static [u8] = b"";

    fn verify_header(patch: &[u8]) -> Result<(), RPParseError> {
        if Self::HEADER.is_empty() {
            return Ok(())
        }

        let header = patch
            .get(..Self::HEADER.len())
            .ok_or(RPParseError::UnexpectedEof)?;

        if header != Self::HEADER {
            return Err(RPParseError::InvalidHeader);
        }

        Ok(())
    }
    fn parse_record(patch: &[u8]) -> Result<RPParseRecord<P>, RPParseError>;
    fn parse(patch: &[u8]) -> Result<B, RPParseError>;
}