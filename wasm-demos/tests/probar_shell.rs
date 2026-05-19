#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::manual_range_contains
)]

//! Probar — shell-demo: lookup() returns the right top-3 per prefix;
//! longest-prefix wins; unknown prefix returns empty.

use wasm_demos::logic::shell::lookup;

#[test]
fn probar_shell_known_prefixes() {
    for (prefix, expected_top) in [
        ("g", "git status"),
        ("git", "git status"),
        ("git s", "git status"),
        ("git p", "git push"),
        ("c", "cargo build"),
        ("cargo", "cargo build"),
        ("cargo b", "cargo build"),
        ("cargo t", "cargo test"),
        ("d", "docker ps"),
        ("docker", "docker ps"),
        ("k", "kubectl get pods"),
        ("ls", "ls -la"),
        ("m", "make demo"),
        ("make", "make demo"),
        ("python", "python3 -m http.server"),
        ("ssh", "ssh dev"),
    ] {
        let sugs = lookup(prefix);
        assert!(
            !sugs.is_empty(),
            "prefix {prefix:?} should return suggestions"
        );
        assert_eq!(sugs[0], expected_top, "top match for {prefix:?}");
        assert!(sugs.len() <= 3, "should be top-3 at most");
    }
}

#[test]
fn probar_shell_longest_prefix_wins() {
    // "git " (4 chars) → general git suggestions
    // "git s" (5 chars) → git status / git stash / git show
    let s1 = lookup("git ");
    let s2 = lookup("git s");
    assert_ne!(s1, s2, "longer prefix should override shorter");
    assert_eq!(s2[0], "git status");
    assert_eq!(s2[1], "git stash");
}

#[test]
fn probar_shell_unknown_prefix_returns_empty() {
    assert!(
        lookup("").is_empty(),
        "empty prefix has no longest-prefix match"
    );
    assert!(lookup("zzz").is_empty());
    assert!(lookup("xyz").is_empty());
}

#[test]
fn probar_shell_all_entries_return_at_most_3() {
    use wasm_demos::logic::shell::SUGGESTIONS;
    for (prefix, _) in SUGGESTIONS {
        let sugs = lookup(prefix);
        assert!(
            sugs.len() <= 3,
            "lookup({prefix:?}) returned {} ≠ ≤3",
            sugs.len()
        );
        assert!(!sugs.is_empty(), "every dictionary prefix should return ≥1");
    }
}
