//! Levels 3 and 4: lazy parse with several previous positions per hash.
//!
//! Each hash maps to a bucket of the most recent positions, newest first.
//! Insertion shifts the bucket. The parser walks the bucket to pick the best
//! candidate, then applies the same one-byte lookahead as level 2.

use crate::bitstream::BitWriter;
use crate::common::{hash4, match_better, next_match_better, LOOKUP_SIZE, NO_MATCH_POS};

/// Compress `src` into `dst` with a bucket of `bucket_size` positions per hash.
///
/// `accept_len` stops the bucket walk once a match reaches that length.
pub fn pack(
    src: &[u8],
    dst: &mut [u8],
    workmem: &mut [u32],
    bucket_size: usize,
    accept_len: usize,
) -> usize {
    let src_size = src.len();
    if src_size == 0 {
        return 0;
    }

    dst[0] = src[0];
    if src_size == 1 {
        return 1;
    }

    let last_match_pos = src_size.saturating_sub(4);
    let lookup = &mut workmem[..LOOKUP_SIZE * bucket_size];
    lookup.fill(NO_MATCH_POS);

    let mut bw = BitWriter::new(dst, 1);
    let mut hash_pos = 0usize;
    let mut cur = 1usize;

    while cur <= last_match_pos {
        while hash_pos < cur {
            insert(
                lookup,
                hash4(&src[hash_pos..]) as usize,
                bucket_size,
                hash_pos,
            );
            hash_pos += 1;
        }

        let mut best_pos = NO_MATCH_POS;
        let mut best_len = 0usize;

        let bucket_start = hash4(&src[cur..]) as usize * bucket_size;
        let len_limit = src_size - cur;

        let mut bucket_idx = 0usize;
        let mut pos = lookup[bucket_start];
        while pos != NO_MATCH_POS {
            let p = pos as usize;
            let mut len = 0usize;
            if best_len < len_limit && src[p + best_len] == src[cur + best_len] {
                while len < len_limit && src[p + len] == src[cur + len] {
                    len += 1;
                }
            }

            if match_better(cur as u32, pos, len as u32, best_pos, best_len as u32) {
                best_pos = pos;
                best_len = len;
                if best_len >= accept_len {
                    break;
                }
            }

            bucket_idx += 1;
            if bucket_idx == bucket_size {
                break;
            }
            pos = lookup[bucket_start + bucket_idx];
        }

        if best_len > 3 && best_len < accept_len && cur < last_match_pos {
            insert(
                lookup,
                hash4(&src[hash_pos..]) as usize,
                bucket_size,
                hash_pos,
            );
            hash_pos += 1;

            let next_start = hash4(&src[cur + 1..]) as usize * bucket_size;
            let next_len_limit = src_size - (cur + 1);

            let mut next_idx = 0usize;
            let mut next_pos = lookup[next_start];
            while next_pos != NO_MATCH_POS {
                let np = next_pos as usize;
                let mut next_len = 0usize;
                if best_len - 1 < next_len_limit
                    && src[np + best_len - 1] == src[cur + 1 + best_len - 1]
                {
                    while next_len < next_len_limit && src[np + next_len] == src[cur + 1 + next_len]
                    {
                        next_len += 1;
                    }
                }

                if next_len >= best_len {
                    if next_pos > 0 && src[next_pos as usize - 1] == src[cur] {
                        if match_better(
                            cur as u32,
                            next_pos - 1,
                            next_len as u32 + 1,
                            best_pos,
                            best_len as u32,
                        ) {
                            best_pos = next_pos - 1;
                            best_len = next_len + 1;
                        }
                    } else if next_match_better(
                        cur as u32,
                        next_pos,
                        next_len as u32,
                        best_pos,
                        best_len as u32,
                    ) {
                        best_len = 0;
                        break;
                    }
                }

                next_idx += 1;
                if next_idx == bucket_size {
                    break;
                }
                next_pos = lookup[next_start + next_idx];
            }
        }

        if best_len > 4 || (best_len == 4 && (cur as u32) - best_pos - 1 < 0x3FE00) {
            let offs = (cur as u32) - best_pos - 1;
            bw.put_match(best_len as u32, offs);
            cur += best_len;
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

/// Insert `value` at the front of the bucket for `hash`, shifting the rest.
///
/// Each slot takes the value carried from the slot before it. The oldest
/// position falls off the end.
#[inline]
fn insert(lookup: &mut [u32], hash: usize, bucket_size: usize, value: usize) {
    let base = hash * bucket_size;
    let mut next = value as u32;
    for slot in &mut lookup[base..base + bucket_size] {
        next = core::mem::replace(slot, next);
    }
}
