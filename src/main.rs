mod tui;

use clap::{Parser, Subcommand, ValueEnum};
use slip39_calculator::{decode, encode, wordlist};
use std::process;

/// SLIP-39 wordlist encoder/decoder
///
/// A tool for encoding and decoding SLIP-39 mnemonic words.
/// Runs in interactive TUI mode by default.
/// Use subcommands for CLI scripting.
#[derive(Parser)]
#[command(name = "slip39c")]
#[command(author, version)]
#[command(about = "SLIP-39 Wordlist Calculator")]
#[command(
    long_about = "SLIP-39 Wordlist Calculator\n\nBy default, this tool launches an interactive TUI (Terminal User Interface) for exploring the wordlist and calculating 10-bit binary representations.\n\nRun 'slip39c --help' for CLI commands."
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Launch interactive TUI (default)
    #[command(name = "tui")]
    Tui {
        /// Enable paper mode (no history)
        #[arg(long, short)]
        paper: bool,

        /// Select input mode (word or binary)
        #[arg(long, value_enum)]
        mode: Option<InputModeArg>,
    },

    /// Encode a SLIP-39 word to its 10-bit binary representation
    #[command(name = "encode-word")]
    EncodeWord {
        /// The SLIP-39 word to encode
        word: String,

        /// Allow fuzzy matching by unique prefix
        #[arg(long, short)]
        prefix: bool,
    },

    /// Decode a 10-bit binary string to its SLIP-39 word
    #[command(name = "decode-bits")]
    DecodeBits {
        /// The 10-bit binary string to decode (e.g., "0000000001")
        binary: String,
    },

    /// Get the index (0-1023) of a SLIP-39 word
    #[command(name = "word-to-index")]
    WordToIndex {
        /// The SLIP-39 word to look up
        word: String,

        /// Allow fuzzy matching by unique prefix
        #[arg(long, short)]
        prefix: bool,
    },

    /// Get the SLIP-39 word at a specific index (0-1023)
    #[command(name = "index-to-word")]
    IndexToWord {
        /// The index (0-1023) of the word to retrieve
        index: usize,
    },

    /// Explain a word: show word -> index -> bits
    #[command(name = "explain")]
    Explain {
        /// The SLIP-39 word to explain
        word: String,

        /// Allow fuzzy matching by unique prefix
        #[arg(long, short)]
        prefix: bool,
    },
}

#[derive(ValueEnum, Clone, Debug)]
enum InputModeArg {
    Word,
    Binary,
}

// Helper to find input word, optionally using prefix matching (placeholder logic for now)
fn find_word(word: &str, _use_prefix: bool) -> Result<String, slip39_calculator::Error> {
    // For now, no fuzzy search in core lib, so just normalize
    let normalized = word.trim().to_lowercase();
    if wordlist().contains(&normalized.as_str()) {
        Ok(normalized)
    } else {
        Err(slip39_calculator::Error::WordNotFound(word.to_string()))
    }
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Tui { paper, mode }) => {
            if let Err(e) = tui::run(
                paper,
                mode.map(|m| match m {
                    InputModeArg::Word => tui::InputMode::Word,
                    InputModeArg::Binary => tui::InputMode::Binary,
                }),
            ) {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
        }
        None => {
            // Default to TUI
            if let Err(e) = tui::run(false, None) {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
        }
        Some(command) => {
            let result: Result<String, slip39_calculator::Error> = match command {
                Commands::EncodeWord { word, prefix } => {
                    find_word(&word, prefix).and_then(|w| encode(&w))
                }

                Commands::DecodeBits { binary } => decode(&binary),

                Commands::WordToIndex { word, prefix } => find_word(&word, prefix).and_then(|w| {
                    wordlist()
                        .iter()
                        .position(|&list_word| list_word == w)
                        .map(|index| index.to_string())
                        .ok_or(slip39_calculator::Error::WordNotFound(w))
                }),

                Commands::IndexToWord { index } => {
                    if index < wordlist().len() {
                        Ok(wordlist()[index].to_string())
                    } else {
                        Err(slip39_calculator::Error::InvalidBinaryLength(index))
                    }
                }

                Commands::Explain { word, prefix } => find_word(&word, prefix).and_then(|w| {
                    let index = wordlist()
                        .iter()
                        .position(|&list_word| list_word == w)
                        .unwrap();

                    let bits = encode(&w)?;

                    Ok(format!("{} -> {} -> {}", w, index, bits))
                }),

                Commands::Tui { .. } => unreachable!(), // Handled above
            };

            match result {
                Ok(output) => println!("{}", output),
                Err(error) => {
                    eprintln!("Error: {}", error);
                    process::exit(1);
                }
            }
        }
    }
}
