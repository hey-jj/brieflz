# brieflz

In-memory LZ77/LZSS codec with a multi-level optimal-parse encoder.

The codec encodes literals and `(length, offset)` matches into a
bitstream. Control bits come from a separate 16-bit tag stream. Match
lengths and offset high bits use a universal exp-Golomb code. There is no
container, header, or window-size limit. The caller stores the decompressed
size and passes it to the decoder.

Ten compression levels trade speed for ratio. Level 1 is fastest. Level 10
is optimal and slowest. Every level shares one wire format, so any level's
output decodes with either decoder.

## Installation

```toml
[dependencies]
brieflz = "0.2"
```

## Buffers

Every function takes caller-allocated buffers. Nothing allocates. Size the
output with `max_packed_size` and the scratch buffer with `workmem_size` or
`workmem_size_level`.

## Safety

`depack` trusts its input and skips bounds checks for speed. `depack_safe`
validates every read and write and returns an error on malformed input.

## no_std

The codec path needs no allocator. Build with `default-features = false` for
`no_std` targets, including `wasm32-unknown-unknown`.

## License

Zlib. See [LICENSE](LICENSE).
