use std::fmt;
use std::error::Error;
use std::time::SystemTime;

pub struct RPPatchEvent<P> {
    pub timestamp: SystemTime,
    pub patch_record: Box<P>
}

#[derive(Debug)]
pub enum RPPatchError {
    UnexpectedEof,
    OverflowPatchRecordEof(u32, u16, u32),
}

impl fmt::Display for RPPatchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RPPatchError::UnexpectedEof => f.write_str("unexpected end of patch file"),
            RPPatchError::OverflowPatchRecordEof(offset, size, rom_size) => write!(f, "Patch record with offset ({offset}) and size ({size}) overflowed rom size ({rom_size})."),
        }
    }
}

impl Error for RPPatchError {}

pub trait RPPatcher<P> {
    fn patch_record(rom: &mut [u8], patch_record: &P) -> Result<RPPatchEvent<P>, RPPatchError>;
    fn patch(rom: &[u8], patch_records: &[P]) -> Result<Vec<RPPatchEvent<P>>, RPPatchError>;
}