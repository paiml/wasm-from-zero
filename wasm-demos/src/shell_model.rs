//! Real `aprender-shell` 3-gram Markov autocomplete — vendored from
//! `presentar::browser::shell_autocomplete` (aprender-present-lib 0.34).
//!
//! The umbrella `aprender-present-lib` 0.34 published crate has a broken
//! `include_bytes!` path in its `showcase` module that prevents it from
//! building for `wasm32-unknown-unknown` (see the matching workaround in
//! `m5-dash/Cargo.toml`). To still use the *real* trained model, we vendor
//! just the loader and inference paths here. The bytes of
//! `assets/aprender-shell-base.apr` are byte-for-byte identical to what
//! `interactive.paiml.com/shell-ml` serves (SHA256
//! `068ac67a89693d2773adc4b850aca5dbb65102653dd27239c960b42e5a7e3974`).
//!
//! Differences from upstream:
//! * Drops the `wasm_bindgen`-exported `ShellAutocompleteDemo` wrapper —
//!   our demo binds directly to `ShellAutocomplete`.
//! * Drops the host-side `include_bytes!` embed — `logic::shell` does
//!   that closer to the demo to keep this module pure.
//! * Uses pure-Rust `ruzstd` for zstd decode instead of the C-binding
//!   `zstd` crate — no C toolchain required for `wasm32`.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::missing_const_for_fn,
    clippy::module_name_repetitions
)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Read;

const HEADER_SIZE: usize = 32;

/// 3-gram Markov shell-command predictor with trie prefix matching.
#[derive(Debug)]
pub struct ShellAutocomplete {
    n: usize,
    ngrams: HashMap<String, HashMap<String, u32>>,
    command_freq: HashMap<String, u32>,
    trie: Trie,
    total_commands: usize,
}

#[derive(Debug, Default)]
struct Trie {
    children: HashMap<char, Trie>,
    is_end: bool,
    command: Option<String>,
}

impl Trie {
    fn new() -> Self {
        Self::default()
    }

    fn insert(&mut self, word: &str) {
        let mut node = self;
        for c in word.chars() {
            node = node.children.entry(c).or_default();
        }
        node.is_end = true;
        node.command = Some(word.to_string());
    }

    fn find_prefix(&self, prefix: &str, limit: usize) -> Vec<String> {
        let mut results = Vec::new();
        let mut node = self;
        for c in prefix.chars() {
            match node.children.get(&c) {
                Some(child) => node = child,
                None => return results,
            }
        }
        Self::collect_commands_recursive(node, &mut results, limit);
        results
    }

    fn collect_commands_recursive(node: &Trie, results: &mut Vec<String>, limit: usize) {
        if results.len() >= limit {
            return;
        }
        if let Some(ref cmd) = node.command {
            results.push(cmd.clone());
        }
        for child in node.children.values() {
            Self::collect_commands_recursive(child, results, limit);
            if results.len() >= limit {
                return;
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct MarkovModelData {
    n: usize,
    ngrams: HashMap<String, HashMap<String, u32>>,
    command_freq: HashMap<String, u32>,
    total_commands: usize,
    #[serde(default)]
    last_trained_pos: usize,
}

impl ShellAutocomplete {
    /// Load from raw `.apr` bytes.
    ///
    /// Header layout (32 bytes):
    /// - `0..4`   magic `b"APRN"`
    /// - `4..6`   version (major, minor)
    /// - `6..8`   model type (u16 LE)
    /// - `8..12`  metadata size (u32 LE)
    /// - `12..16` payload size (u32 LE)
    /// - `16..20` uncompressed size (u32 LE)
    /// - `20`     compression type (`0x00` = none, `0x01`/`0x02` = zstd)
    /// - `21`     flags
    /// - `22..32` reserved
    pub fn load_from_bytes(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() < HEADER_SIZE {
            return Err("Model file too small".to_string());
        }
        if &bytes[0..4] != b"APRN" {
            return Err(format!("Invalid magic bytes: {:?}", &bytes[0..4]));
        }

        let metadata_size = u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]) as usize;
        let payload_size =
            u32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]) as usize;
        let compression = bytes[20];

        let metadata_start = HEADER_SIZE;
        let metadata_end = metadata_start + metadata_size;
        let payload_start = metadata_end;
        let payload_end = payload_start + payload_size;

        if payload_end > bytes.len() {
            return Err(format!(
                "Payload extends beyond file: {} > {}",
                payload_end,
                bytes.len()
            ));
        }

        let payload_compressed = &bytes[payload_start..payload_end];

        let payload_decompressed: Vec<u8> = match compression {
            0x00 => payload_compressed.to_vec(),
            0x01 | 0x02 => {
                let mut decoder = ruzstd::StreamingDecoder::new(payload_compressed)
                    .map_err(|e| format!("zstd init: {e}"))?;
                let mut out = Vec::new();
                decoder
                    .read_to_end(&mut out)
                    .map_err(|e| format!("zstd decode: {e}"))?;
                out
            }
            _ => return Err(format!("Unknown compression type: 0x{compression:02X}")),
        };

        let model_data: MarkovModelData = bincode::deserialize(&payload_decompressed)
            .map_err(|e| format!("Failed to deserialize model: {e}"))?;

        let mut trie = Trie::new();
        for cmd in model_data.command_freq.keys() {
            trie.insert(cmd);
        }

        Ok(Self {
            n: model_data.n,
            ngrams: model_data.ngrams,
            command_freq: model_data.command_freq,
            trie,
            total_commands: model_data.total_commands,
        })
    }

