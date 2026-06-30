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
//! or [`workmem_size_level`].
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
#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

/// Major version number.
pub const VER_MAJOR: u32 = 1;
/// Minor version number.
pub const VER_MINOR: u32 = 3;
/// Patch version number.
pub const VER_PATCH: u32 = 0;
/// Version number as a string.
pub const VER_STRING: &str = "1.3.0";

/// Largest source size the encoder accepts.
///
/// Positions are tracked as `u32`, so the source must fit in a `u32` with
/// room for the all-ones sentinel. Sizes at or above this bound are rejected.
pub const WORD_MAX: usize = u32::MAX as usize;

/// Errors returned by the fallible entry points.
///
/// The C interface signals failure with a single all-ones sentinel
/// (`(unsigned long) -1`). This enum splits that sentinel into named cases
/// and maps back to it through [`Error::as_sentinel`] for a future C shim.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    /// The compression level was outside the range `1..=10`.
    InvalidLevel,
    /// The decoder hit truncated, malformed, or out-of-range input.
    MalformedInput,
}

impl Error {
    /// The all-ones sentinel a C caller expects for this error.
    ///
    /// Both error cases collapse to the same value, matching the single
    /// `BLZ_ERROR` sentinel of the C interface.
    #[must_use]
    pub const fn as_sentinel(self) -> usize {
        usize::MAX
    }
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

/// Scratch size in bytes for [`pack`] (level 1).
///
/// The result is a count of `u32` words times four. Allocate the `workmem`
/// slice as `[u32; result / 4]`.
#[must_use]
pub fn workmem_size(_src_size: usize) -> usize {
    let _ = _src_size;
    unimplemented!("workmem_size: implemented with sizes module")
}

/// Scratch size in bytes for [`pack_level`] at `level`.
///
/// Returns `None` for a level outside `1..=10`, mirroring the C sentinel
/// `(size_t) -1`. The byte count is a `u32`-word count times four.
#[must_use]
pub fn workmem_size_level(_src_size: usize, _level: i32) -> Option<usize> {
    unimplemented!("workmem_size_level: implemented with sizes module")
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
pub fn pack(_src: &[u8], _dst: &mut [u8], _workmem: &mut [u32]) -> usize {
    unimplemented!("pack: implemented with encode module")
}

/// Compress `src` into `dst` at `level` (1 to 10).
///
/// `workmem` must hold at least [`workmem_size_level`] bytes worth of `u32`
/// words for the same `level`. Returns the byte count written, or `0` for
/// empty input. Returns [`Error::InvalidLevel`] for a level outside
/// `1..=10`.
///
/// # Panics
///
/// Panics if `src.len()` is at or above [`WORD_MAX`], or if `dst` or
/// `workmem` is too small.
pub fn pack_level(
    _src: &[u8],
    _dst: &mut [u8],
    _workmem: &mut [u32],
    _level: i32,
) -> Result<usize, Error> {
    unimplemented!("pack_level: implemented with pack_level dispatch module")
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
pub fn depack(_src: &[u8], _dst: &mut [u8], _depacked_size: usize) -> usize {
    unimplemented!("depack: implemented with decode module")
}

/// Decompress `src` into `dst` with full bounds checking.
///
/// Reads at most `src.len()` bytes and writes at most `depacked_size` bytes.
/// Returns the byte count written, which equals `depacked_size` on success.
/// Returns [`Error::MalformedInput`] on any truncated, malformed, or
/// out-of-range input. Returns `Ok(0)` for `depacked_size == 0`.
pub fn depack_safe(_src: &[u8], _dst: &mut [u8], _depacked_size: usize) -> Result<usize, Error> {
    unimplemented!("depack_safe: implemented with decode_safe module")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_constants_match() {
        assert_eq!(VER_MAJOR, 1);
        assert_eq!(VER_MINOR, 3);
        assert_eq!(VER_PATCH, 0);
        assert_eq!(VER_STRING, "1.3.0");
    }

    #[test]
    fn max_packed_size_formula() {
        assert_eq!(max_packed_size(0), 64);
        assert_eq!(max_packed_size(8), 8 + 1 + 64);
        assert_eq!(max_packed_size(1024), 1024 + 128 + 64);
    }

    #[test]
    fn error_sentinel_is_all_ones() {
        assert_eq!(Error::InvalidLevel.as_sentinel(), usize::MAX);
        assert_eq!(Error::MalformedInput.as_sentinel(), usize::MAX);
    }
}
