//! Shared test data and helpers.
//!
//! The byte arrays match the data the codec's own test suite exercises:
//! an incompressible increasing run, an all-zero run, a mixed run, and a set
//! of malformed compressed streams. The golden module holds packed bytes
//! captured from a known-good encoder for byte-exact checks.

#![allow(dead_code)]

pub mod golden;

/// Incompressible bytes 1 through 248. Exercises the literal-only path.
pub const NUMBERS: &[u8] = &[
    1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26,
    27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50,
    51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74,
    75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 91, 92, 93, 94, 95, 96, 97, 98,
    99, 100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115, 116, 117,
    118, 119, 120, 121, 122, 123, 124, 125, 126, 127, 128, 129, 130, 131, 132, 133, 134, 135, 136,
    137, 138, 139, 140, 141, 142, 143, 144, 145, 146, 147, 148, 149, 150, 151, 152, 153, 154, 155,
    156, 157, 158, 159, 160, 161, 162, 163, 164, 165, 166, 167, 168, 169, 170, 171, 172, 173, 174,
    175, 176, 177, 178, 179, 180, 181, 182, 183, 184, 185, 186, 187, 188, 189, 190, 191, 192, 193,
    194, 195, 196, 197, 198, 199, 200, 201, 202, 203, 204, 205, 206, 207, 208, 209, 210, 211, 212,
    213, 214, 215, 216, 217, 218, 219, 220, 221, 222, 223, 224, 225, 226, 227, 228, 229, 230, 231,
    232, 233, 234, 235, 236, 237, 238, 239, 240, 241, 242, 243, 244, 245, 246, 247, 248,
];

/// Eighty zero bytes. Maximally compressible with offset-one runs.
pub const ZEROES: &[u8] = &[0u8; 80];

/// Mixed zero and 0xFF runs. Short offsets and run boundaries.
pub const ALTERNATE: &[u8] = &[
    0x00, 0xFF, 0x00, 0xFF, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0xFF, 0x00, 0x00, 0x00, 0xFF,
    0x00, 0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0x00, 0xFF, 0x00, 0xFF,
    0xFF, 0xFF, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0xFF, 0xFF, 0x00, 0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0x00, 0xFF, 0x00, 0xFF, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
];

/// A malformed compressed stream and the decoded size it claims.
pub struct ErrorCase {
    /// Bytes available to the decoder.
    pub src_size: usize,
    /// Decoded size the caller passes in.
    pub depacked_size: usize,
    /// The compressed bytes.
    pub data: &'static [u8],
}

/// Malformed streams. Each must make the safe decoder report an error.
pub const ERRORS: &[ErrorCase] = &[
    ErrorCase {
        src_size: 0,
        depacked_size: 1,
        data: &[0x42],
    },
    ErrorCase {
        src_size: 1,
        depacked_size: 2,
        data: &[0x42],
    },
    ErrorCase {
        src_size: 2,
        depacked_size: 2,
        data: &[0x42, 0x00],
    },
    ErrorCase {
        src_size: 3,
        depacked_size: 2,
        data: &[0x42, 0x00, 0x00],
    },
    ErrorCase {
        src_size: 3,
        depacked_size: 5,
        data: &[0x42, 0x00, 0x80],
    },
    ErrorCase {
        src_size: 4,
        depacked_size: 5,
        data: &[0x42, 0x55, 0x45, 0x42],
    },
    ErrorCase {
        src_size: 3,
        depacked_size: 5,
        data: &[0x42, 0xAA, 0x8A],
    },
    ErrorCase {
        src_size: 4,
        depacked_size: 5,
        data: &[0x42, 0x00, 0x80, 0x01],
    },
    ErrorCase {
        src_size: 12,
        depacked_size: 3,
        data: &[
            0x42, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0x00, 0x00, 0x00,
        ],
    },
    ErrorCase {
        src_size: 10,
        depacked_size: 5,
        data: &[0x42, 0xAA, 0x8A, 0xAA, 0xAA, 0xAA, 0xAA, 0x00, 0xC0, 0xFF],
    },
    ErrorCase {
        src_size: 4,
        depacked_size: 4,
        data: &[0x42, 0x00, 0x80, 0x00],
    },
];

/// Deterministic byte source for the round-trip and fuzz tests.
///
/// A small linear congruential generator. A fixed seed yields the same bytes
/// every run, so the random-shaped tests stay reproducible.
pub struct Lcg(u32);

impl Lcg {
    /// Start the generator from `seed`.
    pub fn new(seed: u32) -> Self {
        Lcg(seed)
    }

    /// Next byte in the sequence.
    pub fn next_byte(&mut self) -> u8 {
        self.0 = self.0.wrapping_mul(1_103_515_245).wrapping_add(12_345);
        (self.0 >> 16) as u8
    }

    /// Fill `buf` with the next bytes.
    pub fn fill(&mut self, buf: &mut [u8]) {
        for b in buf.iter_mut() {
            *b = self.next_byte();
        }
    }
}

/// Compress `src` at `level` into a fresh buffer and return the packed bytes.
pub fn pack_to_vec(src: &[u8], level: u8) -> Vec<u8> {
    let mut dst = vec![0u8; brieflz::max_packed_size(src.len())];
    let words = brieflz::workmem_size_level(src.len(), level).unwrap() / 4;
    let mut work = vec![0u32; words.max(1)];
    let n = brieflz::pack_level(src, &mut dst, &mut work, level).unwrap();
    dst.truncate(n);
    dst
}
