#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::manual_range_contains
)]

//! Probar — shell-demo: lookup() drives the *real* trained
//! aprender-shell-base.apr 3-gram Markov model (vendored from
//! presentar::browser::shell_autocomplete). We assert structural
//! invariants that hold for the trained model, not specific
//! hand-curated suggestions — the corpus was trained on real shell
//! history and exact strings would be brittle to model retraining.

use wasm_demos::logic::shell::{lookup, TOP_K};
use wasm_demos::shell_model::ShellAutocomplete;

const MODEL_BYTES: &[u8] = include_bytes!("../assets/aprender-shell-base.apr");

#[test]
fn probar_shell_model_loads_with_expected_shape() {
    // Belt-and-suspenders: the same bytes the demo embeds at runtime
    // also deserialise cleanly here, and the resulting model has the
    // shape upstream documents (3-gram, ~380 commands).
    let m = ShellAutocomplete::load_from_bytes(MODEL_BYTES).expect("model loads");
    assert_eq!(m.ngram_size(), 3, "shipped model is 3-gram");
    assert!(
        m.vocab_size() >= 300 && m.vocab_size() <= 500,
        "shipped model vocab should be ~380 per MI-006, got {}",
        m.vocab_size()
    );
}

#[test]
fn probar_shell_known_prefixes_return_real_completions() {
    // Pick prefixes guaranteed to exist in any real shell-history-trained
    // model. Assert only structural invariants that hold for *any* such
    // model: non-empty result, all entries start with the prefix, ≤ TOP_K.
    for prefix in [
        "g", "git", "git ", "git s", "git p", "c", "cargo", "d", "docker", "k", "ls",
    ] {
        let sugs = lookup(prefix);
        assert!(
            !sugs.is_empty(),
            "real model should return ≥1 suggestion for {prefix:?}"
        );
        assert!(
            sugs.len() <= TOP_K,
            "lookup({prefix:?}) returned {} > TOP_K={TOP_K}",
            sugs.len()
        );
        for s in &sugs {
            assert!(
                s.starts_with(prefix),
                "lookup({prefix:?}) returned {s:?} which doesn't start with the prefix",
            );
        }
    }
}

#[test]
fn probar_shell_unknown_prefix_returns_empty() {
    // No trained command starts with these, so the trie walk returns
    // nothing and the panel cleanly empties.
    assert!(
        lookup("").is_empty(),
        "empty input bypasses the model and stays empty"
    );
    assert!(lookup("zzz").is_empty());
    assert!(lookup("xyz").is_empty());
}

#[test]
fn probar_shell_narrower_prefix_narrows_results() {
    // The longer the prefix, the more specific the result set. We don't
    // assert exact strings (corpus-dependent), only that adding chars
    // never produces something outside the new prefix.
    let coarse = lookup("git ");
    let fine = lookup("git s");
    assert!(!coarse.is_empty() && !fine.is_empty());
    for s in &fine {
        assert!(
            s.starts_with("git s"),
            "lookup(\"git s\") leaked {s:?} (no longer starts with \"git s\")"
        );
    }
}
