use std::fmt;

use crate::utils::byte_reader::UnexpectedEof;

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
}

impl fmt::Display for RPParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            RPParseError::UnexpectedEof => "unexpected end of patch file",
            RPParseError::InvalidHeader => "invalid patch file header",
            RPParseError::MissingFooter => "patch file footer not found",
        };
        f.write_str(message)
    }
}

impl std::error::Error for RPParseError {}

impl From<UnexpectedEof> for RPParseError {
    fn from(_: UnexpectedEof) -> Self {
        RPParseError::UnexpectedEof
    }
}

pub trait RPParser<P> {

    fn parse_record(patch: &[u8]) -> Result<RPParseRecord<P>, RPParseError>;
    fn parse(patch: &[u8]) -> Result<Vec<P>, RPParseError>;
}