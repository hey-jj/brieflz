//! Levels 8 to 10: forward dynamic programming with binary search trees.
//!
//! Each hash entry roots a binary search tree keyed by suffix order. As the
//! parser searches a tree it re-roots it at the current position, so matches
//! come back in order of increasing distance. The forward DP records the
//! cheapest way to reach each position. A backward gather then a forward emit
//! produce the token stream.
//!
//! The arrays do not overlap, so this needs `5 * src_size` words plus the
//! lookup table.

use crate::bitstream::BitWriter;
use crate::common::{hash4, match_cost, LOOKUP_SIZE, NO_MATCH_POS, WORD_MAX};

/// Compress `src` into `dst` with the binary-tree DP parser.
///
/// `max_depth` caps the tree descent. `accept_len` stops the search once a
/// match reaches that length and lets later positions skip re-matching inside
/// it. Level 10 passes the maximum for both, making the parse optimal.
pub fn pack(
    src: &[u8],
    dst: &mut [u8],
    workmem: &mut [u32],
    max_depth: u64,
    accept_len: u64,
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

    // Word offsets into workmem.
    let cost = 0usize;
    let mpos = cost + src_size + 1;
    let mlen = mpos + src_size + 1;
    let nodes = mlen + src_size + 1;
    let lookup = nodes + 2 * src_size;

    for i in 0..LOOKUP_SIZE {
        workmem[lookup + i] = NO_MATCH_POS;
    }

    // Position 0 is not parsed, so seed its tree node.
    workmem[lookup + hash4(&src[0..]) as usize] = 0;
    workmem[nodes] = NO_MATCH_POS;
    workmem[nodes + 1] = NO_MATCH_POS;

    for i in 0..=src_size {
        workmem[cost + i] = WORD_MAX;
        workmem[mlen + i] = 1;
    }
    workmem[cost] = 0;
    workmem[cost + 1] = 8;

    let mut next_match_cur = 1usize;

    // Phase 1: lowest-cost path arriving at each position.
    for cur in 1..=last_match_pos {
        // Rebase costs to avoid overflow.
        if workmem[cost + cur] > WORD_MAX - 128 {
            let mut min_cost = WORD_MAX;
            for i in cur..=src_size {
                if workmem[cost + i] < min_cost {
                    min_cost = workmem[cost + i];
                }
            }
            for i in cur..=src_size {
                if workmem[cost + i] != WORD_MAX {
                    workmem[cost + i] -= min_cost;
                }
            }
        }

        if workmem[cost + cur + 1] > workmem[cost + cur] + 9 {
            workmem[cost + cur + 1] = workmem[cost + cur] + 9;
            workmem[mlen + cur + 1] = 1;
        }

        if cur > next_match_cur {
            next_match_cur = cur;
        }

        let mut max_len = 3usize;

        let hash = hash4(&src[cur..]) as usize;
        let mut pos = workmem[lookup + hash];
        workmem[lookup + hash] = cur as u32;

        // Indices of the slots that take the next less/greater children.
        let mut lt_node = 2 * cur;
        let mut gt_node = 2 * cur + 1;
        let mut lt_len = 0usize;
        let mut gt_len = 0usize;

        let len_limit = if cur == next_match_cur {
            src_size - cur
        } else if accept_len < (src_size - cur) as u64 {
            accept_len as usize
        } else {
            src_size - cur
        };
        let mut num_chain = max_depth;

        loop {
            if pos == NO_MATCH_POS || num_chain == 0 {
                workmem[nodes + lt_node] = NO_MATCH_POS;
                workmem[nodes + gt_node] = NO_MATCH_POS;
                break;
            }
            num_chain -= 1;
            let p = pos as usize;

            let mut len = lt_len.min(gt_len);
            while len < len_limit && src[p + len] == src[cur + len] {
                len += 1;
            }

            if cur == next_match_cur && len > max_len {
                for i in (max_len + 1)..=len {
                    let mc = match_cost((cur - p - 1) as u32, i as u32);
                    let cost_there = workmem[cost + cur] + mc;
                    if cost_there < workmem[cost + cur + i] {
                        workmem[cost + cur + i] = cost_there;
                        workmem[mpos + cur + i] = (cur - p - 1) as u32;
                        workmem[mlen + cur + i] = i as u32;
                    }
                }
                max_len = len;
                if len as u64 >= accept_len {
                    next_match_cur = cur + len;
                }
            }

            if len as u64 >= accept_len || len == len_limit {
                workmem[nodes + lt_node] = workmem[nodes + 2 * p];
                workmem[nodes + gt_node] = workmem[nodes + 2 * p + 1];
                break;
            }

            if src[p + len] < src[cur + len] {
                workmem[nodes + lt_node] = pos;
                lt_node = 2 * p + 1;
                pos = workmem[nodes + lt_node];
                lt_len = len;
            } else {
                workmem[nodes + gt_node] = pos;
                gt_node = 2 * p;
                pos = workmem[nodes + gt_node];
                gt_len = len;
            }
        }
    }

    for cur in (last_match_pos + 1)..src_size {
        if workmem[cost + cur + 1] > workmem[cost + cur] + 9 {
            workmem[cost + cur + 1] = workmem[cost + cur] + 9;
            workmem[mlen + cur + 1] = 1;
        }
    }

    // Phase 2: gather tokens backwards into a contiguous range.
    let mut next_token = src_size;
    let mut cur = src_size;
    while cur > 1 {
        workmem[mlen + next_token] = workmem[mlen + cur];
        workmem[mpos + next_token] = workmem[mpos + cur];
        cur -= workmem[mlen + cur] as usize;
        next_token -= 1;
    }

    // Phase 3: emit tokens forward.
    let mut cur = 1usize;
    let mut i = next_token + 1;
    while i <= src_size {
        let ml = workmem[mlen + i] as usize;
        if ml == 1 {
            bw.put_literal(src[cur]);
        } else {
            let offs = workmem[mpos + i];
            bw.put_match(ml as u32, offs);
        }
        cur += ml;
        i += 1;
    }

    bw.finalize()
}
