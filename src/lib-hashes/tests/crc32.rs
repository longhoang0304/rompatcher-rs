use crate::hashes::crc32::CRC32;

// All test vectors below are CRC-32/ISO-HDLC (the zlib/PNG variant): reflected
// input and output, polynomial 0xEDB88320, init 0xFFFFFFFF, final XOR
// 0xFFFFFFFF. They are widely published and let us pin the implementation to
// the canonical algorithm.

#[test]
fn empty_input_hashes_to_zero() {
    assert_eq!(CRC32::hash(&[]), 0x0000_0000);
}

#[test]
fn check_value_of_the_standard_test_string() {
    // "123456789" is the conventional CRC check string; the ISO-HDLC variant
    // is defined to produce 0xCBF43926.
    assert_eq!(CRC32::hash(b"123456789"), 0xCBF4_3926);
}

#[test]
fn single_byte() {
    assert_eq!(CRC32::hash(b"a"), 0xE8B7_BE43);
}

#[test]
fn short_ascii_string() {
    assert_eq!(CRC32::hash(b"abc"), 0x3524_41C2);
}

#[test]
fn longer_ascii_string() {
    assert_eq!(
        CRC32::hash(b"The quick brown fox jumps over the lazy dog"),
        0x414F_A339
    );
}

#[test]
fn handles_full_byte_range() {
    // A buffer covering every possible byte value exercises every table entry.
    let data: Vec<u8> = (0u16..=255).map(|b| b as u8).collect();
    assert_eq!(CRC32::hash(&data), 0x29058C73);
}

#[test]
fn distinct_inputs_produce_distinct_hashes() {
    assert_ne!(CRC32::hash(b"hello"), CRC32::hash(b"world"));
}

#[test]
fn order_sensitivity() {
    // CRC is position dependent, so swapping bytes must change the result.
    assert_ne!(CRC32::hash(&[0x01, 0x02]), CRC32::hash(&[0x02, 0x01]));
}

#[test]
fn validate_accepts_the_matching_crc() {
    let data = b"123456789";
    assert!(CRC32::validate(data, 0xCBF4_3926));
}

#[test]
fn validate_rejects_a_mismatched_crc() {
    let data = b"123456789";
    assert!(!CRC32::validate(data, 0xDEAD_BEEF));
}

#[test]
fn validate_matches_hash() {
    let data = b"The quick brown fox jumps over the lazy dog";
    assert!(CRC32::validate(data, CRC32::hash(data)));
}
