# SLIP-39 Calculator

A Rust library and interactive TUI for encoding/decoding [SLIP-39](https://github.com/satoshilabs/slips/blob/master/slip-0039.md) mnemonic words to/from their 10-bit binary representation.

## Features

- 🔐 **Complete SLIP-39 Wordlist**: All 1024 words from the official specification
- 🧪 **Thoroughly Tested**: 16 tests including verification against official wordlist
- 📚 **Library + Binary**: Use as a library or standalone CLI
- 🎨 **Interactive TUI**: Beautiful terminal interface with ratatui
- 🦀 **Idiomatic Rust**: Modern Rust 2021 edition with comprehensive error handling
- 🛡️ **Security-First**: Designed with ephemeral sessions and memory safety in mind

## Installation

### Prerequisites

- Rust 1.92.0 or later
- [mise](https://mise.jdx.dev/) (recommended for version management)

### Building from Source

```bash
# Clone the repository
git clone https://github.com/yourusername/slip39-calculator.git
cd slip39-calculator

# Install Rust 1.92.0 with mise
mise install

# Build the project
mise exec -- cargo build --release

# Run tests
mise exec -- cargo test
```

## Usage

### Interactive TUI (Default)

Simply run the application without arguments to launch the interactive Terminal User Interface:

```bash
slip39c
```

#### TUI Features

- **Dual Input Modes**:
  - **Word Mode** (Default): Type words to find their 10-bit binary index.
  - **Binary Mode**: Type 0s and 1s to find the corresponding word.
- **Incremental Search**: Filter 1024 words instantly as you type.
- **Carousel Navigation**: Browse suggestion candidates horizontally with `Left`/`Right`.
- **Memory Grid**: Visual representation of the 10-bit binary value (Cyan = 1, Gray = 0).
- **History**: Keep track of up to 20 selected words.
- **Visual Feedback**:
  - **Normal Mode**: Cyan (Blue) theme.
  - **Paper Mode**: Red theme (warning: no history).

#### Startup Select
If you run `slip39c` without arguments, a **Selection Modal** will appear letting you choose between Word and Binary input.

You can also bypass the modal with CLI flags:

```bash
# Launch directly into Word Input Mode
slip39c --mode word

# Launch directly into Binary Input Mode
slip39c --mode binary
```

#### Controls

| Key | Action |
| --- | --- |
| `Type` | Filter suggestions (Word) / Enter bits (Binary) |
| `Enter` | Select current suggestion / Decode binary |
| `←` / `→` | Navigate suggestions / Mode Selection (Startup) |
| `↑` / `↓` | Navigate saved words history |
| `Esc` | Exit application |

### Security Features

The tool is designed to be safe for use in ephemeral environments (e.g., Tails OS, air-gapped machines):

1.  **No Disk Writes**: The application **never** writes to the filesystem. No logs, no cache, no history files.
2.  **RAM Only**: All state (selected words, input) exists only in process memory (`Vec<String>`).
3.  **Ephemeral Session**:
    - Memory is released immediately upon exit (`Esc`).
    - **Paper Mode** (`--paper`): Explicitly clears the internal buffer before adding a new word, ensuring **zero history retention** even in RAM during the session. Useful for transcribing words one by one to physical paper.
4.  **Terminal Hygiene**: Uses Alternate Screen buffers to ensure no sensitive words remain in your terminal's scrollback history after exit.

### CLI Mode (Scripting)

> [!WARNING]
> **Security Notice**: Using the CLI with sensitive arguments (e.g., `slip39c encode-word <secret>`) involves risks:
> 1. **Shell History**: Your shell (bash, zsh, etc.) will likely save the command and arguments to its history file (`~/.zsh_history`, etc.).
> 2. **Process List**: While running, arguments are visible to other users/processes via `ps`.
>
> For sensitive operations, **always use the Interactive TUI** (default mode) or disable your shell history before running CLI commands.

The CLI provides subcommands for single-shot operations:

```bash
# Encode a word to binary
slip39c encode-word academic
# Output: 0000000000

# Decode binary to word
slip39c decode-bits 0000000001
# Output: acid

# Get the index of a word (0-1023)
slip39c word-to-index zero
# Output: 1023

# Get the word at a specific index
slip39c index-to-word 0
# Output: academic

# View help
slip39c --help
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

For detailed development documentation, including project structure, wordlist integrity technical details, and advanced workflows, please refer to [AGENTS.md](AGENTS.md).

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
