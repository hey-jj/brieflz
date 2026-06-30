//! Level 1: greedy parse with single-slot hashing.
//!
//! The lookup table holds the most recent position for each hash. Each step
//! looks up one candidate, measures the match, and emits it when the length
//! gate passes. Otherwise it emits a literal.

use crate::bitstream::BitWriter;
use crate::common::{hash4, NO_MATCH_POS};

/// Compress `src` into `dst` at level 1. Returns the compressed byte count.
///
/// `workmem` holds the single-slot lookup table. Empty input returns zero and
/// writes nothing. One byte writes that byte and returns one.
pub fn pack(src: &[u8], dst: &mut [u8], workmem: &mut [u32]) -> usize {
    let src_size = src.len();
    if src_size == 0 {
        return 0;
    }

    dst[0] = src[0];
    if src_size == 1 {
        return 1;
    }

    let last_match_pos = src_size.saturating_sub(4);
    let lookup = &mut workmem[..crate::common::LOOKUP_SIZE];
    lookup.fill(NO_MATCH_POS);

    let mut bw = BitWriter::new(dst, 1);
    let mut hash_pos = 0usize;
    let mut cur = 1usize;

    while cur <= last_match_pos {
        while hash_pos < cur {
            lookup[hash4(&src[hash_pos..]) as usize] = hash_pos as u32;
            hash_pos += 1;
        }

        let pos = lookup[hash4(&src[cur..]) as usize];
        let mut len = 0usize;

        if pos != NO_MATCH_POS {
            let pos = pos as usize;
            let len_limit = src_size - cur;
            while len < len_limit && src[pos + len] == src[cur + len] {
                len += 1;
            }
        }

        let pos_u = pos as usize;
        // Encode a match when it beats four literals. Length four only counts
        // when its offset is small, since a distant length-four match costs
        // more bits than four literals.
        if len > 4 || (len == 4 && (cur as u32) - pos - 1 < 0x7E00) {
            let offs = (cur - pos_u - 1) as u32;
            bw.put_match(len as u32, offs);
            cur += len;
        } else {
            bw.put_literal(src[cur]);
            cur += 1;
        }
    }

    while cur < src_size {
        bw.put_literal(src[cur]);
        cur += 1;
    }

    bw.finalize()
}
