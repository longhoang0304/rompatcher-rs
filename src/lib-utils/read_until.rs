#[derive(Debug)]
pub struct PatternNotFound;

pub fn read_until<'a>(data: &'a [u8], pattern: &[u8]) -> Result<&'a [u8], PatternNotFound> {
    data.windows(pattern.len())
        .position(|w| w == pattern)
        .map(|i| &data[..i]) // everything before the pattern
        .ok_or(PatternNotFound)
}