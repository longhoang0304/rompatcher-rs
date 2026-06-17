use crate::rp::cores::rp_parser::RPParser;
use crate::rp::cores::rp_patcher::{RPPatchError, RPPatcher};
use crate::rp::ups::ups::UPS;
use crate::rp::ups::ups_record::UPSRecord;

// Convenience constructor for the tests; the CRC32 fields are unused by patch().
fn ups(records: Vec<UPSRecord>, inp_rom_size: usize, out_rom_size: usize) -> UPS {
    UPS {
        records,
        inp_rom_size,
        out_rom_size,
        inp_rom_crc32: 0,
        out_rom_crc32: 0,
        patch_crc32: 0,
    }
}

#[test]
fn applies_a_single_record() {
    let rom = vec![0u8; 4];
    let patch = ups(vec![UPSRecord::new(1, vec![0xAA, 0xBB])], 4, 4);

    let result = UPS::patch(&rom, &patch).expect("patch should apply");

    // rom is all zeros, so the output equals the xor payload at offset 1.
    assert_eq!(result.patched_rom, vec![0x00, 0xAA, 0xBB, 0x00]);
    assert_eq!(result.events.len(), 1);
}

#[test]
fn xor_combines_with_existing_rom_bytes() {
    let rom = vec![0x0F, 0x0F, 0x0F, 0x0F];
    let patch = ups(vec![UPSRecord::new(0, vec![0xFF, 0xF0])], 4, 4);

    let result = UPS::patch(&rom, &patch).expect("patch should apply");

    assert_eq!(result.patched_rom, vec![0xF0, 0xFF, 0x0F, 0x0F]);
}

#[test]
fn applies_multiple_records() {
    // Second record's relative offset is measured from one past the previous
    // chunk's terminator, exercising the cursor advance.
    let rom = vec![0u8; 4];
    let patch = ups(
        vec![
            UPSRecord::new(1, vec![0xAA]),
            UPSRecord::new(0, vec![0xBB]),
        ],
        4,
        4,
    );

    let result = UPS::patch(&rom, &patch).expect("patch should apply");

    assert_eq!(result.patched_rom, vec![0x00, 0xAA, 0x00, 0xBB]);
    assert_eq!(result.events.len(), 2);
}

#[test]
fn expands_rom_to_output_size() {
    // Input is shorter than the output; patch() must grow the ROM before
    // writing past the original end.
    let rom = vec![0u8; 2];
    let patch = ups(vec![UPSRecord::new(2, vec![0xCC])], 2, 4);

    let result = UPS::patch(&rom, &patch).expect("patch should apply");

    assert_eq!(result.patched_rom, vec![0x00, 0x00, 0xCC, 0x00]);
}

#[test]
fn patch_returns_new_rom_without_mutating_source() {
    let rom = vec![1u8, 2, 3, 4];
    let patch = ups(vec![UPSRecord::new(0, vec![0xFF, 0xFF])], 4, 4);

    let result = UPS::patch(&rom, &patch).expect("patch should apply");

    assert_eq!(rom, vec![1, 2, 3, 4], "source rom must be untouched");
    assert_eq!(result.patched_rom, vec![0xFE, 0xFD, 3, 4]);
}

#[test]
fn record_starting_past_end_of_rom_is_an_error() {
    let rom = vec![0u8; 4];
    let patch = ups(vec![UPSRecord::new(5, vec![0xAA])], 4, 4); // write offset 5 > len 4

    assert!(matches!(
        UPS::patch(&rom, &patch),
        Err(RPPatchError::OverflowPatchRecordEof(5, 1, 4))
    ));
}

// End-to-end: parse a patch, then apply it to a blank ROM that the patch grows.
#[test]
fn round_trip_parse_then_patch() {
    let patch = [
        b'U', b'P', b'S', b'1', // header
        0x84, // input size = 4
        0x85, // output size = 5
        0x81, 0xAA, 0x00, // record 1: offset +1, xor [AA]
        0x80, 0xBB, 0xCC, 0x00, // record 2: offset +0, xor [BB, CC]
        0x00, 0x00, 0x00, 0x00, // input crc32
        0x00, 0x00, 0x00, 0x00, // output crc32
        0x00, 0x00, 0x00, 0x00, // patch crc32
    ];

    let parsed = UPS::parse(&patch).expect("patch should parse");
    let rom = vec![0u8; 4];

    let result = UPS::patch(&rom, &parsed).expect("patch should apply");

    assert_eq!(result.patched_rom, vec![0x00, 0xAA, 0x00, 0xBB, 0xCC]);
}