    /// Suggest completions for a prefix, ranked by score.
    pub fn suggest(&self, prefix: &str, count: usize) -> Vec<(String, f32)> {
        let prefix = prefix.trim();
        let tokens: Vec<&str> = prefix.split_whitespace().collect();
        let ends_with_space = prefix.is_empty() || prefix.ends_with(' ');

        let capacity = count * 4;
        let mut suggestions = Vec::with_capacity(capacity);
        let mut seen = std::collections::HashSet::with_capacity(capacity);

        for cmd in self.trie.find_prefix(prefix, capacity) {
            if Self::is_corrupted_command(&cmd) {
                continue;
            }
            let freq = self.command_freq.get(&cmd).copied().unwrap_or(1);
            let score = freq as f32 / self.total_commands.max(1) as f32;
            seen.insert(cmd.clone());
            suggestions.push((cmd, score));
        }

        if !tokens.is_empty() && ends_with_space {
            let context_start = tokens.len().saturating_sub(self.n - 1);
            let context = tokens[context_start..].join(" ");
            let prefix_trimmed = prefix.trim();

            if let Some(next_tokens) = self.ngrams.get(&context) {
                let total: u32 = next_tokens.values().sum();
                let mut completion = String::with_capacity(prefix_trimmed.len() + 32);
                for (token, ngram_count) in next_tokens {
                    completion.clear();
                    completion.push_str(prefix_trimmed);
                    completion.push(' ');
                    completion.push_str(token);
                    let score = *ngram_count as f32 / total as f32;
                    if !seen.contains(&completion) {
                        seen.insert(completion.clone());
                        suggestions.push((completion.clone(), score * 0.8));
                    }
                }
            }
        }

        if !tokens.is_empty() && !ends_with_space && tokens.len() >= 2 {
            let partial_token = tokens.last().unwrap_or(&"");
            let context_tokens = &tokens[..tokens.len() - 1];
            let context_start = context_tokens.len().saturating_sub(self.n - 1);
            let context = context_tokens[context_start..].join(" ");
            let context_prefix = context_tokens.join(" ");

            if let Some(next_tokens) = self.ngrams.get(&context) {
                let total: u32 = next_tokens.values().sum();
                let mut completion = String::with_capacity(context_prefix.len() + 32);
                for (token, ngram_count) in next_tokens {
                    if token.starts_with(partial_token) && !Self::is_corrupted_token(token) {
                        completion.clear();
                        completion.push_str(&context_prefix);
                        completion.push(' ');
                        completion.push_str(token);
                        let score = *ngram_count as f32 / total as f32;
                        if !seen.contains(&completion) {
                            seen.insert(completion.clone());
                            suggestions.push((completion.clone(), score * 0.9));
                        }
                    }
                }
            }
        }

        if prefix.is_empty() && suggestions.is_empty() {
            let mut top_cmds: Vec<_> = self
                .command_freq
                .iter()
                .map(|(k, v)| (k.clone(), *v as f32 / self.total_commands.max(1) as f32))
                .collect();
            top_cmds.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            suggestions = top_cmds;
        }

        suggestions.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        suggestions.truncate(count);
        suggestions
    }

