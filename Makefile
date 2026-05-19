.DELETE_ON_ERROR:
.ONESHELL:
.SUFFIXES:

.PHONY: help install validate explain score lint audit status graph codegen \
        kani-stubs lean-stubs probar-stubs invariants scaffold generate \
        proof-status coverage demo test build fmt fmt-check clippy \
        coverage-test ci clean lean-build lean-clean wasm

PV ?= pv
CONTRACTS := contracts/wasm-rendering-v1.yaml \
             contracts/wasm-lifecycle-v1.yaml \
             contracts/wasm-panels-v1.yaml

help:
	@echo "wasm-from-zero — pv-gated Makefile (companion repo for the wasm-from-zero course)"
	@echo ""
	@echo "Every learner command goes THROUGH pv. cargo runs the demos,"
	@echo "but the contracts are the source of truth — pv validates them,"
	@echo "scores them, and emits the runtime assertions the demo binaries"
	@echo "assert against. Lean 4 proves the invariants at L5."
	@echo ""
	@echo "  Install:"
	@echo "    make install       — cargo install aprender-contracts-cli (provides pv)"
	@echo ""
	@echo "  pv contract gates:"
	@echo "    make validate      — pv validate per contract  (schema gate)"
	@echo "    make score         — pv score per contract     (5-dim rubric)"
	@echo "    make lint          — pv lint per contract"
	@echo ""
	@echo "  Build + demo:"
	@echo "    make build         — cargo build --workspace --release"
	@echo "    make demo          — run every demo binary on native target"
	@echo "    make test          — cargo test --workspace --release"
	@echo "    make wasm          — cargo build --target wasm32-unknown-unknown"
	@echo ""
	@echo "  Browser demo (real WASM in a real browser):"
	@echo "    make wasm-bundle   — cargo build wasm32 + wasm-bindgen → m5-dash/web/pkg/"
	@echo "    make serve         — wasm-bundle + python3 -m http.server (PORT=3000)"
	@echo ""
	@echo "  Lean 4:"
	@echo "    make lean-build    — type-check every theorem (L5 proof)"
	@echo ""
	@echo "  Quality gates:"
	@echo "    make ci            — fmt + clippy + test + coverage + lint"

install:
	@if command -v $(PV) >/dev/null 2>&1; then \
		echo "[install] pv already on PATH ($$($(PV) --version 2>&1 | head -1))"; \
	else \
		cargo install aprender-contracts-cli || exit 1; \
	fi

validate:
	@for c in $(CONTRACTS); do echo "--- pv validate $$c ---"; $(PV) validate $$c; done

explain:
	@for c in $(CONTRACTS); do echo "--- pv explain $$c ---"; $(PV) explain $$c; done

score:
	@for c in $(CONTRACTS); do echo "--- pv score $$c ---"; $(PV) score $$c; done

lint:
	@for c in $(CONTRACTS); do echo "--- pv lint $$c ---"; $(PV) lint $$c; done

audit:
	@for c in $(CONTRACTS); do echo "--- pv audit $$c ---"; $(PV) audit $$c; done

status:
	@for c in $(CONTRACTS); do echo "--- pv status $$c ---"; $(PV) status $$c; done

graph:
	@for c in $(CONTRACTS); do echo "--- pv graph $$c ---"; $(PV) graph $$c; done

codegen:
	@mkdir -p target/pv
	@$(PV) codegen contracts/ --output target/pv/all-assertions.rs

proof-status:
	$(PV) proof-status contracts/

coverage:
	$(PV) coverage contracts/

build:
	cargo build --workspace --release

test:
	PROPTEST_CASES=256 cargo test --workspace --release

wasm:
	@for crate in m1-canvas m1-app m2-elm-wasm m2-events m3-components m3-charts m4-bundle m4-tests m5-dash; do \
		echo "--- compile $$crate to wasm32 ---"; \
		cargo build --release -p $$crate --target wasm32-unknown-unknown; \
	done

demo:
	@echo "=== M1.1 canvas ==="
	@cargo run --release --bin canvas-demo
	@echo "=== M1.2 app ==="
	@cargo run --release --bin app-demo
	@echo "=== M2.1 counter ==="
	@cargo run --release --bin counter-demo
	@echo "=== M2.2 events ==="
	@cargo run --release --bin events-demo
	@echo "=== M3.1 components ==="
	@cargo run --release --bin components-demo
	@echo "=== M3.2 charts ==="
	@cargo run --release --bin charts-demo
	@echo "=== M4.1 bundle ==="
	@cargo run --release --bin bundle-demo
	@echo "=== M4.2 vdom snapshot ==="
	@cargo run --release --bin tests-demo
	@echo "=== M5 capstone dash ==="
	@cargo run --release --bin dash

# ---- Browser demo (real WASM in a real browser, no JS) ----
# 1. cargo build --target wasm32-unknown-unknown -p m5-dash
# 2. wasm-bindgen → m5-dash/web/pkg/{m5_dash.js, m5_dash_bg.wasm}
# 3. python3 -m http.server → http://127.0.0.1:3000/
# Open the URL in any browser to see m5-dash paint to a real <canvas>.
WASM_BG := /mnt/nvme-raid0/targets/wasm-from-zero/wasm32-unknown-unknown/release/m5_dash.wasm
PORT ?= 3000

wasm-bundle:
	@command -v wasm-bindgen >/dev/null 2>&1 || { \
		echo "wasm-bindgen not on PATH — run: cargo install wasm-bindgen-cli --version 0.2.121"; exit 1; }
	cargo build --release --target wasm32-unknown-unknown -p m5-dash
	@WASM=$$(find target -name 'm5_dash.wasm' -path '*release*' -not -path '*deps*' 2>/dev/null | head -1); \
	[ -z "$$WASM" ] && WASM="$(WASM_BG)"; \
	wasm-bindgen "$$WASM" --out-dir m5-dash/web/pkg --target web --no-typescript
	@echo "bundle ready: m5-dash/web/pkg/{m5_dash.js, m5_dash_bg.wasm}"

serve: wasm-bundle
	@echo "Serving m5-dash dashboard at http://127.0.0.1:$(PORT)/"
	@echo "Press Ctrl-C to stop."
	@cd m5-dash/web && python3 -m http.server $(PORT)

lean-build:
	cd lean && lake build

lean-clean:
	cd lean && lake clean

ci: fmt-check clippy test coverage-test lint

fmt:
	cargo fmt --all

fmt-check:
	cargo fmt --all -- --check

clippy:
	cargo clippy --workspace --all-targets -- -D warnings

coverage-test:
	cargo llvm-cov --workspace --release --ignore-filename-regex 'main\.rs|src/bin/' --fail-under-lines 95

clean:
	cargo clean || exit 1
	rm -rf target/pv || exit 1
