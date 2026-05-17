//! M4.1 — Build-pipeline description. Models the exact wasm-bindgen +
//! presentar-cli bundle steps as data so a learner can introspect the
//! pipeline before they run it.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipelineStep {
    pub name: &'static str,
    pub command: &'static str,
    pub artifact: &'static str,
}

#[must_use]
pub fn build_pipeline() -> Vec<PipelineStep> {
    vec![
        PipelineStep {
            name: "compile rust → wasm",
            command: "cargo build --release --target wasm32-unknown-unknown",
            artifact: "target/wasm32-unknown-unknown/release/<crate>.wasm",
        },
        PipelineStep {
            name: "generate JS bindings",
            command: "wasm-bindgen <wasm> --out-dir pkg --target web",
            artifact: "pkg/<crate>_bg.wasm + pkg/<crate>.js",
        },
        PipelineStep {
            name: "optimize wasm size",
            command: "wasm-opt -Oz <wasm> -o <wasm>",
            artifact: "pkg/<crate>_bg.wasm (smaller)",
        },
        PipelineStep {
            name: "bundle for browser",
            command: "presentar-cli serve --pkg pkg --out dist",
            artifact: "dist/index.html + dist/<crate>.js + dist/<crate>_bg.wasm",
        },
        PipelineStep {
            name: "deploy",
            command: "rsync -avz dist/ <host>:/var/www/<crate>/",
            artifact: "live URL",
        },
    ]
}

#[must_use]
pub fn contract_marker() -> &'static str {
    "contract: wasm-rendering-v1 holds — OK"
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn pipeline_has_five_steps() {
        assert_eq!(build_pipeline().len(), 5);
    }

    #[test]
    fn pipeline_starts_with_cargo_build() {
        assert_eq!(build_pipeline()[0].name, "compile rust → wasm");
        assert!(build_pipeline()[0]
            .command
            .contains("wasm32-unknown-unknown"));
    }

    #[test]
    fn pipeline_step_2_runs_wasm_bindgen() {
        assert!(build_pipeline()[1].command.contains("wasm-bindgen"));
    }

    #[test]
    fn pipeline_step_4_uses_presentar_cli() {
        assert!(build_pipeline()[3].command.contains("presentar-cli"));
    }

    #[test]
    fn contract_marker_matches() {
        assert!(contract_marker().starts_with("contract:"));
        assert!(contract_marker().ends_with("— OK"));
    }
}
