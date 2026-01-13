# Guidelines for AI Agents

This document provides context and guidelines for AI coding assistants working on this project.

## Project Overview

**SLIP-39 Calculator** is a Rust library and interactive TUI for encoding/decoding SLIP-39 mnemonic words to/from 10-bit binary representation.

- **Language**: Rust 2021 edition
- **Minimum Rust Version**: 1.92.0 (managed via mise)
- **Architecture**: Library (`lib.rs`) + Binary (`main.rs`)
- **UI Framework**: ratatui for TUI, clap for CLI
- **Testing**: Unit tests, integration tests, doc tests

## Code Style

### Rust Idioms

- **Error Handling**: Use `thiserror` for error types, return `Result` types
- **Naming**: snake_case for functions/variables, PascalCase for types
- **Documentation**: Doc comments (`///`) for public APIs with examples
- **Tests**: Unit tests in same file, integration tests in `tests/`

### Project Conventions

- **All interfaces in English**: Code, comments, documentation, error messages
- **Comprehensive testing**: Every public function should have tests
- **Security considerations**: This handles cryptographic material (mnemonic words)
  - TUI uses alternate screen buffer (no scrollback)
  - Paper mode available for maximum security
  - TUI uses alternate screen buffer (no scrollback)
  - Paper mode available for maximum security
  - No logging of sensitive data

## Project Structure

```
slip39-calculator/
├── src/
│   ├── lib.rs          # Core encode/decode library
│   └── main.rs         # CLI/TUI binary (slip39c)
├── const/
│   └── wordlist.txt    # Official SLIP-39 wordlist (commit 1524583)
├── tests/
│   └── integration_test.rs # Integration tests
├── Cargo.toml          # Package manifest
├── .mise.toml          # Rust version config (1.92)
├── .envrc.example      # direnv configuration template
├── lefthook.yml        # Git hooks configuration
└── README.md
```

## Wordlist Integrity

The wordlist is loaded lazily at runtime from the official SLIP-39 repository source:
- **Source**: [satoshilabs/slips](https://github.com/satoshilabs/slips/blob/master/slip-0039/wordlist.txt)
- **Commit**: `1524583213f1392321109b0ff0a91330836ecb32` (2019-03-02)
- **SHA256**: `bcc4555340332d169718aed8bf31dd9d5248cb7da6e5d355140ef4f1e601eec3`
- **Loading**: Uses `std::sync::OnceLock` for lazy initialization.

The wordlist file is embedded into the binary at compile time using `include_str!`, then parsed and validated on first access. The SHA256 checksum is verified at test time to ensure integrity.

## Architecture

### Core Library (`src/lib.rs`)

**Public API:**
```rust
pub fn wordlist() -> &'static [&'static str];  // 1024 SLIP-39 words
pub fn encode(word: &str) -> Result<String, Error>;
pub fn decode(binary: &str) -> Result<String, Error>;
pub enum Error { ... }
```

**Design principles:**
- Stateless functions
- No external dependencies for core logic
- Compile-time wordlist embedding via `include_str!` + `OnceLock`

### TUI Implementation (In Progress)

**UI Design:**
- **3-section layout**: Suggestions carousel (top), word grid (center), input (bottom)
- **Navigation**: 
  - Horizontal (←→): Navigate search suggestions
  - Vertical (↑↓): Navigate saved words (1-20)
- **Color scheme**:
  - Active bits (1): Cyan
  - Grid/headers: Yellow
  - UI text: White
- **Grid style**: "Memory RAM" style with bit indices (512, 256, 128...)

**Key features:**
- Single-word display (one at a time)
- Incremental search with filtering
- Shows 20 words max (SLIP-39 recovery phrase length)
- Security: alternate screen, no scrollback

### CLI Fallback

```bash
```bash
slip39c encode-word <WORD>
slip39c decode-bits <BINARY>
```

For scripting and non-interactive use.

## Testing Strategy

### Unit Tests (`cargo test --lib`)
- Test all public functions
- Edge cases (first word, last word, invalid input)
- Error handling paths

### Integration Tests (`tests/integration_test.rs`)
- Wordlist integrity (length, order, boundaries)
- Round-trip encoding/decoding
- **Wordlist verification**: Compare against official SLIP-39 spec (ignored by default)
  - Run with: `cargo test -- --ignored`

### Doc Tests
- Examples in documentation must compile and pass

## Development Workflow

### Making Changes

1. **Branch naming**: `feat/`, `fix/`, `refactor/`, `docs/`
2. **Commits**: Conventional commits format
   - `feat:` new features
   - `fix:` bug fixes
   - `docs:` documentation
   - `test:` tests
   - `refactor:` code refactoring
3. **Testing**: Run `cargo test` before committing
4. **Formatting**: Use `cargo fmt`
5. **Linting**: Use `cargo clippy`

### Development Environment

This project uses [mise](https://mise.jdx.dev/) for Rust version management and [lefthook](https://github.com/evilmartians/lefthook) for git hooks.

#### Setup

1.  **Install dependencies**:
    ```bash
    mise install
    ```
2.  **Configure direnv** (Recommended):
    ```bash
    cp .envrc.example .envrc
    direnv allow
    ```
    This ensures `cargo` always uses the correct Rust version.

### Common Development Tasks

```bash
# Format code
cargo fmt

# Lint
cargo clippy

# Run all tests
cargo test

# Run wordlist verification (slow/ignored by default)
cargo test -- --ignored
```

### Incremental Development

Follow the task breakdown in `task.md`:
1. ✅ Core library (complete)
2. ✅ TUI implementation (complete)
3. ✅ CLI fallback (complete)
4. 🚧 Documentation (in progress)
5. ⏳ Verification

## Security Considerations

This project handles cryptographic recovery phrases. When implementing features:

- **Never log sensitive data**: No printing of word lists or binary sequences to stdout in production
- **Clear screen on exit**: TUI must use alternate screen buffer
- **Paper mode**: Provide non-accumulative mode for air-gapped use
- **No network calls**: Except for tests (marked `#[ignore]`)

## Common Tasks

### Adding a new word encoding function

```rust
/// Brief description
///
/// # Arguments
/// * `param` - Description
///
/// # Returns
/// * `Ok(...)` - Success case
/// * `Err(Error::...)` - Error case
///
/// # Example
/// ```
/// use slip39_calculator::function_name;
/// assert_eq!(function_name(...), ...);
/// ```
pub fn function_name(...) -> Result<...> { ... }

#[cfg(test)]
mod tests {
    #[test]
    fn test_function_name() { ... }
}
```

### Adding TUI widget

Use ratatui patterns:
- Create widget struct in separate module
- Implement `Widget` trait
- Use `Block`, `Paragraph`, `Table` from ratatui
- Handle events in main app loop

### Modifying wordlist

**DON'T**. The wordlist is from the SLIP-39 specification and must not be modified. Any changes would break compatibility with the standard.

If you suspect an issue, run the verification test:
```bash
cargo test -- --ignored test_wordlist_matches_official_slip39
```

## Questions?

- Check `implementation_plan.md` for architecture decisions
- Review `walkthrough.md` for what's been implemented
- Run tests to understand behavior: `cargo test`
