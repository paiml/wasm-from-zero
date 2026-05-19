# wasm-from-zero

<p align="center">
  <img src="assets/hero.png" alt="wasm-from-zero — six in-browser WASM demos painted from compiled Rust: Canvas2D clip, Elm counter, process table, showcase animation, capstone dashboard, and a real aprender-shell Markov autocomplete. No hand-written JavaScript." width="100%"/>
</p>

Six Rust → WebAssembly demos. Zero hand-written JavaScript. Every `<canvas>` pixel
comes from compiled Rust via [`presentar`](https://crates.io/crates/aprender-present-lib).

```bash
make serve              # → http://127.0.0.1:3000
```

| # | Demo | What it shows |
|---|------|---------------|
| 1 | [canvas](wasm-demos/web/canvas/) | Canvas2D clipping survives NaN, Inf, negative, oversize coords |
| 2 | [counter](wasm-demos/web/counter/) | Elm-style `init` / `update` / `view`, click or keypress |
| 3 | [process-table](wasm-demos/web/process-table/) | Sortable process table + 4 Hz sparkline |
| 4 | [showcase](wasm-demos/web/showcase/) | 60 Hz animated bar chart, donut, particles, softmax |
| 5 | [wasm-dash](wasm-demos/web/wasm-dash/) | The `m5-dash` capstone, jittered at 30 Hz |
| 6 | [shell](wasm-demos/web/shell/) | **Real** `aprender-shell` 3-gram Markov autocomplete |

## Where the ML model lives

Demo 6 ships the real trained `aprender-shell-base.apr` model — 3-gram Markov,
~380 commands, ~9 KB zstd-compressed — byte-for-byte identical to what
[`interactive.paiml.com/shell-ml`](https://interactive.paiml.com/shell-ml) serves
(SHA-256 `068ac67a89693d2773adc4b850aca5dbb65102653dd27239c960b42e5a7e3974`).

- **Model file**: [`wasm-demos/assets/aprender-shell-base.apr`](wasm-demos/assets/aprender-shell-base.apr) — embedded into the wasm binary via `include_bytes!`.
- **Loader**: [`wasm-demos/src/shell_model.rs`](wasm-demos/src/shell_model.rs) — `ShellAutocomplete::load_from_bytes` parses the 32-byte APR header, decompresses the payload with pure-Rust [`ruzstd`](https://crates.io/crates/ruzstd), bincode-deserializes the n-gram tables, and builds a prefix trie. Vendored from `presentar::browser::shell_autocomplete` because the published `aprender-present-lib 0.34` has a broken `include_bytes!` path in an unrelated module that fails wasm builds (the same workaround `m5-dash` documents).
- **Integration**: [`wasm-demos/src/logic.rs`](wasm-demos/src/logic.rs) — `shell::lookup(prefix)` returns the top-3 completions on every keystroke, painted to the canvas by [`shell_demo.rs`](wasm-demos/src/shell_demo.rs).

## Quick start

```bash
git clone https://github.com/paiml/wasm-from-zero
cd wasm-from-zero
make install                                  # cargo install aprender-contracts-cli
rustup target add wasm32-unknown-unknown
make serve                                    # browser gallery
make demo                                     # all 9 demos on native
make ci                                       # fmt + clippy + test + 95% coverage + pv lint
```

## Layout

- `m1-canvas/`, `m1-app/`, `m2-elm-wasm/`, `m2-events/`, `m3-components/`, `m3-charts/`, `m4-bundle/`, `m4-tests/`, `m5-dash/` — the nine module crates the course builds in order.
- `wasm-demos/` — the six-demo browser gallery. Paint logic in `src/logic.rs`, mount wrappers in `src/<name>_demo.rs`, HTML harness in `web/<name>/`, 60 native probar tests in `tests/`.
- `contracts/` — three YAML contracts (`wasm-rendering-v1`, `wasm-lifecycle-v1`, `wasm-panels-v1`) gated by `pv`.
- `lean/` — Lean 4 theorems discharging the universal claim at L5.

## License

Dual-licensed `MIT OR Apache-2.0`.
