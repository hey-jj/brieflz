//! Feed random bytes to the safe decoder. It must never panic.
//!
//! This guards the bounds and overflow logic. The safe decoder either decodes
//! or returns an error, but never reads or writes out of range. Slide the
//! start offset across each random buffer the way the codec's own suite does.

mod common;

use common::Lcg;

#[test]
fn safe_decoder_never_panics_on_random() {
    let size = 4093 / 2;
    let mut rng = Lcg::new(1);
    let mut buf = vec![0u8; size];
    let mut sink = vec![0u8; 4093];

    let sink_len = sink.len();
    for _ in 0..256 {
        rng.fill(&mut buf);
        for j in 0..size / 2 {
            let src = &buf[j..];
            let _ = brieflz::depack_safe(src, &mut sink, sink_len);
        }
    }
}

#[test]
fn pack_then_depack_random_lengths() {
    let mut rng = Lcg::new(7);
    for len in [0usize, 1, 2, 3, 4, 5, 7, 13, 64, 200, 1000] {
        let mut data = vec![0u8; len];
        rng.fill(&mut data);
        for level in 1..=10 {
            let packed = common::pack_to_vec(&data, level);
            let mut out = vec![0u8; len];
            let n = brieflz::depack(&packed, &mut out, len);
            assert_eq!(n, len);
            assert_eq!(out, data, "mismatch at len {len} level {level}");
        }
    }
}
