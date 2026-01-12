//! SLIP-39 wordlist encoder/decoder
//!
//! This library provides functions to encode and decode SLIP-39 mnemonic words
//! to/from their 10-bit binary representation.

use std::sync::OnceLock;

/// Errors that can occur during encoding/decoding
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Word '{0}' not found in SLIP-39 wordlist")]
    WordNotFound(String),

    #[error("Invalid binary string: {0}")]
    InvalidBinary(String),

    #[error("Binary must be exactly 10 bits, got {0} bits")]
    InvalidBinaryLength(usize),
}

/// The complete SLIP-39 wordlist (1024 words)
/// Loaded lazily from const/wordlist.txt on first access
/// Official source: https://github.com/satoshilabs/slips/blob/1524583/slip-0039/wordlist.txt
/// Commit: 1524583213f1392321109b0ff0a91330836ecb32 (2019-03-02)
pub static WORDLIST: OnceLock<Vec<&'static str>> = OnceLock::new();

/// Get the WORDLIST, initializing it if necessary
/// This function is public to allow integration tests to access the wordlist
pub fn wordlist() -> &'static [&'static str] {
    WORDLIST.get_or_init(|| {
        include_str!("../const/wordlist.txt")
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| line.trim())
            .collect()
    })
}

/// SHA256 checksum of the official wordlist.txt file
/// This ensures the wordlist matches the official SLIP-39 specification.
/// Official commit: 1524583213f1392321109b0ff0a91330836ecb32 (2019-03-02)
/// If this test fails, either:
/// 1. The wordlist file was corrupted (restore from git)
/// 2. You intentionally updated it (update this constant via `shasum -a 256 const/wordlist.txt`)
pub const WORDLIST_SHA256: &str =
    "bcc4555340332d169718aed8bf31dd9d5248cb7da6e5d355140ef4f1e601eec3";

/// Encode a SLIP-39 word to its 10-bit binary representation
///
/// # Arguments
/// * `word` - A word from the SLIP-39 wordlist
///
/// # Returns
/// * `Ok(String)` - 10-bit binary string (e.g., "0000000001")
/// * `Err(Error::WordNotFound)` - If word is not in the wordlist
///
/// # Example
/// ```
/// use slip39_calculator::encode;
///
/// let binary = encode("acid").unwrap();
/// assert_eq!(binary, "0000000001");
/// ```
pub fn encode(word: &str) -> Result<String, Error> {
    wordlist()
        .iter()
        .position(|&w| w == word)
        .map(|index| format!("{:010b}", index))
        .ok_or_else(|| Error::WordNotFound(word.to_string()))
}

/// Decode a 10-bit binary string to its SLIP-39 word
///
/// # Arguments
/// * `binary` - A 10-bit binary string (e.g., "0000000001")
///
/// # Returns
/// * `Ok(String)` - The corresponding SLIP-39 word
/// * `Err(Error)` - If binary is invalid or out of range
///
/// # Example
/// ```
/// use slip39_calculator::decode;
///
/// let word = decode("0000000001").unwrap();
/// assert_eq!(word, "acid");
/// ```
pub fn decode(binary: &str) -> Result<String, Error> {
    // Validate length
    if binary.len() != 10 {
        return Err(Error::InvalidBinaryLength(binary.len()));
    }

    // Validate characters
    if !binary.chars().all(|c| c == '0' || c == '1') {
        return Err(Error::InvalidBinary(
            "Binary string must only contain '0' and '1'".to_string(),
        ));
    }

    // Parse binary to index
    let index =
        usize::from_str_radix(binary, 2).map_err(|e| Error::InvalidBinary(e.to_string()))?;

    // Get word from wordlist
    wordlist()
        .get(index)
        .map(|&w| w.to_string())
        .ok_or_else(|| {
            Error::InvalidBinary(format!("Index {} out of wordlist range (0-1023)", index))
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_first_word() {
        assert_eq!(encode("academic").unwrap(), "0000000000");
    }

    #[test]
    fn test_encode_second_word() {
        assert_eq!(encode("acid").unwrap(), "0000000001");
    }

    #[test]
    fn test_encode_last_word() {
        assert_eq!(encode("zero").unwrap(), "1111111111");
    }

    #[test]
    fn test_encode_word_not_found() {
        let result = encode("notaword");
        assert!(result.is_err());
        assert!(matches!(result, Err(Error::WordNotFound(_))));
    }

    #[test]
    fn test_decode_first_word() {
        assert_eq!(decode("0000000000").unwrap(), "academic");
    }

    #[test]
    fn test_decode_second_word() {
        assert_eq!(decode("0000000001").unwrap(), "acid");
    }

    #[test]
    fn test_decode_last_word() {
        assert_eq!(decode("1111111111").unwrap(), "zero");
    }

    #[test]
    fn test_decode_invalid_length() {
        let result = decode("01010");
        assert!(result.is_err());
        assert!(matches!(result, Err(Error::InvalidBinaryLength(5))));
    }

    #[test]
    fn test_decode_invalid_characters() {
        let result = decode("01010abcde");
        assert!(result.is_err());
        assert!(matches!(result, Err(Error::InvalidBinary(_))));
    }

    #[test]
    fn test_roundtrip() {
        let word = "acquire";
        let binary = encode(word).unwrap();
        let decoded = decode(&binary).unwrap();
        assert_eq!(decoded, word);
    }

    #[test]
    fn test_wordlist_checksum() {
        use sha2::{Digest, Sha256};
        use std::fs;

        // Read the official wordlist.txt file
        let content = fs::read("const/wordlist.txt").expect("Failed to read const/wordlist.txt");

        // Calculate SHA256
        let mut hasher = Sha256::new();
        hasher.update(&content);
        let result = hasher.finalize();
        let hash = format!("{:x}", result);

        assert_eq!(
            hash, WORDLIST_SHA256,
            "Wordlist checksum mismatch!\n\
             Expected: {}\n\
             Got:      {}\n\
             \n\
             The official wordlist.txt file has been modified.\n\
             This should match commit 1524583213f1392321109b0ff0a91330836ecb32 from:\n\
             https://github.com/satoshilabs/slips/blob/master/slip-0039/wordlist.txt\n\
             \n\
             If this was intentional, update WORDLIST_SHA256.\n\
             Otherwise, restore the file from git: git checkout const/wordlist.txt",
            WORDLIST_SHA256, hash
        );
    }

    #[test]
    fn test_wordlist_initialization() {
        // Verify wordlist initializes correctly
        let words = wordlist();
        assert_eq!(
            words.len(),
            1024,
            "SLIP-39 wordlist must have exactly 1024 words"
        );

        // Verify first and last words
        assert_eq!(words[0], "academic");
        assert_eq!(words[1023], "zero");
    }
}
