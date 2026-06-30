//! Decode the captured golden streams back to their inputs.
//!
//! This checks that both decoders reproduce the original bytes for every
//! captured packed stream, independent of the round-trip tests.

mod common;

use common::golden::{ALTERNATE, LARGE, NUMBERS, ZEROES};

fn check(data: &[u8], table: &[(i32, usize, &[u8])]) {
    for &(_level, prefix, packed) in table {
        let want = &data[..prefix];

        let mut out = vec![0u8; prefix];
        let n = brieflz::depack_safe(packed, &mut out, prefix).unwrap();
        assert_eq!(n, prefix);
        assert_eq!(&out[..], want);

        let mut out2 = vec![0u8; prefix];
        let n2 = brieflz::depack(packed, &mut out2, prefix);
        assert_eq!(n2, prefix);
        assert_eq!(&out2[..], want);
    }
}

#[test]
fn numbers_decode_golden() {
    check(common::NUMBERS, NUMBERS);
}

#[test]
fn zeroes_decode_golden() {
    check(common::ZEROES, ZEROES);
}

#[test]
fn alternate_decode_golden() {
    check(common::ALTERNATE, ALTERNATE);
}

#[test]
fn large_decode_golden() {
    for &(_level, input, packed) in LARGE {
        let mut out = vec![0u8; input.len()];
        assert_eq!(
            brieflz::depack_safe(packed, &mut out, input.len()).unwrap(),
            input.len()
        );
        assert_eq!(&out[..], input);

        let mut out2 = vec![0u8; input.len()];
        assert_eq!(brieflz::depack(packed, &mut out2, input.len()), input.len());
        assert_eq!(&out2[..], input);
    }
}
