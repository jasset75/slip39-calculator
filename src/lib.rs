//! SLIP-39 wordlist encoder/decoder
//!
//! This library provides functions to encode and decode SLIP-39 mnemonic words
//! to/from their 10-bit binary representation.

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
pub const WORDLIST: &[&str] = &include!("../const/wordlist_array.txt");

/// SHA256 checksum of the wordlist array file
/// This ensures the wordlist hasn't been accidentally modified.
/// If this test fails, either:
/// 1. The wordlist file was corrupted (restore from git)
/// 2. You intentionally updated it (update this constant via `shasum -a 256 const/wordlist_array.txt`)
pub const WORDLIST_SHA256: &str = "5f8d1360496a206dc80ea5a513f6ab1f36982b8f4b3005d0cfb3ba0302eba0ac";


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
    WORDLIST
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
            "Binary string must only contain '0' and '1'".to_string()
        ));
    }
    
    // Parse binary to index
    let index = usize::from_str_radix(binary, 2)
        .map_err(|e| Error::InvalidBinary(e.to_string()))?;
    
    // Get word from wordlist
    WORDLIST
        .get(index)
        .map(|&w| w.to_string())
        .ok_or_else(|| Error::InvalidBinary(
            format!("Index {} out of wordlist range (0-1023)", index)
        ))
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
        use sha2::{Sha256, Digest};
        
        // Reconstruct the wordlist array file content
        let mut content = String::from("[\n");
        for word in WORDLIST.iter() {
            content.push_str(&format!("\"{}\",\n", word));
        }
        content.push_str("]\n");
        
        // Calculate SHA256
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        let result = hasher.finalize();
        let hash = format!("{:x}", result);
        
        assert_eq!(
            hash, 
            WORDLIST_SHA256,
            "Wordlist checksum mismatch!\n\
             Expected: {}\n\
             Got:      {}\n\
             \n\
             The wordlist file has been modified. If this was intentional, update WORDLIST_SHA256.\n\
             Otherwise, restore the file from git: git checkout const/wordlist_array.txt",
            WORDLIST_SHA256,
            hash
        );
    }
}
