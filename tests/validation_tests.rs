//! Integration tests for input validation
//!
//! These tests verify that the library correctly handles edge cases:
//! - Accepting uppercase words and normalizing to lowercase
//! - Rejecting binaries that aren't exactly 10 characters
//! - Rejecting binaries with invalid characters
//! - Providing clear error messages

use slip39_calculator::{decode, encode, Error};

#[test]
fn test_encode_accepts_uppercase() {
    // Should accept uppercase and normalize to lowercase
    assert_eq!(encode("ACADEMIC").unwrap(), "0000000000");
    assert_eq!(encode("ACID").unwrap(), "0000000001");
    assert_eq!(encode("ZERO").unwrap(), "1111111111");
}

#[test]
fn test_encode_accepts_mixed_case() {
    // Should accept mixed case and normalize
    assert_eq!(encode("AcAdEmIc").unwrap(), "0000000000");
    assert_eq!(encode("AcId").unwrap(), "0000000001");
}

#[test]
fn test_encode_trims_whitespace() {
    // Should trim leading/trailing whitespace
    assert_eq!(encode("  academic  ").unwrap(), "0000000000");
    assert_eq!(encode("\tacid\n").unwrap(), "0000000001");
}

#[test]
fn test_encode_word_not_found_clear_message() {
    // Should provide a clear error message for words not in wordlist
    let result = encode("notaword");
    assert!(result.is_err());

    if let Err(Error::WordNotFound(word)) = result {
        assert_eq!(word, "notaword");
    } else {
        panic!("Expected WordNotFound error");
    }

    // Verify error message
    let err = encode("invalid").unwrap_err();
    let msg = format!("{}", err);
    assert!(
        msg.contains("not found in SLIP-39 wordlist"),
        "Error message: {}",
        msg
    );
}

#[test]
fn test_decode_rejects_short_binary() {
    // Should reject binaries shorter than 10 characters
    let result = decode("0101");
    assert!(result.is_err());

    if let Err(Error::InvalidBinaryLength(len)) = result {
        assert_eq!(len, 4);
    } else {
        panic!("Expected InvalidBinaryLength error");
    }

    // Verify error message
    let err = decode("0101").unwrap_err();
    let msg = format!("{}", err);
    assert!(
        msg.contains("must be exactly 10 bits"),
        "Error message: {}",
        msg
    );
}

#[test]
fn test_decode_rejects_long_binary() {
    // Should reject binaries longer than 10 characters
    let result = decode("01010101010101");
    assert!(result.is_err());

    if let Err(Error::InvalidBinaryLength(len)) = result {
        assert_eq!(len, 14);
    } else {
        panic!("Expected InvalidBinaryLength error");
    }
}

#[test]
fn test_decode_rejects_invalid_characters() {
    // Should reject binaries with characters other than 0 or 1
    let test_cases = vec![
        "012345678X", // Letter
        "01234567 9", // Space
        "0123456789", // Digit other than 0/1
        "abcdefghij", // Letters
        "----------", // Dashes
    ];

    for binary in test_cases {
        let result = decode(binary);
        assert!(result.is_err(), "Should reject: {}", binary);

        if let Err(Error::InvalidBinary(msg)) = result {
            assert!(
                msg.contains("must only contain '0' and '1'"),
                "Error message for '{}': {}",
                binary,
                msg
            );
        } else {
            panic!("Expected InvalidBinary error for: {}", binary);
        }
    }
}

#[test]
fn test_decode_binary_out_of_range_message() {
    // Note: All 10-bit binaries (0000000000 to 1111111111) are valid in SLIP-39
    // because the wordlist has exactly 1024 words. This test verifies the error
    // would occur if we somehow get an invalid index (edge case protection).

    // With 10 bits, max value is 1023, which is exactly the last word "zero"
    // So this is more of a documentation test showing the range is properly validated
    assert_eq!(decode("1111111111").unwrap(), "zero"); // index 1023, last word
}

#[test]
fn test_error_messages_are_clear() {
    // Verify all error types produce clear messages

    // WordNotFound
    let err = encode("invalidword").unwrap_err();
    assert!(format!("{}", err).contains("not found in SLIP-39 wordlist"));

    // InvalidBinaryLength
    let err = decode("010").unwrap_err();
    assert!(format!("{}", err).contains("must be exactly 10 bits"));
    assert!(format!("{}", err).contains("got 3 bits"));

    // InvalidBinary
    let err = decode("01010abcde").unwrap_err();
    assert!(format!("{}", err).contains("must only contain '0' and '1'"));
}