    fn is_corrupted_command(cmd: &str) -> bool {
        if cmd.contains("  ") {
            return true;
        }
        if cmd.trim_end().ends_with('\\') {
            return true;
        }
        cmd.split_whitespace().any(Self::is_corrupted_token)
    }

    fn is_corrupted_token(token: &str) -> bool {
        if let Some(dash_pos) = token.find('-') {
            if dash_pos > 0 && dash_pos < token.len() - 1 {
                let before = &token[..dash_pos];
                let after = &token[dash_pos + 1..];
                let subcommands = [
                    "commit", "checkout", "clone", "push", "pull", "merge", "rebase", "status",
                    "add", "build", "run", "test", "install",
                ];
                if subcommands.contains(&before) && (after.len() <= 2 || after.starts_with('-')) {
                    return true;
                }
            }
        }
        false
    }

    /// Number of unique commands in the model — used by tests to verify
    /// the right `.apr` file got loaded (the shipped base model has ~380
    /// commands per the upstream MI-006 spec).
    pub fn vocab_size(&self) -> usize {
        self.command_freq.len()
    }

    /// N-gram order — used by tests to verify the 3-gram base model.
    pub fn ngram_size(&self) -> usize {
        self.n
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MODEL_BYTES: &[u8] = include_bytes!("../assets/aprender-shell-base.apr");

    /// Build a synthetic APR header for an uncompressed payload — the
    /// shipped model uses zstd (0x01), so an uncompressed (0x00) test
    /// case is the only way to exercise that branch.
    fn make_apr(metadata: &[u8], payload: &[u8], compression: u8) -> Vec<u8> {
        let mut buf = Vec::with_capacity(HEADER_SIZE + metadata.len() + payload.len());
        buf.extend_from_slice(b"APRN");
        buf.extend_from_slice(&[1, 0]); // version
        buf.extend_from_slice(&[0x10, 0]); // model type
        buf.extend_from_slice(&(metadata.len() as u32).to_le_bytes());
        buf.extend_from_slice(&(payload.len() as u32).to_le_bytes());
        buf.extend_from_slice(&(payload.len() as u32).to_le_bytes()); // uncompressed
        buf.push(compression);
        buf.push(0); // flags
        buf.extend_from_slice(&[0u8; 10]); // reserved
        buf.extend_from_slice(metadata);
        buf.extend_from_slice(payload);
        buf
    }

    fn make_minimal_model_payload() -> Vec<u8> {
        let mut ngrams: HashMap<String, HashMap<String, u32>> = HashMap::new();
        let mut next = HashMap::new();
        next.insert("status".to_string(), 5);
        next.insert("commit".to_string(), 3);
        ngrams.insert("git".to_string(), next);
        let mut command_freq = HashMap::new();
        command_freq.insert("git status".to_string(), 5);
        command_freq.insert("git commit".to_string(), 3);
        command_freq.insert("ls -la".to_string(), 2);
        let data = MarkovModelData {
            n: 3,
            ngrams,
            command_freq,
            total_commands: 10,
            last_trained_pos: 0,
        };
        bincode::serialize(&data).unwrap()
    }

    #[test]
    fn shipped_model_loads_and_answers() {
        let m = ShellAutocomplete::load_from_bytes(MODEL_BYTES).expect("load shipped model");
        assert_eq!(m.ngram_size(), 3);
        assert!(m.vocab_size() >= 300);
        // Real model returns suggestions for `git` and they all start with it.
        let s = m.suggest("git", 5);
        assert!(!s.is_empty());
        for (text, _) in &s {
            assert!(text.starts_with("git"));
        }
        // Scores are sorted descending.
        for w in s.windows(2) {
            assert!(w[0].1 >= w[1].1);
        }
    }

    #[test]
    fn rejects_short_input() {
        let err = ShellAutocomplete::load_from_bytes(&[0u8; 8]).unwrap_err();
        assert!(err.contains("too small"));
    }

    #[test]
    fn rejects_bad_magic() {
        let mut bytes = vec![0u8; HEADER_SIZE];
        bytes[0..4].copy_from_slice(b"XXXX");
        let err = ShellAutocomplete::load_from_bytes(&bytes).unwrap_err();
        assert!(err.contains("magic"));
    }

    #[test]
    fn rejects_payload_overrun() {
        // Header claims a 999-byte payload but only ~32 bytes are present.
        let mut bytes = vec![0u8; HEADER_SIZE];
        bytes[0..4].copy_from_slice(b"APRN");
        bytes[12..16].copy_from_slice(&999u32.to_le_bytes());
        let err = ShellAutocomplete::load_from_bytes(&bytes).unwrap_err();
        assert!(err.contains("Payload extends beyond file"));
    }

    #[test]
    fn rejects_unknown_compression() {
        let bytes = make_apr(b"", b"junk", 0x77);
        let err = ShellAutocomplete::load_from_bytes(&bytes).unwrap_err();
        assert!(err.contains("Unknown compression"));
    }

    #[test]
    fn rejects_undecodable_bincode_payload() {
        // compression=0x00 → payload taken verbatim, then bincode fails.
        let bytes = make_apr(b"", b"not a real bincode payload", 0x00);
        let err = ShellAutocomplete::load_from_bytes(&bytes).unwrap_err();
        assert!(err.contains("deserialize"));
    }

    #[test]
    fn rejects_invalid_zstd_payload() {
        let bytes = make_apr(b"", b"not real zstd", 0x01);
        // Either init or decode will fail — we accept either label.
        let err = ShellAutocomplete::load_from_bytes(&bytes).unwrap_err();
        assert!(err.contains("zstd"));
    }

    #[test]
    fn synthetic_uncompressed_model_round_trips() {
        let payload = make_minimal_model_payload();
        let bytes = make_apr(b"", &payload, 0x00);
        let m = ShellAutocomplete::load_from_bytes(&bytes).unwrap();
        assert_eq!(m.ngram_size(), 3);
        assert_eq!(m.vocab_size(), 3);
        // Trie hits.
        let s = m.suggest("git", 10);
        assert!(s.iter().any(|(t, _)| t == "git status"));
        // N-gram step (prefix ends with space).
        let s = m.suggest("git ", 10);
        assert!(s.iter().any(|(t, _)| t == "git status"));
        // Partial-token n-gram step (mid-typing, prefix doesn't end space).
        let s = m.suggest("git s", 10);
        assert!(s.iter().any(|(t, _)| t == "git status"));
        // Empty prefix on a non-empty model falls back to top-by-frequency.
        let s = m.suggest("", 10);
        assert!(!s.is_empty());
    }

    #[test]
    fn corrupted_token_filter_catches_known_patterns() {
        // `commit-m` matches before='commit', after='m' (len<=2) → corrupted.
        assert!(ShellAutocomplete::is_corrupted_token("commit-m"));
        assert!(ShellAutocomplete::is_corrupted_token("checkout-b"));
        // Subcommand followed by --flag-name → also flagged.
        assert!(ShellAutocomplete::is_corrupted_token("push---force"));
        // Genuine flag-like tokens that aren't the bug pattern aren't flagged.
        assert!(!ShellAutocomplete::is_corrupted_token("clean"));
        assert!(!ShellAutocomplete::is_corrupted_token("commit"));
        assert!(!ShellAutocomplete::is_corrupted_token("-h"));
        assert!(!ShellAutocomplete::is_corrupted_token("unknown-subcommand"));
    }

    #[test]
    fn corrupted_command_filter_catches_known_patterns() {
        assert!(ShellAutocomplete::is_corrupted_command("foo  bar")); // double space
        assert!(ShellAutocomplete::is_corrupted_command("foo \\")); // trailing backslash
        assert!(ShellAutocomplete::is_corrupted_command("git commit-m")); // bad token
        assert!(!ShellAutocomplete::is_corrupted_command("git commit -m"));
        assert!(!ShellAutocomplete::is_corrupted_command("ls -la"));
    }

    #[test]
    fn trie_walks_then_collects() {
        let mut t = Trie::new();
        t.insert("git status");
        t.insert("git stash");
        t.insert("git commit");
        let mut all = t.find_prefix("git ", 10);
        all.sort();
        assert_eq!(all, vec!["git commit", "git stash", "git status"]);
        // Limit honoured.
        let two = t.find_prefix("git ", 2);
        assert_eq!(two.len(), 2);
        // Missing prefix → empty.
        assert!(t.find_prefix("xyz", 10).is_empty());
    }
}
