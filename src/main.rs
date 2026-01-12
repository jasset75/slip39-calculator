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

// Helper to find input word, optionally using prefix matching
fn find_word(word: &str, use_prefix: bool) -> Result<String, String> {
    if use_prefix {
        slip39_calculator::find_by_prefix(word).map_err(|e| format!("{}", e))
    } else {
        // Default behavior: just normalize and check strictly (via encode internal logic or manual check)
        // Since we want to return the *correctly cased* word for further processing,
        // we essentially do what encode does but return the word itself.
        let normalized = word.trim().to_lowercase();
        if wordlist().contains(&normalized.as_str()) {
            Ok(normalized)
        } else {
            Err(format!("Word '{}' not found in SLIP-39 wordlist", word))
        }
    }
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::EncodeWord { word, prefix } => {
            find_word(&word, prefix).and_then(|w| encode(&w).map_err(|e| format!("{}", e)))
        }

        Commands::DecodeBits { binary } => decode(&binary).map_err(|e| format!("{}", e)),

        Commands::WordToIndex { word, prefix } => find_word(&word, prefix).and_then(|w| {
            wordlist()
                .iter()
                .position(|&list_word| list_word == w)
                .map(|index| index.to_string())
                .ok_or_else(|| format!("Word '{}' not found in SLIP-39 wordlist", w))
        }),

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

        Commands::Explain { word, prefix } => {
            find_word(&word, prefix).and_then(|w| {
                let index = wordlist()
                    .iter()
                    .position(|&list_word| list_word == w)
                    .unwrap(); // find_word guarantees it's in list

                let bits = encode(&w).map_err(|e| format!("{}", e))?;

                Ok(format!("{} -> {} -> {}", w, index, bits))
            })
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
