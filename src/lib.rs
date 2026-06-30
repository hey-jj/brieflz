//! In-memory LZ77/LZSS codec with a multi-level optimal-parse encoder.
//!
//! The codec encodes a stream of literals and `(length, offset)` matches.
//! Control bits come from a separate 16-bit tag bitstream read MSB-first.
//! Match lengths and offset high bits use a universal code (order-1
//! exp-Golomb). The offset low byte is stored raw. There is no window-size
//! limit and no container or framing. The caller stores the decompressed
//! size out of band and passes it to the decoder.
//!
//! # Buffers
//!
//! Every function takes caller-allocated buffers. Nothing here allocates.
//! Size `dst` with [`max_packed_size`] and `workmem` with [`workmem_size`]
//! or [`workmem_size_level`]. The `workmem` slice holds `u32` words, so divide
//! the byte size by four for its length.
//!
//! # Levels
//!
//! Compression has ten levels. Level 1 is fastest with the worst ratio.
//! Level 10 is optimal and slowest. The wire format is identical across
//! levels, so any level's output decodes with either decoder.
//!
//! # Safety
//!
//! [`depack`] trusts its input and skips bounds checks for speed. Feed it
//! only data you produced and the exact decompressed size. [`depack_safe`]
//! validates every read and write and returns [`Error::MalformedInput`] on
//! bad input.
//!
//! # Example
//!
//! ```
//! let data = b"abracadabra abracadabra";
//! let mut packed = vec![0u8; brieflz::max_packed_size(data.len())];
//! let mut work = vec![0u32; brieflz::workmem_size() / 4];
//! let n = brieflz::pack(data, &mut packed, &mut work);
//!
//! let mut out = vec![0u8; data.len()];
//! let got = brieflz::depack_safe(&packed[..n], &mut out, data.len()).unwrap();
//! assert_eq!(&out[..got], data);
//! ```
#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod bitstream;
mod btparse;
mod common;
mod decode;
mod hashbucket;
mod lazy;
mod leparse;
mod level1;
mod sizes;

pub use sizes::{workmem_size, workmem_size_level};

/// Largest source size the encoder accepts.
///
/// Positions are tracked as `u32`, so the source must fit in a `u32` with
/// room for the all-ones sentinel. Sizes at or above this bound are rejected.
pub const WORD_MAX: usize = u32::MAX as usize;

/// Errors returned by the fallible entry points.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    /// The compression level was outside the range `1..=10`.
    InvalidLevel,
    /// The decoder hit truncated, malformed, or out-of-range input.
    MalformedInput,
}

#[cfg(feature = "std")]
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidLevel => f.write_str("compression level must be 1 to 10"),
            Error::MalformedInput => f.write_str("malformed or truncated compressed input"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

/// Upper bound on the compressed size of `src_size` bytes.
///
/// Use this to size the `dst` buffer before calling [`pack`] or
/// [`pack_level`]. The bound is `src_size + src_size / 8 + 64`.
#[must_use]
pub fn max_packed_size(src_size: usize) -> usize {
    src_size + src_size / 8 + 64
}

/// Compress `src` into `dst` at level 1 and return the byte count written.
///
/// `workmem` must hold at least [`workmem_size`] bytes worth of `u32` words.
/// Returns `0` for empty input and writes nothing in that case. Input of one
/// byte writes that byte and returns `1`.
///
/// # Panics
///
/// Panics if `src.len()` is at or above [`WORD_MAX`], or if `dst` or
/// `workmem` is too small.
#[must_use]
pub fn pack(src: &[u8], dst: &mut [u8], workmem: &mut [u32]) -> usize {
    assert!(src.len() < WORD_MAX, "src_size must be below WORD_MAX");
    level1::pack(src, dst, workmem)
}

/// Compress `src` into `dst` at `level` (1 to 10).
///
/// `workmem` must hold at least [`workmem_size_level`] bytes worth of `u32`
/// words for the same `level`. Returns the byte count written, or `0` for
/// empty input. Returns [`Error::InvalidLevel`] for a level outside `1..=10`.
///
/// # Panics
///
/// Panics if `src.len()` is at or above [`WORD_MAX`], or if `dst` or
/// `workmem` is too small.
pub fn pack_level(
    src: &[u8],
    dst: &mut [u8],
    workmem: &mut [u32],
    level: u8,
) -> Result<usize, Error> {
    assert!(src.len() < WORD_MAX, "src_size must be below WORD_MAX");
    let n = match level {
        1 => level1::pack(src, dst, workmem),
        2 => lazy::pack(src, dst, workmem),
        3 => hashbucket::pack(src, dst, workmem, 2, 16),
        4 => hashbucket::pack(src, dst, workmem, 4, 16),
        5 => leparse::pack(src, dst, workmem, 1, 16),
        6 => leparse::pack(src, dst, workmem, 8, 32),
        7 => leparse::pack(src, dst, workmem, 64, 64),
        8 => btparse::pack(src, dst, workmem, 16, 96),
        9 => btparse::pack(src, dst, workmem, 32, 224),
        10 => btparse::pack(src, dst, workmem, u64::MAX, u64::MAX),
        _ => return Err(Error::InvalidLevel),
    };
    Ok(n)
}

/// Decompress `src` into `dst`, trusting the input.
///
/// Decodes until it has produced `depacked_size` bytes. Skips all bounds
/// checks for speed. The caller must supply valid compressed data and the
/// exact decompressed size. Returns the byte count written, which equals
/// `depacked_size` on success. Returns `0` for `depacked_size == 0`.
///
/// # Panics
///
/// Reads past `src` or writes past `dst` on malformed input, which may
/// panic. Use [`depack_safe`] for untrusted data.
#[must_use]
pub fn depack(src: &[u8], dst: &mut [u8], depacked_size: usize) -> usize {
    decode::depack(src, dst, depacked_size)
}

/// Decompress `src` into `dst` with full bounds checking.
///
/// Reads at most `src.len()` bytes and writes at most `depacked_size` bytes.
/// Returns the byte count written, which equals `depacked_size` on success.
/// Returns [`Error::MalformedInput`] on any truncated, malformed, or
/// out-of-range input, or when `dst` is shorter than `depacked_size`. Returns
/// `Ok(0)` for `depacked_size == 0`.
pub fn depack_safe(src: &[u8], dst: &mut [u8], depacked_size: usize) -> Result<usize, Error> {
    decode::depack_safe(src, dst, depacked_size)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn max_packed_size_formula() {
        assert_eq!(max_packed_size(0), 64);
        assert_eq!(max_packed_size(8), 8 + 1 + 64);
        assert_eq!(max_packed_size(1024), 1024 + 128 + 64);
    }
}
