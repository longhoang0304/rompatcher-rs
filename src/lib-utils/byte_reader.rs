use std::error::Error;
use std::fmt;

use num_traits::{CheckedAdd, CheckedShl, FromPrimitive, PrimInt, Unsigned};

pub struct ByteReader<'a> {
    data: &'a [u8],
    pos: usize,
}

#[derive(Debug)]
pub enum ByteReaderError {
    /// Ended before a full field could be read.
    UnexpectedEof,
    /// Parsed value is too big for current variable.
    ValueOverflow,
}

impl fmt::Display for ByteReaderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            ByteReaderError::UnexpectedEof => "unexpected end of patch file",
            ByteReaderError::ValueOverflow => "read value overflowed, cannot be held by current variable",
        };
        f.write_str(message)
    }
}

impl Error for ByteReaderError {}

impl<'a> ByteReader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    pub fn consumed(&self) -> usize {
        self.pos
    }

    pub fn take(&mut self, n: usize) -> Result<&'a [u8], ByteReaderError> {
        let end = self.pos.checked_add(n).ok_or(ByteReaderError::UnexpectedEof)?;
        let slice = self.data.get(self.pos..end).ok_or(ByteReaderError::UnexpectedEof)?;
        self.pos = end;
        Ok(slice)
    }

    pub fn u8_take(&mut self) -> Result<u8, ByteReaderError> {
        Ok(self.take(1)?[0])
    }

    pub fn u16_be_take(&mut self) -> Result<u16, ByteReaderError> {
        let b = self.take(2)?;
        Ok(u16::from_be_bytes([b[0], b[1]]))
    }

    pub fn u24_be_take(&mut self) -> Result<u32, ByteReaderError> {
        let b = self.take(3)?;
        Ok(u32::from_be_bytes([0, b[0], b[1], b[2]]))
    }

    pub fn u32_le_take(&mut self) -> Result<u32, ByteReaderError> {
        let b = self.take(4)?;
        let bytes: [u8; 4] = b.try_into().map_err(|_| ByteReaderError::UnexpectedEof)?;
        Ok(u32::from_be_bytes(bytes))
    }

    pub fn vli_take<U>(&mut self) -> Result<U, ByteReaderError>
    where U: PrimInt + Unsigned + FromPrimitive + CheckedShl + CheckedAdd,
        U: From<u8>
    {
        let mut value: U = U::zero();
        let mut shift = 0;

        loop {
            let byte = self.u8_take()?;
            let bits: U = (byte & 0x7F).into();

            // value += (byte & 0x7f) << shift
            let chunk = bits.checked_shl(shift).ok_or(ByteReaderError::ValueOverflow)?;

            // if high bits are dropped -> overflow
            if chunk.unsigned_shr(shift) != bits {
                return Err(ByteReaderError::ValueOverflow);
            }

            // otherwise add
            value = value.checked_add(&chunk).ok_or(ByteReaderError::ValueOverflow)?;

            // high bit set => final byte (UPS convention)
            if byte & 0x80 != 0 {
                return Ok(value);
            }

            shift += 7;

            // UPS bias: value += 1 << shift
            let bias = U::one().checked_shl(shift).ok_or(ByteReaderError::ValueOverflow)?;
            value = value.checked_add(&bias).ok_or(ByteReaderError::ValueOverflow)?;
        }
    }

    // ======== NO MUTATIONS =========

    // Non-mut version of take
    pub fn get(&self, start: usize, n: usize) -> Result<&'a [u8], ByteReaderError> {
        let end = start.checked_add(n).ok_or(ByteReaderError::UnexpectedEof)?;
        let slice = self.data.get(start..end).ok_or(ByteReaderError::UnexpectedEof)?;
        Ok(slice)
    }

    pub fn u8_get(&self, start: usize) -> Result<u8, ByteReaderError> {
        Ok(self.get(start, 1)?[0])
    }

    /// Non-mut version of vli
    pub fn vli_get<U>(&self, start: usize) -> Result<(U, usize), ByteReaderError>
    where U: PrimInt + Unsigned + FromPrimitive + CheckedShl + CheckedAdd,
          U: From<u8>
    {
        let mut read = 0;
        let mut value: U = U::zero();
        let mut shift = 0;

        loop {
            let byte = self.u8_get(start + read)?;
            let bits: U = (byte & 0x7F).into();

            // value += (byte & 0x7f) << shift
            let chunk = bits.checked_shl(shift).ok_or(ByteReaderError::ValueOverflow)?;

            // if high bits are dropped -> overflow
            if chunk.unsigned_shr(shift) != bits {
                return Err(ByteReaderError::ValueOverflow);
            }

            // otherwise add
            value = value.checked_add(&chunk).ok_or(ByteReaderError::ValueOverflow)?;

            // high bit set => final byte (UPS convention)
            if byte & 0x80 != 0 {
                return Ok((value, read));
            }

            shift += 7;

            // UPS bias: value += 1 << shift
            let bias = U::one().checked_shl(shift).ok_or(ByteReaderError::ValueOverflow)?;
            value = value.checked_add(&bias).ok_or(ByteReaderError::ValueOverflow)?;

            read += 1;
        }
    }

    pub fn get_until_byte(&self, start: usize, exp_byte: u8) -> Result<Vec<u8>, ByteReaderError> {
        let mut data = Vec::new();
        let mut start = start;

        loop {
            let byte = self.u8_get(start)?;
            if byte == exp_byte {
                return Ok(data);
            }
            data.push(byte);

            start += 1;
            if start >= self.data.len() {
                return Err(ByteReaderError::UnexpectedEof);
            }
        }
    }
}
