//! Encoder side of the tag bitstream.
//!
//! Control bits accumulate into a 16-bit tag. The encoder shifts each bit in
//! at the low end and writes a full tag little-endian into a slot reserved
//! earlier in the output. The decoder reads bits back out at the high end, so
//! a full 16-bit tag preserves emission order.
//!
//! Only the per-bit path is implemented. The batched lookup path in the C
//! source is a speed optimization that produces the same bytes.

/// Writes literals, match tokens, and the tag bitstream into the output slice.
///
/// The writer tracks a cursor for payload bytes and a separate index for the
/// pending tag slot, mirroring the two output pointers of the C encoder.
pub struct BitWriter<'a> {
    out: &'a mut [u8],
    /// Next free byte for payload and new tag slots.
    next_out: usize,
    /// Reserved slot for the tag currently being filled.
    tag_out: usize,
    /// Accumulating tag value.
    tag: u32,
    /// Bits that still fit in the current tag before a flush.
    bits_left: i32,
}

impl<'a> BitWriter<'a> {
    /// Start a writer over `out` with the cursor at `start`.
    ///
    /// `start` is one past the verbatim first byte. The first tag slot is
    /// reserved here, matching the C setup that writes byte zero, then claims
    /// two bytes for the first tag.
    pub fn new(out: &'a mut [u8], start: usize) -> Self {
        let tag_out = start;
        BitWriter {
            out,
            next_out: start + 2,
            tag_out,
            tag: 0,
            bits_left: 16,
        }
    }

    /// Append one payload byte at the cursor.
    #[inline]
    pub fn put_byte(&mut self, b: u8) {
        self.out[self.next_out] = b;
        self.next_out += 1;
    }

    /// Push one control bit into the tag, flushing a full tag first.
    #[inline]
    pub fn put_bit(&mut self, bit: u32) {
        // Flush when the tag is full. bits_left is post-decremented, so the
        // flush fires when it was zero.
        let was = self.bits_left;
        self.bits_left -= 1;
        if was == 0 {
            self.out[self.tag_out] = (self.tag & 0x00FF) as u8;
            self.out[self.tag_out + 1] = ((self.tag >> 8) & 0x00FF) as u8;
            self.tag_out = self.next_out;
            self.next_out += 2;
            self.bits_left = 15;
        }
        self.tag = (self.tag << 1) + bit;
    }

    /// Encode `val` with the gamma2 universal code. Requires `val >= 2`.
    ///
    /// Emits the bit below the leading one, then for each lower bit a one
    /// continuation bit followed by that value bit, then a terminating zero.
    pub fn put_gamma(&mut self, val: u32) {
        debug_assert!(val >= 2);
        // Mask for the second-highest set bit of val.
        let mut mask = 1u32 << (log2_second(val));
        self.put_bit(u32::from(val & mask != 0));
        while {
            mask >>= 1;
            mask != 0
        } {
            self.put_bit(1);
            self.put_bit(u32::from(val & mask != 0));
        }
        self.put_bit(0);
    }

    /// Emit a match token: tag bit, length gamma, offset-high gamma, low byte.
    #[inline]
    pub fn put_match(&mut self, len: u32, offs: u32) {
        self.put_bit(1);
        self.put_gamma(len - 2);
        self.put_gamma((offs >> 8) + 2);
        self.put_byte((offs & 0x00FF) as u8);
    }

    /// Emit a literal token: a zero tag bit and the raw byte.
    #[inline]
    pub fn put_literal(&mut self, b: u8) {
        self.put_bit(0);
        self.put_byte(b);
    }

    /// Write the trailing bit and flush the final partial tag.
    ///
    /// Returns the cursor, which is the compressed byte count.
    pub fn finalize(&mut self) -> usize {
        self.put_bit(1);
        self.tag <<= self.bits_left;
        self.out[self.tag_out] = (self.tag & 0x00FF) as u8;
        self.out[self.tag_out + 1] = ((self.tag >> 8) & 0x00FF) as u8;
        self.next_out
    }
}

/// Bit index of the second-highest set bit of `val`. Requires `val >= 2`.
#[inline]
fn log2_second(val: u32) -> u32 {
    debug_assert!(val >= 2);
    30 - val.leading_zeros()
}
