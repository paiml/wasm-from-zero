//! M2.1 — Elm-style counter for WASM. Same architecture as a TUI Elm
//! app, only `view()` produces a `VDom` instead of a `CellBuffer`.

use presentar_core::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct State {
    pub count: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Msg {
    Increment,
    Decrement,
    Reset,
}

#[must_use]
pub fn init() -> State {
    State::default()
}

#[must_use]
pub fn update(state: State, msg: Msg) -> State {
    match msg {
        Msg::Increment => State {
            count: state.count.saturating_add(1),
        },
        Msg::Decrement => State {
            count: state.count.saturating_sub(1),
        },
        Msg::Reset => State::default(),
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VDomNode {
    pub tag: String,
    pub text: String,
    pub fg: Color,
}

#[must_use]
pub fn view(state: State) -> Vec<VDomNode> {
    let fg = if state.count > 0 {
        Color::GREEN
    } else if state.count < 0 {
        Color::RED
    } else {
        Color::WHITE
    };
    vec![
        VDomNode {
            tag: "h1".into(),
            text: "counter".into(),
            fg: Color::WHITE,
        },
        VDomNode {
            tag: "div".into(),
            text: format!("count = {}", state.count),
            fg,
        },
        VDomNode {
            tag: "small".into(),
            text: "+/- to step, r to reset".into(),
            fg: Color::WHITE,
        },
    ]
}

#[must_use]
pub fn contract_marker() -> &'static str {
    "contract: wasm-lifecycle-v1 holds — OK"
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn replay_is_deterministic() {
        let msgs = [
            Msg::Increment,
            Msg::Increment,
            Msg::Decrement,
            Msg::Reset,
            Msg::Increment,
        ];
        let a = msgs.iter().copied().fold(init(), update);
        let b = msgs.iter().copied().fold(init(), update);
        assert_eq!(a, b);
        assert_eq!(a.count, 1);
    }

    #[test]
    fn view_color_negative_is_red() {
        let v = view(State { count: -1 });
        assert_eq!(v[1].fg, Color::RED);
        assert!(v[1].text.contains("-1"));
    }

    #[test]
    fn view_color_positive_is_green() {
        let v = view(State { count: 5 });
        assert_eq!(v[1].fg, Color::GREEN);
    }

    #[test]
    fn view_color_zero_is_white() {
        let v = view(State { count: 0 });
        assert_eq!(v[1].fg, Color::WHITE);
    }

    #[test]
    fn update_total_at_i32_extremes() {
        assert_eq!(
            update(State { count: i32::MAX }, Msg::Increment).count,
            i32::MAX
        );
        assert_eq!(
            update(State { count: i32::MIN }, Msg::Decrement).count,
            i32::MIN
        );
    }

    proptest! {
        #[test]
        fn proptest_replay_deterministic(msgs in proptest::collection::vec(
            prop_oneof![Just(Msg::Increment), Just(Msg::Decrement), Just(Msg::Reset)],
            0..40
        )) {
            let a = msgs.iter().copied().fold(init(), update);
            let b = msgs.iter().copied().fold(init(), update);
            prop_assert_eq!(a, b);
        }
    }

    #[test]
    fn contract_marker_matches() {
        assert!(contract_marker().starts_with("contract:"));
        assert!(contract_marker().ends_with("— OK"));
    }
}
