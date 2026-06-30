//! Levels 5 to 7: backwards dynamic programming with left-extension.
//!
//! The parse runs right to left. It builds hash chains, then computes the
//! lowest-cost path from each position to the end. When a match improves the
//! cost it extends to the left so repeated patterns resolve without searching
//! at every position.
//!
//! The scratch arrays overlap exactly as the C code lays them out so the work
//! fits in `3 * src_size` words for large inputs. `cost` shares storage with
//! `prev`, and `lookup` shares storage with `mpos`. The access order keeps
//! every read ahead of the write that would clobber it.

use crate::bitstream::BitWriter;
use crate::common::{hash4_bits, log2, match_cost, HASH_BITS, LOOKUP_SIZE, NO_MATCH_POS};

const LITERAL_COST: u32 = 9;

/// Compress `src` into `dst` with the left-extension DP parser.
///
/// `max_depth` caps the hash-chain walk. `accept_len` stops the walk once a
/// match reaches that length.
pub fn pack(
    src: &[u8],
    dst: &mut [u8],
    workmem: &mut [u32],
    max_depth: u32,
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

    let mut bw = BitWriter::new(dst, 1);

    if src_size < 4 {
        for &b in &src[1..src_size] {
            bw.put_literal(b);
        }
        return bw.finalize();
    }

    let last_match_pos = src_size - 4;

    // Word offsets into workmem. cost aliases prev, lookup aliases mpos.
    let prev = 0usize;
    let mpos = src_size;
    let mlen = mpos + src_size;
    let cost = prev;
    let lookup = mpos;

    // Phase 1: build hash chains.
    let bits = if 2 * src_size < LOOKUP_SIZE {
        HASH_BITS
    } else {
        log2(src_size as u32)
    };

    for i in 0..(1usize << bits) {
        workmem[lookup + i] = NO_MATCH_POS;
    }

    if last_match_pos > 0 {
        for i in 0..=last_match_pos {
            let hash = hash4_bits(&src[i..], bits) as usize;
            workmem[prev + i] = workmem[lookup + hash];
            workmem[lookup + hash] = i as u32;
        }
    }

    // Seed the last three positions as literals.
    workmem[mlen + src_size - 3] = 1;
    workmem[mlen + src_size - 2] = 1;
    workmem[mlen + src_size - 1] = 1;
    workmem[cost + src_size - 3] = 27;
    workmem[cost + src_size - 2] = 18;
    workmem[cost + src_size - 1] = 9;
    workmem[cost + src_size] = 0;

    // Phase 2: lowest-cost path from each position to the end.
    let mut cur = last_match_pos;
    while cur > 0 {
        let mut pos = workmem[prev + cur];

        workmem[cost + cur] = workmem[cost + cur + 1] + LITERAL_COST;
        workmem[mlen + cur] = 1;

        let mut max_len = 3usize;
        let len_limit = src_size - cur;
        let mut num_chain = max_depth;

        while pos != NO_MATCH_POS && num_chain != 0 {
            num_chain -= 1;
            let p = pos as usize;
            let mut len = 0usize;

            if max_len < len_limit && src[p + max_len] == src[cur + max_len] {
                while len < len_limit && src[p + len] == src[cur + len] {
                    len += 1;
                }
            }

            if len > max_len {
                let mut min_cost = u32::MAX;
                let mut min_cost_len = 3usize;
                for i in (max_len + 1)..=len {
                    let mc = match_cost((cur - p - 1) as u32, i as u32);
                    let cost_here = mc + workmem[cost + cur + i];
                    if cost_here < min_cost {
                        min_cost = cost_here;
                        min_cost_len = i;
                    }
                }
                max_len = len;

                if min_cost < workmem[cost + cur] {
                    workmem[cost + cur] = min_cost;
                    workmem[mpos + cur] = pos;
                    workmem[mlen + cur] = min_cost_len as u32;

                    // Left-extend while the byte before each side matches.
                    if p > 0 && src[p - 1] == src[cur - 1] {
                        let mut p_ext = p;
                        let mut len_ext = min_cost_len;
                        loop {
                            cur -= 1;
                            p_ext -= 1;
                            len_ext += 1;
                            let mc = match_cost((cur - p_ext - 1) as u32, len_ext as u32);
                            let cost_here = mc + workmem[cost + cur + len_ext];
                            workmem[cost + cur] = cost_here;
                            workmem[mpos + cur] = p_ext as u32;
                            workmem[mlen + cur] = len_ext as u32;
                            if !(p_ext > 0 && src[p_ext - 1] == src[cur - 1]) {
                                break;
                            }
                        }
                        break;
                    }
                }
            }

            if len >= accept_len || len == len_limit {
                break;
            }
            pos = workmem[prev + p];
        }

        cur -= 1;
    }

    workmem[mpos] = 0;
    workmem[mlen] = 1;

    // Phase 3: emit, following the recorded path.
    let mut i = 1usize;
    while i < src_size {
        let ml = workmem[mlen + i] as usize;
        if ml == 1 {
            bw.put_literal(src[i]);
        } else {
            let offs = i as u32 - workmem[mpos + i] - 1;
            bw.put_match(ml as u32, offs);
        }
        i += ml;
    }

    bw.finalize()
}
