# wasm-from-zero

<p align="center">
  <img src="assets/hero.png" alt="wasm-from-zero — Canvas2D · Elm · Compose on aprender presentar, tested with probar. Six runnable browser demos via `make serve`." width="100%"/>
</p>

[![CI](https://github.com/paiml/wasm-from-zero/actions/workflows/ci.yml/badge.svg)](https://github.com/paiml/wasm-from-zero/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![Version](https://img.shields.io/badge/version-0.1.0-blue.svg)](Cargo.toml)
[![Coverage](https://img.shields.io/badge/coverage-95%25%20gate-brightgreen.svg)](#quality-gates)
[![pv contracts](https://img.shields.io/badge/pv%20contracts-3%20valid-brightgreen.svg)](contracts/)
[![WASM target](https://img.shields.io/badge/target-wasm32--unknown--unknown-blueviolet.svg)](#install)

Companion repository for the **WASM from Zero** Coursera course — a planned addition to the
[Rust for Data Engineering](https://www.coursera.org/specializations/rust-for-data-engineering)
specialization (slot TBD). Sister to
[`paiml/tui-from-zero`](https://github.com/paiml/tui-from-zero) — same c7 pattern, same
contracts shape, retargeted from the terminal to the browser.

The course teaches the **WASM-first** half of `presentar` — the pure-Rust UI framework from
[`paiml/aprender`](https://github.com/paiml/aprender). Where tui-from-zero leans on
[`aprender-present-terminal`](https://crates.io/crates/aprender-present-terminal), this repo
leans on [`aprender-present-lib`](https://crates.io/crates/aprender-present-lib) — the same
`presentar-core` widgets and Color type, surfaced through a `browser::App` and
`Canvas2DRenderer` that target `wasm32-unknown-unknown`.

Probar (`aprender-present-test`) provides the same snapshot-testing pattern as before, now
producing **VDOM snapshots** instead of cell-buffer snapshots.

## Browser demo gallery

```bash
make serve              # → http://127.0.0.1:3000
```

Six interactive WASM apps painted from compiled Rust, **zero hand-written JavaScript** —
`wasm-bindgen` emits the ES-module loader, `presentar-core` provides the widget vocabulary,
every `<canvas>` pixel comes from one `paint_ops(state) -> Vec<DrawOp>` per demo.

| # | Path | What it demos | Crate(s) |
|---|---|---|---|
| 1 | [`web/canvas/`](wasm-demos/web/canvas/) | `m1-canvas::clip` survives 8 adversarial inputs (NaN, Inf, negative, oversize) | m1-canvas |
| 2 | [`web/counter/`](wasm-demos/web/counter/) | Elm `init / update / view` — click `[+]` `[−]` `[reset]` or press `+`/`=`/`-`/`r` on the keyboard; state mutates in Rust | m2-elm-wasm |
| 3 | [`web/process-table/`](wasm-demos/web/process-table/) | Sortable process table — click a column header (case-insensitive `COMMAND`); click again to reverse direction; live sparkline ticks 4 Hz | m3-components, m3-charts |
| 4 | [`web/showcase/`](wasm-demos/web/showcase/) | 60Hz animated: bar chart + rotating donut + particles + softmax | (synthetic) |
| 5 | [`web/wasm-dash/`](wasm-demos/web/wasm-dash/) | The m5-dash capstone, jittered at 30Hz; same `build_paint_list(Dashboard::fixture())` the native demo prints | m5-dash |
| 6 | [`web/shell/`](wasm-demos/web/shell/) | Markov-style shell autocomplete (pure-presentar port of [interactive.paiml.com/shell-ml](https://interactive.paiml.com/shell-ml)) | (embedded model) |

Each demo's pure paint logic lives in `wasm-demos::logic::<demo>::paint_ops()` and is
exercised by **60 native probar tests** — including 32 bisection-proof regression tests
pinning every paint-layer bug ever caught by hand-verification (four full rounds of
external QA, every flagged issue verified fixed or confirmed not-a-bug — see commit
history for the round-by-round trail).

## The three pillars

| Pillar | Contract | What it gates | Demo crates that prove it at runtime |
|---|---|---|---|
| **Canvas** | [`wasm-rendering-v1`](contracts/wasm-rendering-v1.yaml) | `Canvas2DRenderer` bounds clamping · NaN-safe coords · Color clamping to `[0,1]` · `App::mount` returns Result not panic · JsValue round-trip | [`m1-canvas`](m1-canvas/), [`m1-app`](m1-app/), [`m4-bundle`](m4-bundle/), [`m4-tests`](m4-tests/) |
| **Elm** | [`wasm-lifecycle-v1`](contracts/wasm-lifecycle-v1.yaml) | `update` totality · `view` referential transparency · event-replay determinism · `requestAnimationFrame` at-most-once per frame · unmount drops pending callbacks | [`m2-elm-wasm`](m2-elm-wasm/), [`m2-events`](m2-events/) |
| **Compose** | [`wasm-panels-v1`](contracts/wasm-panels-v1.yaml) | Component layout non-overlap · monotonic chart cursor · cursor clamped to chart bounds · `BrowserRouter::match` total | [`m3-components`](m3-components/), [`m3-charts`](m3-charts/), [`m5-dash`](m5-dash/) |

Each contract declares a formula, domain, invariants, proof obligations, falsification tests,
and a Kani harness stub. `pv validate` enforces the schema; `pv score` grades each contract
across five dimensions (Spec / Falsify / Kani / Lean / Bind); Lean 4 discharges the
universal claim at L5 (see [`lean/WasmFromZero/Theorems/`](lean/WasmFromZero/Theorems/));
runtime markers in each demo binary assert the obligation at exit.

## Demos

Nine workspace crates, one demo binary each. Each binary prints `contract: <name> — OK` to
stderr at exit; the [CI workflow](.github/workflows/ci.yml) greps for that marker as the
runtime half of the proof.

| Crate | What it teaches | Gating contract | Demo binary |
|---|---|---|---|
| [`m1-canvas`](m1-canvas/) | Canvas2D draw plans — bounds clip, NaN-safe coords, color clamp (modelled on `presentar_lib::browser::Canvas2DRenderer`) | `wasm-rendering-v1` | `canvas-demo` |
| [`m1-app`](m1-app/) | `App::mount(target_id)` lifecycle — Result-returning, never panics on missing target | `wasm-rendering-v1` | `app-demo` |
| [`m2-elm-wasm`](m2-elm-wasm/) | Elm-style counter for WASM (init/update/view → VDom) with proptest replay determinism | `wasm-lifecycle-v1` | `counter-demo` |
| [`m2-events`](m2-events/) | Total `dispatch(BrowserEvent) -> Option<AppMsg>` covering Click, KeyPress, Resize, BeforeUnload | `wasm-lifecycle-v1` | `events-demo` |
| [`m3-components`](m3-components/) | Container/Row/Column layout into pixel rects (non-overlap, deterministic) | `wasm-panels-v1` | `components-demo` |
| [`m3-charts`](m3-charts/) | Gauge fill + line-chart cursor math (monotonic, NaN-safe, clamped) | `wasm-panels-v1` | `charts-demo` |
| [`m4-bundle`](m4-bundle/) | The 5-step WASM build pipeline: `cargo build → wasm-bindgen → wasm-opt → presentar-cli → deploy` | `wasm-rendering-v1` | `bundle-demo` |
| [`m4-tests`](m4-tests/) | VDOM snapshot harness — `snapshot()` + `diff_snapshot()` (probar pattern, no headless browser needed) | `wasm-rendering-v1` | `tests-demo` |
| [`m5-dash`](m5-dash/) | Capstone — composes every prior module into a small in-browser dashboard model | `wasm-panels-v1` | `dash` |

`make demo` runs all nine on native target. `make wasm` builds every crate for
`wasm32-unknown-unknown` (validates the WASM target compiles end-to-end).

## Stack

| Layer | Where it comes from | Version |
|---|---|---|
| `Color`, widget primitives | [`aprender-present-core`](https://crates.io/crates/aprender-present-core) | 0.33 |
| Browser backend (`App`, `Canvas2DRenderer`, `BrowserRouter`) | [`aprender-present-lib`](https://crates.io/crates/aprender-present-lib) | 0.31 |
| Snapshot testing (probar) | [`aprender-present-test`](https://crates.io/crates/aprender-present-test) as dev-dep | 0.31 |
| Build pipeline | [`aprender-present-cli`](https://crates.io/crates/aprender-present-cli) (`presentar-cli serve|bundle`) | 0.31 |
| Contract validator | [`aprender-contracts-cli`](https://crates.io/crates/aprender-contracts-cli) (the `pv` binary) | latest |
| Lean 4 proofs | `lake build` over [`lean/WasmFromZero/Theorems/`](lean/WasmFromZero/Theorems/) | v4.13.0 |
| Property tests | `proptest` | 1.x |

Every dep resolves from crates.io. No git deps, no path deps to a sibling checkout.

## Install

```bash
make install                                  # cargo install aprender-contracts-cli
rustup target add wasm32-unknown-unknown     # needed for `make wasm`
```

### Prerequisites

- Rust 1.75+
- `wasm32-unknown-unknown` target (`rustup target add wasm32-unknown-unknown`)
- Optional: [`wasm-bindgen-cli`](https://crates.io/crates/wasm-bindgen-cli) for browser bundles
- Optional: `elan` + Lean 4 v4.13 for `make lean-build`
- Optional: [`cargo-llvm-cov`](https://crates.io/crates/cargo-llvm-cov) for the coverage gate

## Quick start

```bash
git clone https://github.com/paiml/wasm-from-zero
cd wasm-from-zero
make install
rustup target add wasm32-unknown-unknown

# Gate the contracts (schema + rubric)
make validate
make score

# Build + run every demo on native (no browser needed)
make build
make demo

# OR: build the WASM bundle + serve the six-demo gallery in a browser
make serve              # → http://127.0.0.1:3000

# Verify everything compiles for wasm32 too
make wasm

# Type-check the Lean 4 proofs (~1 s)
make lean-build

# Probar snapshot tests — 41 demo-layer tests + the M1-M5 invariant tests
cargo test --workspace

# Full pre-merge gate
make ci
```

## Quality gates

`make ci` runs locally; the [CI workflow](.github/workflows/ci.yml) runs the same gates on
every push plus a `cargo build --target wasm32-unknown-unknown` and the per-binary runtime
contract assertions.

| Gate | Tool | Threshold |
|---|---|---|
| Formatting | `cargo fmt --all --check` | clean |
| Build (native) | `cargo build --workspace --locked` | clean |
| Build (wasm32) | `cargo build --workspace --locked --target wasm32-unknown-unknown` | clean |
| Tests | `cargo test --workspace --locked` | **110+ tests** (50 M1–M5 + 60 demo-gallery — 28 invariant + 32 bug-bisecting regressions) |
| Line coverage | `cargo llvm-cov --workspace --fail-under-lines 95` | 95% (local achieves 100%) |
| Clippy | `cargo clippy --workspace --all-targets -- -D warnings` | clean |
| Contract validation | `pv validate contracts/<name>.yaml` × 3 | 0 errors per contract |
| Runtime contract markers | `cargo run --bin <demo>` × 9 | every binary emits its `contract: ... — OK` |
| Lean 4 proofs | `cd lean && lake build` | 5 + 5 + 5 = 15 theorems compile |
| pmat comply (advisory) | `pmat comply check` | `continue-on-error: true` |

## Repository layout

```
contracts/
  wasm-rendering-v1.yaml     Canvas2D + DOM safety obligations
  wasm-lifecycle-v1.yaml     Elm + rAF + unmount obligations
  wasm-panels-v1.yaml        Component layout + chart + router obligations
m1-canvas/                   Canvas2D clip/clamp math
m1-app/                      App::mount/unmount lifecycle
m2-elm-wasm/                 Elm counter (init/update/view → VDom)
m2-events/                   BrowserEvent → AppMsg dispatch
m3-components/               Container/Row/Column layout
m3-charts/                   gauge + line-chart cursor math
m4-bundle/                   WASM build pipeline (data-modelled)
m4-tests/                    VDOM snapshot harness (probar)
m5-dash/                     capstone — composes everything
wasm-demos/                  the six-demo browser gallery
  src/logic.rs                 pure paint_ops() per demo (DrawOp data — testable on native)
  src/<name>_demo.rs           wasm-bindgen mount_<name>(canvas_id) wrappers
  web/<name>/index.html        per-demo HTML harness (one <script type="module">)
  tests/probar_*.rs            60 native tests (28 invariant + 32 bug-bisecting regressions)
lean/                        Lean 4 lakefile + theorems per pillar
.github/workflows/ci.yml     pv-gated CI with a wasm32 build step
Makefile                     pv + cargo + lake + wasm one-button targets
assets/hero.{svg,png}        the image at the top of this README
```

## Relationship to tui-from-zero

`tui-from-zero` and `wasm-from-zero` are deliberately twins. They share:

- The same c7 design-by-provable-contracts pattern (3 named YAML contracts, Lean theorems, runtime markers, probar snapshot tests, `make ci` workflow)
- The same `presentar-core` widget vocabulary and `Color` type
- The same Elm-style architecture in M2 (just swap `CellBuffer` ↔ `VDom`)
- The same probar dev-dep + snapshot pattern

The differences are exactly what changes between targets:

- terminal `CellBuffer` ↔ browser Canvas2D + VDOM
- crossterm `KeyEvent` ↔ web-sys `Event`
- `ptop-mini` running in your terminal ↔ in-browser dashboard
- single-binary CLI ↔ `wasm-bindgen` + `presentar-cli` bundle pipeline

Two repos, one teaching narrative.

## License

Dual-licensed under MIT or Apache-2.0 — pick the one that fits your downstream use.
SPDX: `MIT OR Apache-2.0`.
