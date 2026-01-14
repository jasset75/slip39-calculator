# Agent Guidelines - Crypto Toolbox

This is a **Rust Workspace** containing multiple cryptographic tools.
This document defines **GLOBAL** rules. For specific tool architecture, check the `AGENTS.md` inside each crate directory.

## Workspace Overview
- **Path**: `/Users/juan/work/crypto-toolbox`
- **Structure**:
  - `crates/`: Source code for individual tools.
  - `Cargo.toml`: Workspace definition.

## Technology Stack (Global)
- **Language**: Rust (Latest Stable defined in `.mise.toml`)
- **Version Management**: `mise`
- **Linter/Formatter**: `clippy` / `rustfmt`
- **Git Hooks**: `lefthook`

## Global Development Workflow

### 1. Build & Test
Commands run from the workspace root apply to **all** crates.
```bash
# Test entire workspace
cargo test --workspace

# Check for warnings
cargo clippy --workspace -- -D warnings

# Build all release binaries
cargo build --release --workspace
```

### 2. Code Style
- **English only**: Comments, variable names, commits.
- **Fail on warnings**: CI checks will fail if `clippy` reports warnings.
- **Formatting**: Always run `cargo fmt` before committing.
- **Testing**: Every public function must have unit tests.

### 3. Commit Messages
Use [Conventional Commits](https://www.conventionalcommits.org/):
- `feat(slip39): add new word` -> Scope is the crate name.
- `fix(common): repair hex parser`
- `chore: update dependencies`

## Navigation
- **SLIP-39**: `crates/slip39/AGENTS.md`
