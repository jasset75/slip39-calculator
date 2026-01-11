use slip39_calculator::{decode, encode, WORDLIST};

#[test]
fn test_encode_decode_roundtrip() {
    // Test roundtrip for first 10 words
    for word in WORDLIST.iter().take(10) {
        let binary = encode(word).unwrap();
        let decoded = decode(&binary).unwrap();
        assert_eq!(decoded, *word);
    }
}

#[test]
fn test_wordlist_length() {
    // SLIP-39 wordlist must have exactly 1024 words (2^10)
    assert_eq!(WORDLIST.len(), 1024);
}

#[test]
fn test_wordlist_alphabetical_order() {
    // Verify wordlist is in alphabetical order
    for i in 0..WORDLIST.len() - 1 {
        assert!(
            WORDLIST[i] < WORDLIST[i + 1],
            "Wordlist not in alphabetical order at index {}: '{}' should come before '{}'",
            i,
            WORDLIST[i],
            WORDLIST[i + 1]
        );
    }
}

#[test]
fn test_first_and_last_words() {
    // Verify first and last words match SLIP-39 specification
    assert_eq!(WORDLIST[0], "academic");
    assert_eq!(WORDLIST[1023], "zero");
}

#[test]
#[ignore] // Only run with: cargo test -- --ignored
fn test_wordlist_matches_official_slip39() {
    // This test downloads the official SLIP-39 wordlist and compares
    // Run with: cargo test -- --ignored --test-threads=1
    
    const OFFICIAL_URL: &str = 
        "https://raw.githubusercontent.com/satoshilabs/slips/master/slip-0039/wordlist.txt";
    
    // Download official wordlist
    let response = reqwest::blocking::get(OFFICIAL_URL)
        .expect("Failed to download official SLIP-39 wordlist");
    
    let official_text = response
        .text()
        .expect("Failed to read response text");
    
    // Parse words from official list
    let official_words: Vec<&str> = official_text
        .lines()
        .filter(|line| !line.is_empty())
        .collect();
    
    // Compare lengths
    assert_eq!(
        WORDLIST.len(),
        official_words.len(),
        "Wordlist length mismatch. Expected {}, got {}",
        official_words.len(),
        WORDLIST.len()
    );
    
    // Compare each word
    for (i, (local, official)) in WORDLIST.iter().zip(official_words.iter()).enumerate() {
        assert_eq!(
            local, official,
            "Word mismatch at index {}: local='{}', official='{}'",
            i, local, official
        );
    }
    
    println!("✓ Wordlist matches official SLIP-39 specification ({} words)", WORDLIST.len());
}
