//! Decoder edge cases: empty input and the malformed-stream negative path.

mod common;

use common::ERRORS;

/// Every malformed stream makes the safe decoder report an error.
#[test]
fn safe_decoder_rejects_malformed() {
    let mut sink = vec![0u8; 4093];
    for (i, case) in ERRORS.iter().enumerate() {
        let src = &case.data[..case.src_size];
        let res = brieflz::depack_safe(src, &mut sink, case.depacked_size);
        assert_eq!(
            res,
            Err(brieflz::Error::MalformedInput),
            "error case {i} should fail"
        );
    }
}

/// A `depacked_size` larger than `dst` makes the safe decoder report an error
/// instead of writing past the buffer.
#[test]
fn safe_decoder_rejects_too_small_output() {
    assert_eq!(
        brieflz::depack_safe(&[0x42], &mut [], 1),
        Err(brieflz::Error::MalformedInput)
    );
}

/// The safe decoder reports an error when the match length overflows.
#[test]
fn safe_decoder_rejects_gamma_length_overflow() {
    let src = [
        0x42, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFC, 0xFF, 0x00, 0x00, 0x00,
    ];
    let mut dst = [0u8; 2];

    assert_eq!(
        brieflz::depack_safe(&src, &mut dst, 2),
        Err(brieflz::Error::MalformedInput)
    );
}

/// Both decoders return zero for an empty request and leave `dst` untouched.
#[test]
fn empty_decode() {
    let src = [0u8; 0];
    let mut dst = [0xAAu8; 8];
    assert_eq!(brieflz::depack_safe(&src, &mut dst, 0), Ok(0));
    assert_eq!(dst, [0xAA; 8]);
    assert_eq!(brieflz::depack(&src, &mut dst, 0), 0);
    assert_eq!(dst, [0xAA; 8]);
}

/// Empty compression returns zero and writes nothing to `dst`.
#[test]
fn empty_pack() {
    let mut dst = [0xAAu8; 8];
    let mut work = vec![0u32; brieflz::workmem_size() / 4];
    assert_eq!(brieflz::pack(&[], &mut dst, &mut work), 0);
    assert_eq!(dst, [0xAA; 8]);
    for level in 1..=10 {
        let words = brieflz::workmem_size_level(0, level).unwrap() / 4;
        let mut w = vec![0u32; words.max(1)];
        assert_eq!(brieflz::pack_level(&[], &mut dst, &mut w, level), Ok(0));
        assert_eq!(dst, [0xAA; 8], "level {level} wrote to dst");
    }
}
