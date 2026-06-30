//! Shared constants, hashing, and cost heuristics used by every parser.
//!
//! These match the wire-format math exactly. The cost functions and match
//! comparison heuristics decide which literals and matches each parser emits,
//! so their integer arithmetic must stay bit-for-bit identical to keep the
//! compressed output deterministic.

/// Number of hash bits. Sets the lookup table size and the scratch size.
pub const HASH_BITS: u32 = 17;

/// Lookup table entry count. Equals `1 << HASH_BITS`.
pub const LOOKUP_SIZE: usize = 1 << HASH_BITS;

/// Sentinel stored in the lookup table for "no earlier position".
///
/// This is the all-ones `u32`, the same value the encoder uses to mark an
/// empty hash slot or an absent chain link.
pub const NO_MATCH_POS: u32 = u32::MAX;

/// Largest source size the encoder accepts.
///
/// Positions live in `u32`, so the source size must stay below the all-ones
/// value to leave room for [`NO_MATCH_POS`].
pub const WORD_MAX: u32 = u32::MAX;

/// Hash four bytes at `p` into `bits` bits.
///
/// Reads four bytes little-endian and applies Fibonacci hashing (Knuth's
/// multiplicative hash). The multiply wraps modulo `2^32`.
#[inline]
pub fn hash4_bits(p: &[u8], bits: u32) -> u32 {
    let val = u32::from(p[0])
        | (u32::from(p[1]) << 8)
        | (u32::from(p[2]) << 16)
        | (u32::from(p[3]) << 24);
    val.wrapping_mul(2_654_435_761) >> (32 - bits)
}

/// Hash four bytes at `p` using the default [`HASH_BITS`].
#[inline]
pub fn hash4(p: &[u8]) -> u32 {
    hash4_bits(p, HASH_BITS)
}

/// Floor of the base-2 logarithm of `n`. Position of the highest set bit.
///
/// Defined only for `n > 0`. Callers uphold that invariant.
#[inline]
pub fn log2(n: u32) -> u32 {
    debug_assert!(n > 0);
    31 - n.leading_zeros()
}

/// Bit count to encode `n` with the gamma2 universal code. Equals `2 * log2(n)`.
///
/// Defined only for `n >= 2`.
#[inline]
pub fn gamma_cost(n: u32) -> u32 {
    debug_assert!(n >= 2);
    2 * log2(n)
}

/// Bit cost of a match with the given offset and length.
///
/// `offs` is the back distance minus one. The cost is one tag bit, the length
/// gamma, the offset-high gamma, and eight bits for the offset low byte.
#[inline]
pub fn match_cost(offs: u32, len: u32) -> u32 {
    1 + gamma_cost(len - 2) + gamma_cost((offs >> 8) + 2) + 8
}

/// Is the candidate match `(new_pos, new_len)` better than `(pos, len)` at `cur`?
///
/// Compares back distances scaled by eight. A longer match wins outright. An
/// equal-plus-one length wins when its scaled distance is no larger.
// The `>= len + 1` term reads as the inverse of the `> len + 1` term above it
// and states the heuristic directly. Keep it as written.
#[allow(clippy::int_plus_one)]
#[inline]
pub fn match_better(cur: u32, new_pos: u32, new_len: u32, pos: u32, len: u32) -> bool {
    // The old match may be the NO_MATCH_POS sentinel, so the subtraction can
    // wrap. The math relies on that wrap, so wrap here too.
    let offs = cur.wrapping_sub(pos).wrapping_sub(1);
    let new_offs = cur.wrapping_sub(new_pos).wrapping_sub(1);
    (new_len > len + 1) || (new_len >= len + 1 && new_offs / 8 <= offs)
}

/// Should a match at `cur + 1` displace the current match, forcing a literal now?
///
/// Three cases, each pairing a length gain with a distance shrink. The integer
/// divisions are part of the heuristic and must stay exact.
#[inline]
pub fn next_match_better(cur: u32, new_pos: u32, new_len: u32, pos: u32, len: u32) -> bool {
    let offs = cur.wrapping_sub(pos).wrapping_sub(1);
    let new_offs = (cur + 1).wrapping_sub(new_pos).wrapping_sub(1);
    (new_len > len + 1 && new_offs / 8 < offs)
        || (new_len > len && new_offs < offs)
        || (new_len >= len && new_offs < offs / 4)
}
