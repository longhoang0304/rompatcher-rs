use crate::rp::cores::rp_parser::{RPParseError, RPParser};
use crate::rp::ups::ups::UPS;

// UPS VLI quick reference (high bit = final byte, with the +1 bias on each
// continuation):
//   0 -> 0x80   1 -> 0x81   2 -> 0x82   4 -> 0x84
//   128 -> [0x00, 0x80]
//
// A 12-byte footer follows every patch: input CRC32, output CRC32, patch
// CRC32, each little-endian.

#[test]
fn parses_a_single_record() {
    let patch = [
        b'U', b'P', b'S', b'1', // header
        0x84, // input size = 4
        0x84, // output size = 4
        0x81, 0xAA, 0xBB, 0x00, // record: offset +1, xor [AA, BB], terminator
        0x44, 0x33, 0x22, 0x11, // input crc32  = 0x11223344 (LE)
        0x88, 0x77, 0x66, 0x55, // output crc32 = 0x55667788 (LE)
        0xCC, 0xBB, 0xAA, 0x99, // patch crc32  = 0x99AABBCC (LE)
    ];

    let ups = UPS::parse(&patch).expect("patch should parse");

    assert_eq!(ups.inp_rom_size, 4);
    assert_eq!(ups.out_rom_size, 4);
    assert_eq!(ups.inp_rom_crc32, 0x1122_3344);
    assert_eq!(ups.out_rom_crc32, 0x5566_7788);
    assert_eq!(ups.patch_crc32, 0x99AA_BBCC);

    assert_eq!(ups.records.len(), 1);
    assert_eq!(ups.records[0].offset, 1);
    assert_eq!(ups.records[0].payload, vec![0xAA, 0xBB]);
}

#[test]
fn parses_multiple_records_in_order() {
    let patch = [
        b'U', b'P', b'S', b'1', // header
        0x84, // input size = 4
        0x84, // output size = 4
        0x81, 0xAA, 0x00, // record 1: offset +1, xor [AA]
        0x80, 0xBB, 0xCC, 0x00, // record 2: offset +0, xor [BB, CC]
        0x00, 0x00, 0x00, 0x00, // input crc32
        0x00, 0x00, 0x00, 0x00, // output crc32
        0x00, 0x00, 0x00, 0x00, // patch crc32
    ];

    let ups = UPS::parse(&patch).expect("patch should parse");

    assert_eq!(ups.records.len(), 2);
    assert_eq!(ups.records[0].offset, 1);
    assert_eq!(ups.records[0].payload, vec![0xAA]);
    assert_eq!(ups.records[1].offset, 0);
    assert_eq!(ups.records[1].payload, vec![0xBB, 0xCC]);
}

#[test]
fn parses_an_empty_payload_record() {
    // A bare offset followed immediately by the terminator: no xor bytes.
    let patch = [
        b'U', b'P', b'S', b'1', // header
        0x82, // input size = 2
        0x82, // output size = 2
        0x81, 0x00, // record: offset +1, empty payload
        0x00, 0x00, 0x00, 0x00, // input crc32
        0x00, 0x00, 0x00, 0x00, // output crc32
        0x00, 0x00, 0x00, 0x00, // patch crc32
    ];

    let ups = UPS::parse(&patch).expect("patch should parse");

    assert_eq!(ups.records.len(), 1);
    assert_eq!(ups.records[0].offset, 1);
    assert!(ups.records[0].payload.is_empty());
}

#[test]
fn parses_a_multibyte_offset() {
    // 128 encodes as the two-byte VLI [0x00, 0x80]; this exercises the
    // continuation-byte path of the decoder and the record-size accounting.
    let patch = [
        b'U', b'P', b'S', b'1', // header
        0x80, // input size = 0
        0x80, // output size = 0
        0x00, 0x80, 0xEE, 0x00, // record: offset +128, xor [EE]
        0x00, 0x00, 0x00, 0x00, // input crc32
        0x00, 0x00, 0x00, 0x00, // output crc32
        0x00, 0x00, 0x00, 0x00, // patch crc32
    ];

    let ups = UPS::parse(&patch).expect("patch should parse");

    assert_eq!(ups.records.len(), 1);
    assert_eq!(ups.records[0].offset, 128);
    assert_eq!(ups.records[0].payload, vec![0xEE]);
}

#[test]
fn parses_a_patch_with_no_records() {
    // Header + sizes followed immediately by the footer.
    let patch = [
        b'U', b'P', b'S', b'1', // header
        0x80, // input size = 0
        0x80, // output size = 0
        0x00, 0x00, 0x00, 0x00, // input crc32
        0x00, 0x00, 0x00, 0x00, // output crc32
        0x00, 0x00, 0x00, 0x00, // patch crc32
    ];

    let ups = UPS::parse(&patch).expect("patch should parse");

    assert!(ups.records.is_empty());
}

#[test]
fn rejects_invalid_header() {
    let patch = [
        b'N', b'O', b'P', b'E', // wrong magic
        0x80, 0x80, // sizes
        0x00, 0x00, 0x00, 0x00, // input crc32
        0x00, 0x00, 0x00, 0x00, // output crc32
        0x00, 0x00, 0x00, 0x00, // patch crc32
    ];

    assert!(matches!(
        UPS::parse(&patch),
        Err(RPParseError::InvalidHeader)
    ));
}

#[test]
fn rejects_patch_shorter_than_header() {
    let patch = [b'U', b'P', b'S'];

    assert!(matches!(
        UPS::parse(&patch),
        Err(RPParseError::UnexpectedEof)
    ));
}

#[test]
fn missing_footer_is_an_error() {
    // The trailing block is only 11 bytes, so it cannot be the 12-byte footer.
    let patch = [
        b'U', b'P', b'S', b'1', // header
        0x84, // input size = 4
        0x84, // output size = 4
        0x81, 0xAA, 0xBB, 0x00, // one record
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // only 11 bytes
    ];

    assert!(matches!(
        UPS::parse(&patch),
        Err(RPParseError::MissingFooter)
    ));
}
