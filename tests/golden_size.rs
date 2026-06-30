//! Guard the golden fixture size so it cannot creep back to a bulk dump.
//!
//! The byte-exact vectors earn their keep, but the fixture once stored the
//! same input ten times and swept every prefix length. This test pins a line
//! and byte ceiling. Cross either one and the build fails, which forces a
//! choice: trim the vectors or raise the ceiling on purpose.

use std::fs;

const GOLDEN: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/common/golden.rs");

const MAX_LINES: usize = 5_000;
const MAX_BYTES: usize = 400_000;

#[test]
fn golden_fixture_stays_small() {
    let text = fs::read_to_string(GOLDEN).expect("read golden fixture");
    let lines = text.lines().count();
    let bytes = text.len();
    assert!(
        lines <= MAX_LINES,
        "golden.rs has {lines} lines, ceiling is {MAX_LINES}"
    );
    assert!(
        bytes <= MAX_BYTES,
        "golden.rs is {bytes} bytes, ceiling is {MAX_BYTES}"
    );
}
