//! Exercise the length-4 cost gates at their exact thresholds.
//!
//! A length-4 match is only emitted when its back distance is small. Level 1
//! uses the threshold `0x7E00`. The lazy and hash-bucket parsers use
//! `0x3FE00`. The rest of the test suite tops out near a back distance of
//! 2593, far below either threshold, so the gate decision never runs at its
//! boundary. These tests build inputs that place a single length-4 match right
//! at the threshold and confirm the decision flips there.
//!
//! Construction: fill a buffer with an incompressible xorshift stream, then
//! plant a unique 4-byte token at the end and a copy of it `d` bytes earlier.
//! When the encoder reaches the trailing copy it sees one length-4 match whose
//! offset is `d - 1`. With `d - 1 == threshold - 1` the gate accepts and the
//! match replaces four literals, shrinking the output by one byte. With
//! `d - 1 == threshold` the gate rejects and the output matches the no-token
//! baseline. Changing the constant breaks one of the two equalities.

mod common;

/// Incompressible filler. The window has no 4-byte repeat, so the only match
/// is the planted token.
fn fill(buf: &mut [u8]) {
    let mut x: u64 = 0x1234_5678;
    for b in buf.iter_mut() {
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        *b = (x >> 23) as u8;
    }
}

/// Build a buffer of `n` bytes. When `dist` is set, plant a 4-byte token at
/// the end and a copy `dist` bytes before it.
fn build(n: usize, dist: Option<usize>) -> Vec<u8> {
    let mut buf = vec![0u8; n];
    fill(&mut buf);
    if let Some(d) = dist {
        let tok = [0x11u8, 0x22, 0x33, 0x44];
        let end = n - 4;
        let early = end - d;
        buf[early..early + 4].copy_from_slice(&tok);
        buf[end..end + 4].copy_from_slice(&tok);
    }
    buf
}

/// Pack `data` at `level` and return the packed byte count.
fn packed_len(level: u8, data: &[u8]) -> usize {
    common::pack_to_vec(data, level).len()
}

/// Pack, decode with both decoders, and assert the bytes round-trip.
fn assert_roundtrip(level: u8, data: &[u8]) {
    let packed = common::pack_to_vec(data, level);
    let mut out = vec![0u8; data.len()];
    assert_eq!(
        brieflz::depack_safe(&packed, &mut out, data.len()).unwrap(),
        data.len()
    );
    assert_eq!(out, data);
    let mut out2 = vec![0u8; data.len()];
    assert_eq!(brieflz::depack(&packed, &mut out2, data.len()), data.len());
    assert_eq!(out2, data);
}

/// At a gate `threshold` for `level`, the accept boundary emits the match and
/// the reject boundary falls back to literals.
fn check_gate(level: u8, threshold: usize) {
    let n = threshold + 64;
    let baseline = packed_len(level, &build(n, None));
    // dist = threshold places the match offset at threshold - 1, just inside.
    let accept = build(n, Some(threshold));
    // dist = threshold + 1 places the offset at threshold, just outside.
    let reject = build(n, Some(threshold + 1));

    assert!(
        packed_len(level, &accept) < baseline,
        "level {level}: match at offset {} should be emitted",
        threshold - 1
    );
    assert_eq!(
        packed_len(level, &reject),
        baseline,
        "level {level}: match at offset {threshold} should be dropped"
    );

    assert_roundtrip(level, &accept);
    assert_roundtrip(level, &reject);
}

#[test]
fn level1_length4_gate_at_threshold() {
    check_gate(1, 0x7E00);
}

#[test]
fn level4_length4_gate_at_threshold() {
    check_gate(4, 0x3FE00);
}
