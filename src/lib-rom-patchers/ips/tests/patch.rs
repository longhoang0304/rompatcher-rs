use crate::rp::cores::rp_parser::RPParser;
use crate::rp::cores::rp_patcher::RPPatcher;
use crate::rp::ips::ips::IPS;
use crate::rp::ips::ips_record::IPSRecord;

#[test]
fn applies_a_data_record() {
    let rom = vec![0u8; 8];
    let records = vec![IPSRecord::new_with_data(2, 3, &[0xAA, 0xBB, 0xCC])];
    let ips = IPS {
        records,
    };

    let result = IPS::patch(&rom, &ips).expect("patch should apply");

    assert_eq!(result.patched_rom, vec![0, 0, 0xAA, 0xBB, 0xCC, 0, 0, 0]);
    assert_eq!(result.events.len(), 1);
}

// Regression test for bug #1: an RLE record used to panic in copy_from_slice.
// It must fill the run in place without panicking.
#[test]
fn applies_an_rle_record_without_panicking() {
    let rom = vec![0u8; 8];
    let records = vec![IPSRecord::new_with_rle(2, 4, 0xAB)];
    let ips = IPS {
        records,
    };

    let result = IPS::patch(&rom, &ips).expect("patch should apply");

    assert_eq!(result.patched_rom, vec![0, 0, 0xAB, 0xAB, 0xAB, 0xAB, 0, 0]);
}

// Regression test for bug #2: patch() must return the patched ROM and must not
// mutate the caller's source ROM.
#[test]
fn patch_returns_new_rom_without_mutating_source() {
    let rom = vec![1u8, 2, 3, 4];
    let records = vec![IPSRecord::new_with_data(0, 2, &[0xFF, 0xFF])];
    let ips = IPS {
        records,
    };

    let result = IPS::patch(&rom, &ips).expect("patch should apply");

    assert_eq!(rom, vec![1, 2, 3, 4], "source rom must be untouched");
    assert_eq!(result.patched_rom, vec![0xFF, 0xFF, 3, 4]);
}

#[test]
fn applies_multiple_records() {
    let rom = vec![0u8; 8];
    let records = vec![
        IPSRecord::new_with_data(0, 2, &[0x11, 0x22]),
        IPSRecord::new_with_rle(4, 3, 0x99),
    ];
    let ips = IPS {
        records,
    };
    

    let result = IPS::patch(&rom, &ips).expect("patch should apply");

    assert_eq!(result.patched_rom, vec![0x11, 0x22, 0, 0, 0x99, 0x99, 0x99, 0]);
    assert_eq!(result.events.len(), 2);
}

// A data record reaching past the input ROM grows the output, zero-filling
// any gap (IPS offsets are 24-bit and may extend the ROM).
#[test]
fn data_record_past_end_of_rom_expands_it() {
    let rom = vec![0u8; 4];
    let records = vec![IPSRecord::new_with_data(2, 4, &[1, 2, 3, 4])]; // 2 + 4 > 4
    let ips = IPS {
        records,
    };

    let result = IPS::patch(&rom, &ips).expect("patch should expand the rom");

    assert_eq!(result.patched_rom, vec![0, 0, 1, 2, 3, 4]);
}

#[test]
fn rle_record_past_end_of_rom_expands_it() {
    let rom = vec![0u8; 4];
    let records = vec![IPSRecord::new_with_rle(3, 4, 0xFF)]; // 3 + 4 > 4
    let ips = IPS {
        records,
    };

    let result = IPS::patch(&rom, &ips).expect("patch should expand the rom");

    assert_eq!(result.patched_rom, vec![0, 0, 0, 0xFF, 0xFF, 0xFF, 0xFF]);
}

// A gap between the input ROM end and a far record is zero-filled.
#[test]
fn gap_before_an_expanding_record_is_zero_filled() {
    let rom = vec![0xAAu8; 2];
    let records = vec![IPSRecord::new_with_data(5, 2, &[0x11, 0x22])];
    let ips = IPS {
        records,
    };

    let result = IPS::patch(&rom, &ips).expect("patch should expand the rom");

    assert_eq!(result.patched_rom, vec![0xAA, 0xAA, 0, 0, 0, 0x11, 0x22]);
}

// Boundary: a record whose end lands exactly on rom_len must succeed — guards
// against an off-by-one (`<` vs `<=`) in resolve_range.
#[test]
fn data_record_ending_exactly_at_rom_end_succeeds() {
    let rom = vec![0u8; 4];
    let records = vec![IPSRecord::new_with_data(2, 2, &[0xAA, 0xBB])]; // 2 + 2 == 4
    let ips = IPS {
        records,
    };

    let result = IPS::patch(&rom, &ips).expect("patch should apply");

    assert_eq!(result.patched_rom, vec![0, 0, 0xAA, 0xBB]);
}

#[test]
fn rle_record_ending_exactly_at_rom_end_succeeds() {
    let rom = vec![0u8; 4];
    let records = vec![IPSRecord::new_with_rle(0, 4, 0xCC)]; // 0 + 4 == 4
    let ips = IPS {
        records,
    };

    let result = IPS::patch(&rom, &ips).expect("patch should apply");

    assert_eq!(result.patched_rom, vec![0xCC, 0xCC, 0xCC, 0xCC]);
}

// End-to-end: parse a patch, then apply it to a blank ROM.
#[test]
fn round_trip_parse_then_patch() {
    let patch = [
        b'P', b'A', b'T', b'C', b'H', // header
        0x00, 0x00, 0x01, 0x00, 0x02, 0xDE, 0xAD, // data @1, len 2
        0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x02, 0xBE, // rle @4, size 2, value BE
        b'E', b'O', b'F', // footer
    ];

    let records = IPS::parse(&patch).expect("patch should parse");
    let rom = vec![0u8; 8];

    let result = IPS::patch(&rom, &records).expect("patch should apply");

    assert_eq!(result.patched_rom, vec![0, 0xDE, 0xAD, 0, 0xBE, 0xBE, 0, 0]);
}
