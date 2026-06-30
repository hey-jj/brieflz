//! API surface: size formulas, workmem sizes, and invalid-level handling.

const LOOKUP_SIZE: usize = 1 << 17;
const WORD: usize = 4;

#[test]
fn max_packed_size_formula() {
    assert_eq!(brieflz::max_packed_size(0), 64);
    assert_eq!(brieflz::max_packed_size(8), 73);
    for n in [1usize, 7, 100, 1024, 1_000_000] {
        assert_eq!(brieflz::max_packed_size(n), n + n / 8 + 64);
    }
}

#[test]
fn level1_workmem_is_lookup_table() {
    assert_eq!(brieflz::workmem_size(), LOOKUP_SIZE * WORD);
    // Level 1 and level 2 report the same fixed table size.
    assert_eq!(brieflz::workmem_size_level(0, 1), Some(LOOKUP_SIZE * WORD));
}

#[test]
fn workmem_size_level_per_level() {
    let n = 4093usize;
    // Levels 1 and 2 share the level-1 size.
    assert_eq!(brieflz::workmem_size_level(n, 1), Some(LOOKUP_SIZE * WORD));
    assert_eq!(brieflz::workmem_size_level(n, 2), Some(LOOKUP_SIZE * WORD));
    // Buckets scale with bucket size.
    assert_eq!(
        brieflz::workmem_size_level(n, 3),
        Some(LOOKUP_SIZE * 2 * WORD)
    );
    assert_eq!(
        brieflz::workmem_size_level(n, 4),
        Some(LOOKUP_SIZE * 4 * WORD)
    );
    // Levels 5 to 7 share the leparse size.
    let leparse = if LOOKUP_SIZE < 2 * n {
        3 * n
    } else {
        n + LOOKUP_SIZE
    } * WORD;
    for level in 5..=7 {
        assert_eq!(brieflz::workmem_size_level(n, level), Some(leparse));
    }
    // Levels 8 to 10 share the btparse size.
    let btparse = (5 * n + 3 + LOOKUP_SIZE) * WORD;
    for level in 8..=10 {
        assert_eq!(brieflz::workmem_size_level(n, level), Some(btparse));
    }
}

#[test]
fn workmem_size_level_large_input_uses_three_times() {
    // When the input is large the leparse size is three words per byte.
    let n = LOOKUP_SIZE; // 2 * n >= LOOKUP_SIZE
    assert_eq!(brieflz::workmem_size_level(n, 5), Some(3 * n * WORD));
}

#[test]
fn invalid_level_has_no_workmem() {
    for level in [0, 11, 100, 255] {
        assert_eq!(brieflz::workmem_size_level(4093, level), None);
    }
}

#[test]
fn invalid_level_pack_errors() {
    let mut dst = vec![0u8; brieflz::max_packed_size(4)];
    let mut work = vec![0u32; LOOKUP_SIZE];
    for level in [0, 11, 100, 255] {
        assert_eq!(
            brieflz::pack_level(b"data", &mut dst, &mut work, level),
            Err(brieflz::Error::InvalidLevel)
        );
    }
}
