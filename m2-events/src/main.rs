#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

use m2_events::{contract_marker, dispatch, BrowserEvent};

fn main() {
    let probes = [
        BrowserEvent::Click { id: "inc".into() },
        BrowserEvent::Click { id: "dec".into() },
        BrowserEvent::Click { id: "reset".into() },
        BrowserEvent::Click { id: "noop".into() },
        BrowserEvent::KeyPress {
            key: "+".into(),
            ctrl: false,
        },
        BrowserEvent::KeyPress {
            key: "ArrowUp".into(),
            ctrl: false,
        },
        BrowserEvent::KeyPress {
            key: "Escape".into(),
            ctrl: false,
        },
        BrowserEvent::KeyPress {
            key: "c".into(),
            ctrl: true,
        },
        BrowserEvent::KeyPress {
            key: "z".into(),
            ctrl: false,
        },
        BrowserEvent::Resize {
            width: 1280,
            height: 720,
        },
        BrowserEvent::BeforeUnload,
    ];
    println!("M2.2 · BrowserEvent → AppMsg dispatch (totality)\n");
    for ev in &probes {
        println!("  {ev:?}\n     → {:?}\n", dispatch(ev));
    }
    println!(
        "{} probes dispatched, 0 panics — totality contract holds.",
        probes.len()
    );
    eprintln!("{}", contract_marker());
}
