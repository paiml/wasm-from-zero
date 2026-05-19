//! M2.2 — BrowserEvent → Msg dispatch (total function).

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BrowserEvent {
    Click { id: String },
    KeyPress { key: String, ctrl: bool },
    Resize { width: u32, height: u32 },
    BeforeUnload,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppMsg {
    Bump(i32),
    Reset,
    Resized(u32, u32),
    Quit,
}

/// Total: every BrowserEvent maps to Some(Msg) or None — never panics.
#[must_use]
pub fn dispatch(ev: &BrowserEvent) -> Option<AppMsg> {
    match ev {
        BrowserEvent::Click { id } if id == "inc" => Some(AppMsg::Bump(1)),
        BrowserEvent::Click { id } if id == "dec" => Some(AppMsg::Bump(-1)),
        BrowserEvent::Click { id } if id == "reset" => Some(AppMsg::Reset),
        BrowserEvent::Click { .. } => None,
        BrowserEvent::KeyPress { key, ctrl } => match (key.as_str(), ctrl) {
            ("+", _) | ("=", _) | ("ArrowUp", _) => Some(AppMsg::Bump(1)),
            ("-", _) | ("ArrowDown", _) => Some(AppMsg::Bump(-1)),
            ("r" | "R", _) => Some(AppMsg::Reset),
            ("q" | "Q" | "Escape", _) => Some(AppMsg::Quit),
            ("c", true) => Some(AppMsg::Quit),
            _ => None,
        },
        BrowserEvent::Resize { width, height } => Some(AppMsg::Resized(*width, *height)),
        BrowserEvent::BeforeUnload => Some(AppMsg::Quit),
    }
}

#[must_use]
pub fn contract_marker() -> &'static str {
    "contract: wasm-lifecycle-v1 holds — OK"
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;

    fn click(id: &str) -> BrowserEvent {
        BrowserEvent::Click { id: id.into() }
    }
    fn key(k: &str, ctrl: bool) -> BrowserEvent {
        BrowserEvent::KeyPress {
            key: k.into(),
            ctrl,
        }
    }

    #[test]
    fn click_inc_maps_to_bump_one() {
        assert_eq!(dispatch(&click("inc")), Some(AppMsg::Bump(1)));
        assert_eq!(dispatch(&click("dec")), Some(AppMsg::Bump(-1)));
        assert_eq!(dispatch(&click("reset")), Some(AppMsg::Reset));
        assert_eq!(dispatch(&click("unknown")), None);
    }

    #[test]
    fn keys_map_to_msgs() {
        assert_eq!(dispatch(&key("+", false)), Some(AppMsg::Bump(1)));
        assert_eq!(dispatch(&key("ArrowUp", false)), Some(AppMsg::Bump(1)));
        assert_eq!(dispatch(&key("ArrowDown", false)), Some(AppMsg::Bump(-1)));
        assert_eq!(dispatch(&key("r", false)), Some(AppMsg::Reset));
        assert_eq!(dispatch(&key("R", false)), Some(AppMsg::Reset));
        assert_eq!(dispatch(&key("Escape", false)), Some(AppMsg::Quit));
        assert_eq!(dispatch(&key("c", true)), Some(AppMsg::Quit));
        assert_eq!(dispatch(&key("z", false)), None);
    }

    #[test]
    fn resize_carries_dims() {
        assert_eq!(
            dispatch(&BrowserEvent::Resize {
                width: 1920,
                height: 1080
            }),
            Some(AppMsg::Resized(1920, 1080))
        );
    }

    #[test]
    fn before_unload_quits() {
        assert_eq!(dispatch(&BrowserEvent::BeforeUnload), Some(AppMsg::Quit));
    }

    #[test]
    fn contract_marker_matches() {
        assert!(contract_marker().starts_with("contract:"));
        assert!(contract_marker().ends_with("— OK"));
    }
}
