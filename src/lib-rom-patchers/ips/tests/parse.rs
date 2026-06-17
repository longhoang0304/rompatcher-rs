use crate::rp::cores::rp_parser::{RPParseError, RPParser};
use crate::rp::ips::ips::IPS;
use crate::rp::ips::ips_record::IPSRecord;

#[test]
fn parses_a_single_data_record() {
    let patch = [
        b'P', b'A', b'T', b'C', b'H', // header
        0x00, 0x00, 0x10, // offset = 16
        0x00, 0x03, // size = 3
        0xAA, 0xBB, 0xCC, // payload
        b'E', b'O', b'F', // footer
    ];

    let ips = IPS::parse(&patch).expect("patch should parse");

    assert_eq!(ips.records.len(), 1);
    match &ips.records[0] {
        IPSRecord::Data(d) => {
            assert_eq!(d.offset, 16);
            assert_eq!(d.size, 3);
            assert_eq!(d.payload, vec![0xAA, 0xBB, 0xCC]);
        }
        other => panic!("expected a data record, got {other:?}"),
    }
}

#[test]
fn parses_an_rle_record() {
    let patch = [
        b'P', b'A', b'T', b'C', b'H', // header
        0x00, 0x00, 0x05, // offset = 5
        0x00, 0x00, // size = 0 -> RLE record
        0x00, 0x04, // rle_size = 4
        0x7F, // value
        b'E', b'O', b'F', // footer
    ];

    let ips = IPS::parse(&patch).expect("patch should parse");

    assert_eq!(ips.records.len(), 1);
    match &ips.records[0] {
        IPSRecord::RLE(r) => {
            assert_eq!(r.offset, 5);
            assert_eq!(r.rle_size, 4);
            assert_eq!(r.value, 0x7F);
        }
        other => panic!("expected an RLE record, got {other:?}"),
    }
}

#[test]
fn parses_multiple_records_in_order() {
    let patch = [
        b'P', b'A', b'T', b'C', b'H', // header
        0x00, 0x00, 0x00, 0x00, 0x02, 0x11, 0x22, // data @0, len 2
        0x00, 0x00, 0x08, 0x00, 0x00, 0x00, 0x03, 0xFF, // rle @8, size 3, value FF
        b'E', b'O', b'F', // footer
    ];

    let ips = IPS::parse(&patch).expect("patch should parse");

    assert_eq!(ips.records.len(), 2);
    assert!(matches!(ips.records[0], IPSRecord::Data(_)));
    assert!(matches!(ips.records[1], IPSRecord::RLE(_)));
}

// Regression test for the footer ambiguity: the bytes "EOF" inside a payload
// must be consumed as data, not mistaken for the terminator.
#[test]
fn eof_bytes_inside_payload_are_not_treated_as_footer() {
    let patch = [
        b'P', b'A', b'T', b'C', b'H', // header
        0x00, 0x00, 0x00, // offset = 0
        0x00, 0x05, // size = 5
        0x01, b'E', b'O', b'F', 0x02, // payload = [01, "EOF", 02]
        b'E', b'O', b'F', // footer
    ];

    let ips = IPS::parse(&patch).expect("patch should parse");

    assert_eq!(ips.records.len(), 1);
    match &ips.records[0] {
        IPSRecord::Data(d) => assert_eq!(d.payload, vec![0x01, b'E', b'O', b'F', 0x02]),
        other => panic!("expected a data record, got {other:?}"),
    }
}

#[test]
fn rejects_invalid_header() {
    let patch = [b'N', b'O', b'P', b'E', b'!', b'E', b'O', b'F'];

    assert!(matches!(
        IPS::parse(&patch),
        Err(RPParseError::InvalidHeader)
    ));
}

#[test]
fn rejects_patch_shorter_than_header() {
    let patch = [b'P', b'A', b'T'];

    assert!(matches!(
        IPS::parse(&patch),
        Err(RPParseError::UnexpectedEof)
    ));
}

#[test]
fn missing_footer_is_an_error() {
    let patch = [
        b'P', b'A', b'T', b'C', b'H', // header
        0x00, 0x00, 0x00, // offset = 0
        0x00, 0x02, // size = 2
        0xAA, 0xBB, // payload, but no footer follows
    ];

    assert!(matches!(
        IPS::parse(&patch),
        Err(RPParseError::MissingFooter)
    ));
}

// Offsets are 24-bit big-endian; this exercises the upper bytes that the
// small-offset tests never touch.
#[test]
fn parses_a_large_u24_offset() {
    let patch = [
        b'P', b'A', b'T', b'C', b'H', // header
        0x01, 0x02, 0x03, // offset = 0x010203 = 66051
        0x00, 0x01, // size = 1
        0xAA, // payload
        b'E', b'O', b'F', // footer
    ];

    let ips = IPS::parse(&patch).expect("patch should parse");

    assert_eq!(ips.records.len(), 1);
    match &ips.records[0] {
        IPSRecord::Data(d) => {
            assert_eq!(d.offset, 0x01_02_03);
            assert_eq!(d.payload, vec![0xAA]);
        }
        other => panic!("expected a data record, got {other:?}"),
    }
}

// A record whose declared size runs past the available bytes is a malformed
// patch, surfaced as UnexpectedEof by the byte reader.
#[test]
fn truncated_record_is_unexpected_eof() {
    let patch = [
        b'P', b'A', b'T', b'C', b'H', // header
        0x00, 0x00, 0x00, // offset = 0
        0x00, 0x05, // size = 5, but only 1 payload byte present
        0xAA, //
        b'E', b'O', b'F', // footer
    ];

    assert!(matches!(
        IPS::parse(&patch),
        Err(RPParseError::UnexpectedEof)
    ));
}
