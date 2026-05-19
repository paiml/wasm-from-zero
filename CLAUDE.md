# wasm-from-zero

Companion repository for the **TUI from Zero** Coursera course. Built around
[`presentar`](https://github.com/paiml/aprender) — the pure-Rust TUI framework from
`aprender`. Mirrors the c7 design-by-provable-contracts pattern: YAML contracts validated
by `pv`, runtime asserts via demo binaries, Lean 4 proofs at L5.

## Quality Standards

- 95% minimum test coverage
- All quality gates pass before push: `make ci` (fmt + clippy + test + 100% cov + pv lint)
- Zero clippy warnings (`-D warnings`)
- Lean 4 proofs type-check (`make lean-build`)

## Code Search Policy

**NEVER use grep/glob for code search. ALWAYS prefer `pmat query`.**

`pmat query` returns quality-annotated, semantically ranked results with TDG grades,
complexity, fault patterns, and call graphs. Raw grep returns lines.

```bash
pmat query "error handling" --limit 10              # Find by intent
pmat query "diff" --min-grade A                     # Find high-quality examples
pmat query "unwrap" --faults --exclude-tests        # Find fault patterns
pmat query "cellbuffer" --include-source            # Include source code
pmat query --regex "fn\s+test_\w+" --limit 10       # Regex search
pmat query --literal "VERIFICATION SUCCESSFUL"      # Literal string
pmat query "render" --churn --duplicates --faults   # Full enrichment
```

When grep IS acceptable: searching non-code files (TOML, YAML, Markdown) or quick
one-off debugging. `pmat query --literal` and `--regex` cover most ripgrep cases now.

## The pv workflow

Three YAML contracts gate the nine demo crates:

- `contracts/wasm-rendering-v1.yaml` — Render pillar (M1 + M4-tests)
- `contracts/wasm-lifecycle-v1.yaml` — React pillar (M2)
- `contracts/wasm-panels-v1.yaml` — Compose pillar (M3 + M4-scene + M5)

Every demo binary asserts the runtime half of its contract. Lean 4 theorems in
`lean/TuiFromZero/Theorems/` discharge the universal claim at L5.

```bash
make validate    # pv validate per contract — schema gate
make score       # pv score per contract — 5-dim rubric
make lean-build  # cd lean && lake build — type-check all theorems
make demo        # run all 9 demo binaries
make ci          # full pre-merge gate
```

`lake build` MUST run from `lean/` — the lakefile lives there. `make lean-build` does
the `cd` for you.

## Prohibited Tools

- `cargo tarpaulin` — slow, unreliable. Use `cargo llvm-cov` instead.
- `cargo-llvm-cov clean` in Makefiles — prevents caching, slows builds.
