//! Integration tests for prefix lookup feature
//!
//! Verifies:
//! - Unique prefix lookup
//! - Ambiguous prefix detection
//! - Error messages for ambiguous/not found
//! - Case insensitivity and trimming with prefixes

use slip39_calculator::{find_by_prefix, find_matches, Error};

#[test]
fn test_find_matches() {
    // 'ac' should match 'academic', 'acid', 'acoustics', 'acquire', 'acrobat', 'actress', 'actual', 'acupuncture'
    // (exact list depends on wordlist content, but we know it's > 1)
    let matches = find_matches("ac");
    assert!(matches.len() > 1);
    assert!(matches.contains(&"academic"));
    assert!(matches.contains(&"acid"));
}

#[test]
fn test_find_by_prefix_unique() {
    // Unique prefixes should resolve
    assert_eq!(find_by_prefix("aca").unwrap(), "academic");
    assert_eq!(find_by_prefix("aci").unwrap(), "acid");
    assert_eq!(find_by_prefix("zer").unwrap(), "zero");
}

#[test]
fn test_find_by_prefix_exact_match_optimization() {
    // If exact match exists, should return it even if it's a prefix of another (though unlikely in this wordlist)
    assert_eq!(find_by_prefix("academic").unwrap(), "academic");
}

#[test]
fn test_find_by_prefix_ambiguous() {
    // 'ac' matches 'academic', 'acid', 'acquire', etc.
    let result = find_by_prefix("ac");
    assert!(result.is_err());

    if let Err(Error::AmbiguousPrefix(prefix, count, examples)) = result {
        assert_eq!(prefix, "ac");
        assert!(count > 1);
        assert!(examples.contains("academic"));
        assert!(examples.contains("acid"));
    } else {
        panic!("Expected AmbiguousPrefix error");
    }
}

#[test]
fn test_find_by_prefix_not_found() {
    let result = find_by_prefix("xyz");
    assert!(result.is_err());
    assert!(matches!(result, Err(Error::WordNotFound(_))));
}

#[test]
fn test_find_by_prefix_normalization() {
    // Should trim and lowercase before searching
    assert_eq!(find_by_prefix("  ACA  ").unwrap(), "academic");
}
