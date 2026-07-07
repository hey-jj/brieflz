//! Decoders for the tag bitstream.
//!
//! [`depack`] trusts its input and skips bounds checks. [`depack_safe`]
//! validates every read and write. Both seed the tag so the first control bit
//! is a synthetic literal zero, which copies the verbatim first byte. Matches
//! copy one byte at a time so overlapping runs expand correctly.

use crate::Error;

/// Decode `depacked_size` bytes from `src` into `dst`, trusting the input.
///
/// Returns the number of bytes written. Returns zero for `depacked_size == 0`.
/// Malformed input may index out of bounds and panic. Use [`depack_safe`] for
/// untrusted data.
pub fn depack(src: &[u8], dst: &mut [u8], depacked_size: usize) -> usize {
    if depacked_size == 0 {
        return 0;
    }

    let mut sp = 0usize;
    let mut dp = 0usize;
    // Seed: one bit left, high bit zero, so the first read is a literal.
    let mut tag: u32 = 0x4000;
    let mut bits_left: i32 = 1;

    while dp < depacked_size {
        if getbit(src, &mut sp, &mut tag, &mut bits_left) != 0 {
            let len = getgamma(src, &mut sp, &mut tag, &mut bits_left) + 2;
            let mut off = getgamma(src, &mut sp, &mut tag, &mut bits_left) - 2;
            off = (off << 8) + u32::from(src[sp]) + 1;
            sp += 1;

            // Copy one byte at a time so overlapping runs replicate.
            let off = off as usize;
            for _ in 0..len {
                dst[dp] = dst[dp - off];
                dp += 1;
            }
        } else {
            dst[dp] = src[sp];
            sp += 1;
            dp += 1;
        }
    }

    dp
}

#[inline]
fn getbit(src: &[u8], sp: &mut usize, tag: &mut u32, bits_left: &mut i32) -> u32 {
    let was = *bits_left;
    *bits_left -= 1;
    if was == 0 {
        *tag = u32::from(src[*sp]) | (u32::from(src[*sp + 1]) << 8);
        *sp += 2;
        *bits_left = 15;
    }
    let bit = u32::from(*tag & 0x8000 != 0);
    *tag <<= 1;
    bit
}

#[inline]
fn getgamma(src: &[u8], sp: &mut usize, tag: &mut u32, bits_left: &mut i32) -> u32 {
    let mut result = 1u32;
    loop {
        result = (result << 1) + getbit(src, sp, tag, bits_left);
        if getbit(src, sp, tag, bits_left) == 0 {
            break;
        }
    }
    result
}

/// Decode `depacked_size` bytes from `src` into `dst` with bounds checking.
///
/// Reads at most `src.len()` bytes and writes at most `depacked_size` bytes.
/// Returns the byte count on success, or [`Error::MalformedInput`] on any
/// truncated, malformed, or out-of-range input. Returns `Ok(0)` for
/// `depacked_size == 0`.
pub fn depack_safe(src: &[u8], dst: &mut [u8], depacked_size: usize) -> Result<usize, Error> {
    if depacked_size > dst.len() {
        return Err(Error::MalformedInput);
    }
    let mut st = SafeState {
        src,
        sp: 0,
        src_avail: src.len(),
        tag: 0,
        bits_left: 1,
    };
    let mut dst_size = 0usize;
    let mut dst_avail = depacked_size;

    while dst_size < depacked_size {
        let bit = st.getbit().ok_or(Error::MalformedInput)?;

        if bit != 0 {
            let len = st
                .getgamma()
                .ok_or(Error::MalformedInput)?
                .checked_add(2)
                .ok_or(Error::MalformedInput)?;
            let mut off = st.getgamma().ok_or(Error::MalformedInput)?.wrapping_sub(2);

            if off >= 0x00FF_FFFF {
                return Err(Error::MalformedInput);
            }
            if st.src_avail == 0 {
                return Err(Error::MalformedInput);
            }
            st.src_avail -= 1;
            off = (off << 8) + u32::from(st.src[st.sp]) + 1;
            st.sp += 1;

            let produced = depacked_size - dst_avail;
            if off as usize > produced {
                return Err(Error::MalformedInput);
            }
            if len as usize > dst_avail {
                return Err(Error::MalformedInput);
            }
            dst_avail -= len as usize;

            // Copy one byte at a time so overlapping runs replicate.
            let off = off as usize;
            let len = len as usize;
            for k in 0..len {
                dst[dst_size + k] = dst[dst_size + k - off];
            }
            dst_size += len;
        } else {
            if st.src_avail == 0 || dst_avail == 0 {
                return Err(Error::MalformedInput);
            }
            st.src_avail -= 1;
            dst_avail -= 1;
            dst[dst_size] = st.src[st.sp];
            st.sp += 1;
            dst_size += 1;
        }
    }

    Ok(dst_size)
}

struct SafeState<'a> {
    src: &'a [u8],
    sp: usize,
    src_avail: usize,
    tag: u32,
    bits_left: i32,
}

impl SafeState<'_> {
    #[inline]
    fn getbit(&mut self) -> Option<u32> {
        let was = self.bits_left;
        self.bits_left -= 1;
        if was == 0 {
            if self.src_avail < 2 {
                return None;
            }
            self.src_avail -= 2;
            self.tag = u32::from(self.src[self.sp]) | (u32::from(self.src[self.sp + 1]) << 8);
            self.sp += 2;
            self.bits_left = 15;
        }
        let bit = u32::from(self.tag & 0x8000 != 0);
        self.tag <<= 1;
        Some(bit)
    }

    #[inline]
    fn getgamma(&mut self) -> Option<u32> {
        let mut v = 1u32;
        loop {
            let bit = self.getbit()?;
            if v & 0x8000_0000 != 0 {
                return None;
            }
            v = (v << 1) + bit;
            if self.getbit()? == 0 {
                break;
            }
        }
        Some(v)
    }
}
