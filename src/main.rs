use clap::{Parser, Subcommand};
use slip39_calculator::{decode, encode, wordlist};
use std::process;

/// SLIP-39 wordlist encoder/decoder
///
/// A CLI tool for encoding and decoding SLIP-39 mnemonic words
/// to/from their 10-bit binary representation.
#[derive(Parser)]
#[command(name = "slip39c")]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Encode a SLIP-39 word to its 10-bit binary representation
    ///
    /// Example: slip39c encode-word academic
    /// Output: 0000000000
    #[command(name = "encode-word")]
    EncodeWord {
        /// The SLIP-39 word to encode
        word: String,
    },

    /// Decode a 10-bit binary string to its SLIP-39 word
    ///
    /// Example: slip39c decode-bits 0000000001
    /// Output: acid
    #[command(name = "decode-bits")]
    DecodeBits {
        /// The 10-bit binary string to decode (e.g., "0000000001")
        binary: String,
    },

    /// Get the index (0-1023) of a SLIP-39 word
    ///
    /// Example: slip39c word-to-index academic
    /// Output: 0
    #[command(name = "word-to-index")]
    WordToIndex {
        /// The SLIP-39 word to look up
        word: String,
    },

    /// Get the SLIP-39 word at a specific index (0-1023)
    ///
    /// Example: slip39c index-to-word 1023
    /// Output: zero
    #[command(name = "index-to-word")]
    IndexToWord {
        /// The index (0-1023) of the word to retrieve
        index: usize,
    },
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::EncodeWord { word } => encode(&word).map_err(|e| format!("{}", e)),

        Commands::DecodeBits { binary } => decode(&binary).map_err(|e| format!("{}", e)),

        Commands::WordToIndex { word } => wordlist()
            .iter()
            .position(|&w| w == word)
            .map(|index| index.to_string())
            .ok_or_else(|| format!("Word '{}' not found in SLIP-39 wordlist", word)),

        Commands::IndexToWord { index } => {
            if index > 1023 {
                Err(format!("Index {} out of range (must be 0-1023)", index))
            } else {
                wordlist()
                    .get(index)
                    .map(|&w| w.to_string())
                    .ok_or_else(|| format!("Index {} not found in wordlist", index))
            }
        }
    };

    match result {
        Ok(output) => println!("{}", output),
        Err(error) => {
            eprintln!("Error: {}", error);
            process::exit(1);
        }
    }
}
