# Script — wasm-from-zero: nine demos, three pillars, zero JavaScript

**Lesson:** the 90-second first impression of presentar's WASM half.
**Visual reference:** terminal screencast — one repo (`paiml/wasm-from-zero`), nine native binaries, three pv contracts.
**Target length:** ~2 min.

## Demo (one terminal, one Makefile — `paiml/wasm-from-zero`)

```bash
cd ~/src/wasm-from-zero
```

```bash
cat contracts/wasm-rendering-v1.yaml
```
**Three named contracts.** `coords_finite` · `color_clamped` · `bounds_clamped` — the three obligations every Canvas2D draw must satisfy. Schema and proof status live next to the code, not in a wiki.

```bash
make validate && make score
```
**pv gates the contracts.** `wasm-rendering-v1`, `wasm-lifecycle-v1`, `wasm-panels-v1` — schema-valid, rubric-scored, ready for codegen. No browser opened yet.

```bash
make demo
```
**Nine native binaries, one make target.** Canvas → App → Elm counter → events → components → charts → bundle → vdom-snapshot → dash. Each binary is one Rust crate from m1–m5 running through the same `presentar-core` widgets the browser will paint.

```bash
make wasm
```
**Every crate compiles for `wasm32-unknown-unknown`.** Same source, new target triple. No conditional compilation, no `cfg(target_arch)` carve-outs — `presentar` is renderer-agnostic by design.

```bash
cargo test --workspace --test probar_snapshot
```
**Probar snapshots — no headless browser.** VDOM trees stringified to deterministic JSON, diffed against committed golden files. CI runs in 200 ms, not 30 seconds.

```bash
make lean-build
```
**Lean 4 proves the obligations at L5.** Type-checked in ~1 s. `Build completed successfully` is the proof certificate that ships to the contract's `proof_status` field.

## Close

Five weeks. Three pillars. Zero hand-written JavaScript. Every browser-side artifact is a compile target — `wasm-bindgen` emits the loader, `presentar-cli` serves the bundle, probar replaces the headless browser.

## Speaking notes

- Pace ~140 wpm. Let the green "✓" between `make demo` stages breathe.
- Pronunciation: presentar = "pre-sen-TAR" (Spanish, "to present"); probar = "pro-BAR" (Spanish, "to prove"); aprender = "ah-pren-DER"; wasm-bindgen = "wasm bind-gen"; BLAKE3 = "blake three"; Kani = "KAH-nee".
- The browser surface (`presentar-cli serve` → live canvas at localhost:3000) is the M5 capstone screencast (5.2.1), not this script. This script proves the **native + WASM** half compiles and the contracts validate.
- Don't `make ci` on stream — coverage takes 2 min. The bullets above are the demo path; `make ci` is the merge-gate path.
- Land the close on the green `make lean-build` output. No extra terminal output after it.

## Quickstart (one command)

```bash
make demo
```
Runs the nine native binaries in sequence — Canvas2DRenderer paints to stdout, the Elm counter folds Msgs, the components panel renders, the dash capstone composes all four pillars into one VDOM. Each `=== Mx.y name ===` header marks the crate; each binary completes in <1 s.
