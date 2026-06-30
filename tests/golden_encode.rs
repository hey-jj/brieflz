//! Byte-exact encoder parity against captured golden output.
//!
//! Round-trip tests prove the decoder. They do not prove the encoder makes
//! the same choices, since a different valid parse would still round-trip.
//! These checks compare the exact packed bytes for every dataset, prefix, and
//! level against output captured from a known-good encoder.

mod common;

use common::golden::{ALTERNATE, LARGE, LARGE_INPUT, NUMBERS, ZEROES};

fn check(data: &[u8], table: &[(u8, usize, &[u8])]) {
    for &(level, prefix, expected) in table {
        let got = common::pack_to_vec(&data[..prefix], level);
        assert_eq!(
            got, expected,
            "encoder mismatch at level {level} prefix {prefix}"
        );
    }
}

#[test]
fn numbers_match_golden() {
    check(common::NUMBERS, NUMBERS);
}

#[test]
fn zeroes_match_golden() {
    check(common::ZEROES, ZEROES);
}

#[test]
fn alternate_match_golden() {
    check(common::ALTERNATE, ALTERNATE);
}

#[test]
fn large_matches_golden() {
    for &(level, expected) in LARGE {
        let got = common::pack_to_vec(LARGE_INPUT, level);
        assert_eq!(
            got, expected,
            "encoder mismatch at level {level} on large input"
        );
    }
}
