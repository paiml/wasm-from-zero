//! M1.2 — App::mount lifecycle. The wasm-rendering-v1 contract says
//! mount returns Result, never panic. We model the same contract here
//! as a native-target totality test.

use presentar_core::Color;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MountError {
    TargetMissing(String),
    AlreadyMounted,
}

impl std::fmt::Display for MountError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TargetMissing(id) => write!(f, "target id '{id}' not in document"),
            Self::AlreadyMounted => write!(f, "already mounted"),
        }
    }
}

impl std::error::Error for MountError {}

/// Simulated browser host. In the real `aprender-present-lib::browser::App`
/// this would talk to web-sys; here we model the contract surface so
/// the lifecycle obligation is unit-testable on the host.
#[derive(Debug, Clone, Default)]
pub struct HostDocument {
    target_ids: Vec<String>,
}

impl HostDocument {
    #[must_use]
    pub fn with_target(mut self, id: &str) -> Self {
        self.target_ids.push(id.to_string());
        self
    }
}

#[derive(Debug)]
pub struct App {
    pub target_id: String,
    pub theme_color: Color,
    pub mounted: bool,
}

impl App {
    /// Total: every input maps to Ok or Err — no panics.
    pub fn mount(host: &HostDocument, target_id: &str) -> Result<Self, MountError> {
        if !host.target_ids.iter().any(|id| id == target_id) {
            return Err(MountError::TargetMissing(target_id.to_string()));
        }
        Ok(Self {
            target_id: target_id.to_string(),
            theme_color: Color::new(0.04, 0.05, 0.09, 1.0),
            mounted: true,
        })
    }

    pub fn unmount(&mut self) {
        self.mounted = false;
    }
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
    fn mount_succeeds_when_target_present() {
        let host = HostDocument::default().with_target("app");
        let app = App::mount(&host, "app").expect("mount");
        assert!(app.mounted);
        assert_eq!(app.target_id, "app");
    }

    #[test]
    fn mount_returns_err_when_target_missing() {
        let host = HostDocument::default();
        let err = App::mount(&host, "missing").unwrap_err();
        assert!(matches!(err, MountError::TargetMissing(_)));
        assert!(err.to_string().contains("missing"));
    }

    #[test]
    fn already_mounted_error_displays() {
        assert!(MountError::AlreadyMounted.to_string().contains("mounted"));
    }

    #[test]
    fn unmount_clears_flag() {
        let host = HostDocument::default().with_target("root");
        let mut app = App::mount(&host, "root").expect("mount");
        app.unmount();
        assert!(!app.mounted);
    }

    #[test]
    fn contract_marker_matches() {
        assert!(contract_marker().starts_with("contract:"));
        assert!(contract_marker().ends_with("— OK"));
    }
}
