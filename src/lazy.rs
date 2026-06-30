//! Level 2: lazy parse with one-byte lookahead.
//!
//! Same single-slot hashing as level 1. After finding a match, it checks the
//! next position for a better match and may emit a literal instead, or extend
//! the next match one byte to the left.

use crate::bitstream::BitWriter;
use crate::common::{hash4, match_better, next_match_better, NO_MATCH_POS};

/// Compress `src` into `dst` with lazy matching. Returns the byte count.
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

        let mut pos = lookup[hash4(&src[cur..]) as usize];
        let mut len = 0usize;

        if pos != NO_MATCH_POS {
            let p = pos as usize;
            let len_limit = src_size - cur;
            while len < len_limit && src[p + len] == src[cur + len] {
                len += 1;
            }
        }

        if len > 3 && cur < last_match_pos {
            lookup[hash4(&src[hash_pos..]) as usize] = hash_pos as u32;
            hash_pos += 1;

            let next_pos = lookup[hash4(&src[cur + 1..]) as usize];
            let mut next_len = 0usize;

            if next_pos != NO_MATCH_POS && next_pos != pos + 1 {
                let np = next_pos as usize;
                let next_len_limit = src_size - (cur + 1);
                if len - 1 < next_len_limit && src[np + len - 1] == src[cur + 1 + len - 1] {
                    while next_len < next_len_limit && src[np + next_len] == src[cur + 1 + next_len]
                    {
                        next_len += 1;
                    }
                }
            }

            if next_len >= len {
                if next_pos > 0 && src[next_pos as usize - 1] == src[cur] {
                    if match_better(
                        cur as u32,
                        next_pos - 1,
                        next_len as u32 + 1,
                        pos,
                        len as u32,
                    ) {
                        pos = next_pos - 1;
                        len = next_len + 1;
                    }
                } else if next_match_better(cur as u32, next_pos, next_len as u32, pos, len as u32)
                {
                    len = 0;
                }
            }
        }

        if len > 4 || (len == 4 && (cur as u32) - pos - 1 < 0x3FE00) {
            let offs = (cur as u32) - pos - 1;
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
