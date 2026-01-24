# Agent Guidelines - SLIP-39 Calculator

**Scope**: Specific to `crates/slip39`.
**Global Rules**: See [Root AGENTS.md](../../AGENTS.md).

## Project Overview

**SLIP-39 Calculator** is a Rust library and interactive TUI for encoding/decoding SLIP-39 mnemonic words.
- **Frameworks**: `ratatui` (TUI), `clap` (CLI).
- **Core**: `src/lib.rs` (stateless), `const/wordlist.txt` (embedded).

## Architecture Details

### Core Library (`src/lib.rs`)
- **Stateless**: Functions do not hold state.
- **Wordlist**: Loaded lazily via `std::sync::OnceLock`. Verified against official SHA256.

### TUI (`src/tui.rs`)
- **Architecture**: Follows The [Elm Architecture](https://guide.elm-lang.org/architecture/) (Model-View-Update).
    - **Model**: `App` struct. Holds all state (inputs, history, mode).
    - **Update**: `update` function. Pure state transitions based on `Msg`.
    - **View**: `ui` function. Renders `App` state to `Frame`.
    - **Msg**: Enum representing all possible events (Key inputs, Logic events).
- **Layout**: Suggestions (Carousel) -> Grid (Table) -> Input (Paragraph).
- **Paper Mode**: Critical feature. Disables history/cache for security.
- **Navigation**: `←/→` (Suggestions), `↑/↓` (History).

## Specific Development Rules

### Security
This tool handles **recovery phrases**.
- **Paper Mode**: Must never persist data to memory/disk when enabled.
- **Output**: Do not print words to stdout unless explicitly requested by CLI args.

### Testing
- **Integration**: `tests/integration_test.rs` includes a slow test for full wordlist verification.
- **Command**:
  ```bash
  # Standard tests
  cargo test -p slip39-calculator
  
  # Verify official wordlist (Slow)
  cargo test -p slip39-calculator -- --ignored
  ```

## Common Tasks

### Modifying Wordlist
**FORBIDDEN**. The wordlist is fixed by the SLIP-39 standard (commit `1524583`).
