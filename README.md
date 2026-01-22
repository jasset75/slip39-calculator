# Crypto Toolbox

A collection of Rust-based cryptocurrency tools and utilities.

## Tools

| Tool | Description |
|------|-------------|
| [**slip39**](./crates/slip39) | SLIP-39 wordlist encoder/decoder with interactive TUI. |

## Development

This is a Rust Workspace managed with `mise` for toolchain consistency.

### Prerequisites

- [Rust](https://www.rust-lang.org/) (managed via mise)
- [mise](https://mise.jdx.dev/) (version manager)

### Getting Started

1.  **Clone the repository:**
    ```bash
    git clone https://github.com/jasset75/crypto-toolbox
    cd crypto-toolbox
    ```

2.  **Install dependencies (Rust toolchain):**
    ```bash
    mise install
    ```

3.  **Install pre-commit hooks:**
    ```bash
    mise run install-hooks
    ```

### Build

To build all tools in the workspace:
```bash
cargo build --release
```

### Test

To run tests for all tools:
```bash
cargo test --workspace
```

## License

This project is licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
