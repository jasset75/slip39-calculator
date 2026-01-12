# SLIP-39 Calculator

A Rust library and interactive TUI for encoding/decoding [SLIP-39](https://github.com/satoshilabs/slips/blob/master/slip-0039.md) mnemonic words to/from their 10-bit binary representation.

## Features

- 🔐 **Complete SLIP-39 Wordlist**: All 1024 words from the official specification
- 🧪 **Thoroughly Tested**: 16 tests including verification against official wordlist
- 📚 **Library + Binary**: Use as a library or standalone CLI
- 🎨 **Interactive TUI**: Beautiful terminal interface with ratatui *(coming soon)*
- 🦀 **Idiomatic Rust**: Modern Rust 2021 edition with comprehensive error handling

## Installation

### Prerequisites

- Rust 1.88.0 or later
- [mise](https://mise.jdx.dev/) (recommended for version management)

### Building from Source

```bash
# Clone the repository
git clone https://github.com/yourusername/slip39-calculator.git
cd slip39-calculator

# Install Rust 1.88.0 with mise
mise install

# Build the project
mise exec -- cargo build --release

# Run tests
mise exec -- cargo test
```

## Usage

### As a Library

Add to your `Cargo.toml`:

```toml
[dependencies]
slip39-calculator = "0.1.0"
```

Example:

```rust
use slip39_calculator::{encode, decode};

// Encode a word to 10-bit binary
let binary = encode("academic").unwrap();
assert_eq!(binary, "0000000000");

// Decode binary to word
let word = decode("0000000001").unwrap();
assert_eq!(word, "acid");

// Round-trip
let word = "zero";
let binary = encode(word).unwrap();
let decoded = decode(&binary).unwrap();
assert_eq!(decoded, word);
```

### As a CLI *(In Development)*

```bash
# Encode a word
slip39-calculator --cli encode academic
# Output: 0000000000

# Decode binary
slip39-calculator --cli decode 0000000000
# Output: academic
```

### Interactive TUI *(Coming Soon)*

```bash
# Launch interactive mode
slip39-calculator

# Features:
# - Incremental search with horizontal carousel
# - Single-word grid display (memory-style)
# - Vertical navigation between saved words
# - Binary visualization with bit indices (512, 256, 128...)
# - Color-coded: cyan for active bits, yellow for grid
```

## SLIP-39 Wordlist

The SLIP-39 wordlist contains exactly **1024 words** (2^10), allowing each word to be encoded in **10 bits**.

- Words are in alphabetical order
- Index = position in list (0-1023)
- Binary = 10-bit representation of index

Example:
- `academic` (index 0) → `0000000000`
- `acid` (index 1) → `0000000001`
- `zero` (index 1023) → `1111111111`

## Testing

```bash
# Run all tests
mise exec -- cargo test

# Run unit tests only
mise exec -- cargo test --lib

# Verify wordlist against official SLIP-39 spec
mise exec -- cargo test -- --ignored
```

## Development

This project uses [mise](https://mise.jdx.dev/) for Rust version management:

```bash
# Install dependencies
mise install

# Run with mise
mise exec -- cargo run

# Format code
mise exec -- cargo fmt

# Lint
mise exec -- cargo clippy
```

## Project Structure

```
slip39-calculator/
├── src/
│   ├── lib.rs          # Core encode/decode library
│   └── main.rs         # CLI/TUI binary
├── const/
│   └── wordlist.txt    # Official SLIP-39 wordlist (commit 1524583)
├── tests/
│   └── integration_test.rs # Integration tests
├── Cargo.toml
├── .mise.toml          # Rust version config
└── README.md
```

## Wordlist Integrity

The wordlist is loaded lazily at runtime from the official SLIP-39 repository source:
- **Source**: [satoshilabs/slips](https://github.com/satoshilabs/slips/blob/master/slip-0039/wordlist.txt)
- **Commit**: `1524583213f1392321109b0ff0a91330836ecb32` (2019-03-02)
- **SHA256**: `bcc4555340332d169718aed8bf31dd9d5248cb7da6e5d355140ef4f1e601eec3`
- **Loading**: Uses `std::sync::OnceLock` for lazy initialization (no build script required)

The wordlist file is embedded into the binary at compile time using `include_str!`, then parsed and validated on first access. The SHA256 checksum is verified at test time to ensure integrity.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

## References

- [SLIP-39 Specification](https://github.com/satoshilabs/slips/blob/master/slip-0039.md)
- [Official Wordlist](https://github.com/satoshilabs/slips/blob/master/slip-0039/wordlist.txt)

## Contributing

Contributions welcome! Please feel free to submit a Pull Request.
