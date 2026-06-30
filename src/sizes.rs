//! Scratch-buffer sizing per compression level.
//!
//! Each function returns a byte count. Divide by four to get the `u32` word
//! count for the `workmem` slice.

use crate::common::LOOKUP_SIZE;

const WORD: usize = 4;

/// Scratch bytes for level 1. Independent of `src_size`.
#[must_use]
pub fn workmem_size(_src_size: usize) -> usize {
    LOOKUP_SIZE * WORD
}

/// Scratch bytes for `level`. `None` for a level outside `1..=10`.
#[must_use]
pub fn workmem_size_level(src_size: usize, level: i32) -> Option<usize> {
    let words = match level {
        1 | 2 => LOOKUP_SIZE,
        3 => LOOKUP_SIZE * 2,
        4 => LOOKUP_SIZE * 4,
        5..=7 => {
            if LOOKUP_SIZE < 2 * src_size {
                3 * src_size
            } else {
                src_size + LOOKUP_SIZE
            }
        }
        8..=10 => 5 * src_size + 3 + LOOKUP_SIZE,
        _ => return None,
    };
    Some(words * WORD)
}
