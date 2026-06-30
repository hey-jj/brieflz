//! Round-trip parity over the embedded datasets and generated shapes.
//!
//! For each dataset, level, and prefix length, compress then decompress with
//! both decoders and check the result equals the input. Also check the packed
//! size stays within the published bound.

mod common;

use common::{pack_to_vec, Lcg, ALTERNATE, NUMBERS, ZEROES};

fn sweep(data: &[u8], levels: std::ops::RangeInclusive<i32>) {
    for level in levels {
        for i in 1..data.len() {
            let prefix = &data[..i];
            let packed = pack_to_vec(prefix, level);
            assert!(
                packed.len() <= brieflz::max_packed_size(i),
                "size bound at level {level} prefix {i}"
            );

            let mut out = vec![0u8; i];
            let n = brieflz::depack_safe(&packed, &mut out, i).unwrap();
            assert_eq!(n, i);
            assert_eq!(&out[..], prefix, "depack_safe level {level} prefix {i}");

            let mut out2 = vec![0u8; i];
            let n2 = brieflz::depack(&packed, &mut out2, i);
            assert_eq!(n2, i);
            assert_eq!(&out2[..], prefix, "depack level {level} prefix {i}");
        }
    }
}

#[test]
fn numbers_roundtrip() {
    sweep(NUMBERS, 1..=10);
}

#[test]
fn alternate_roundtrip() {
    sweep(ALTERNATE, 1..=10);
}

#[test]
fn zeroes_roundtrip() {
    // The codec's own suite skips level 10 here for runtime. Cover it too.
    sweep(ZEROES, 1..=10);
}

/// Random base with a matched region grown from front to back.
#[test]
fn random_growing_match_roundtrip() {
    let size = 4093;
    for level in 1..=10 {
        for i in (0..size / 2).step_by(311) {
            let mut buf = vec![0u8; size];
            Lcg::new(42).fill(&mut buf);
            for j in 0..i {
                buf[size - i + j] = buf[j];
            }
            check_roundtrip(&buf, level);
        }
    }
}

/// Random base with a growing 0xFF run at the back.
#[test]
fn random_run_at_back_roundtrip() {
    let size = 4093;
    for level in 1..=9 {
        for i in (0..size / 2).step_by(331) {
            let mut buf = vec![0u8; size];
            Lcg::new(42).fill(&mut buf);
            for b in buf.iter_mut().skip(size - i) {
                *b = 0xFF;
            }
            check_roundtrip(&buf, level);
        }
    }
}

/// Random base with a growing 0xFF run at the front.
#[test]
fn random_run_at_front_roundtrip() {
    let size = 4093;
    for level in 1..=9 {
        for i in (0..size / 2).step_by(331) {
            let mut buf = vec![0u8; size];
            Lcg::new(42).fill(&mut buf);
            for b in buf.iter_mut().take(i) {
                *b = 0xFF;
            }
            check_roundtrip(&buf, level);
        }
    }
}

fn check_roundtrip(buf: &[u8], level: i32) {
    let packed = pack_to_vec(buf, level);
    assert!(packed.len() <= brieflz::max_packed_size(buf.len()));

    let mut out = vec![0u8; buf.len()];
    assert_eq!(
        brieflz::depack_safe(&packed, &mut out, buf.len()).unwrap(),
        buf.len()
    );
    assert_eq!(out, buf, "depack_safe level {level}");

    let mut out2 = vec![0u8; buf.len()];
    assert_eq!(brieflz::depack(&packed, &mut out2, buf.len()), buf.len());
    assert_eq!(out2, buf, "depack level {level}");
}

/// A one-byte input packs to one byte and round-trips.
#[test]
fn single_byte_input() {
    for level in 1..=10 {
        let packed = pack_to_vec(&[0x42], level);
        assert_eq!(packed, &[0x42]);
        let mut out = [0u8; 1];
        assert_eq!(brieflz::depack_safe(&packed, &mut out, 1).unwrap(), 1);
        assert_eq!(out[0], 0x42);
        assert_eq!(brieflz::depack(&packed, &mut out, 1), 1);
        assert_eq!(out[0], 0x42);
    }
}

/// A stream packed at any level decodes with both decoders.
#[test]
fn cross_level_interop() {
    let data = b"the quick brown fox jumps over the quick brown fox";
    for level in 1..=10 {
        let packed = pack_to_vec(data, level);
        let mut a = vec![0u8; data.len()];
        let mut b = vec![0u8; data.len()];
        assert_eq!(
            brieflz::depack_safe(&packed, &mut a, data.len()).unwrap(),
            data.len()
        );
        assert_eq!(brieflz::depack(&packed, &mut b, data.len()), data.len());
        assert_eq!(a, data);
        assert_eq!(b, data);
    }
}
