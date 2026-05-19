//! Pure logic extracted from each demo module — testable on native
//! via probar snapshots. No wasm-bindgen / web-sys here.
//!
//! The `paint_ops()` per demo returns a `Vec<DrawOp>` — the wasm
//! `paint()` wrapper iterates that vec and emits the CanvasRenderingContext2d
//! calls. Tests assert on the DrawOp list, which is what actually proves
//! a paint-layer bug fix (since the bugs lived in the demo's paint wrapper,
//! not in the underlying view()/build_paint_list()/sort_procs() that the
//! underlying crates already tested).

/// One canvas draw operation. Equality is intentional — snapshot tests
/// compare op lists for byte-identity.
#[derive(Debug, Clone, PartialEq)]
pub enum DrawOp {
    Fill(String),   // set_fill_style_str
    Stroke(String), // set_stroke_style_str
    Font(String),   // set_font
    LineWidth(f64), // set_line_width
    FillRect(f64, f64, f64, f64),
    StrokeRect(f64, f64, f64, f64),
    /// (text, x, y) — `text` is the literal string that lands on canvas.
    FillText(String, f64, f64),
}

/// Helper used by demo paint wrappers: replay a list of DrawOps onto a
/// CanvasRenderingContext2d. The pure `paint_ops()` per demo + this
/// replay are the two halves of "paint via testable data".
#[cfg(target_arch = "wasm32")]
pub fn replay(
    ctx: &web_sys::CanvasRenderingContext2d,
    ops: &[DrawOp],
) -> Result<(), wasm_bindgen::JsValue> {
    for op in ops {
        match op {
            DrawOp::Fill(s) => ctx.set_fill_style_str(s),
            DrawOp::Stroke(s) => ctx.set_stroke_style_str(s),
            DrawOp::Font(s) => ctx.set_font(s),
            DrawOp::LineWidth(w) => ctx.set_line_width(*w),
            DrawOp::FillRect(x, y, w, h) => ctx.fill_rect(*x, *y, *w, *h),
            DrawOp::StrokeRect(x, y, w, h) => ctx.stroke_rect(*x, *y, *w, *h),
            DrawOp::FillText(t, x, y) => {
                ctx.fill_text(t, *x, *y)?;
            }
        }
    }
    Ok(())
}

pub mod canvas {
    use super::DrawOp;
    use m1_canvas::{clip, CanvasDims, DrawRect};
    use presentar_core::Color;

    /// A test input + its name (so failures point at the right adversarial case).
    pub struct Plan {
        pub label: &'static str,
        pub rect: DrawRect,
    }

    pub fn adversarial_plans() -> Vec<Plan> {
        let c = Color::GREEN;
        vec![
            Plan {
                label: "clean",
                rect: DrawRect {
                    x: 60.0,
                    y: 60.0,
                    w: 200.0,
                    h: 120.0,
                    color: c,
                },
            },
            Plan {
                label: "overflow-right",
                rect: DrawRect {
                    x: 700.0,
                    y: 60.0,
                    w: 600.0,
                    h: 120.0,
                    color: c,
                },
            },
            Plan {
                label: "overflow-bottom",
                rect: DrawRect {
                    x: 60.0,
                    y: 540.0,
                    w: 200.0,
                    h: 400.0,
                    color: c,
                },
            },
            Plan {
                label: "negative-w",
                rect: DrawRect {
                    x: 320.0,
                    y: 60.0,
                    w: -50.0,
                    h: 120.0,
                    color: c,
                },
            },
            Plan {
                label: "nan-x",
                rect: DrawRect {
                    x: f64::NAN,
                    y: 60.0,
                    w: 200.0,
                    h: 120.0,
                    color: c,
                },
            },
            Plan {
                label: "inf-y",
                rect: DrawRect {
                    x: 60.0,
                    y: f64::INFINITY,
                    w: 200.0,
                    h: 120.0,
                    color: c,
                },
            },
            Plan {
                label: "off-screen",
                rect: DrawRect {
                    x: 2000.0,
                    y: 60.0,
                    w: 200.0,
                    h: 120.0,
                    color: c,
                },
            },
            Plan {
                label: "zero-w",
                rect: DrawRect {
                    x: 320.0,
                    y: 200.0,
                    w: 0.0,
                    h: 120.0,
                    color: c,
                },
            },
        ]
    }

    /// Run clip() over the 8 adversarial inputs.
    pub fn clip_results(canvas: CanvasDims) -> Vec<(&'static str, Option<DrawRect>)> {
        adversarial_plans()
            .into_iter()
            .map(|p| (p.label, clip(p.rect, canvas)))
            .collect()
    }

    /// Pure paint-op list for the canvas demo.
    ///
    /// BUG #6 history (rounds 2 + 3):
    ///   round 2 — the original demo painted survivors into a scaled
    ///             "preview band" at y=60..260 so overflow-bottom
    ///             landed at y=330 even though its text annotation
    ///             read `out=(60,540,200×180)`. Footer claimed 3 but
    ///             only 2 rectangles were visible. UNSOUND.
    ///   QA fix — round 2 replaced the preview band with a 1-pixel
    ///             inline swatch per survivor. Count matched the
    ///             footer but the demo no longer demonstrated clip
    ///             at all — it was a pass/fail legend.
    ///   round 3 — paint each surviving rect AT ITS LITERAL CLIPPED
    ///             (x, y, w, h) on the canvas (translucent green so
    ///             overlaps with labels read cleanly). Now:
    ///                * pixels at the stated `out=…` positions ARE green
    ///                * the count of green rects == survivor count == footer
    ///                * the demo actually shows what clip() does
    pub fn paint_ops(canvas: CanvasDims, w: f64, h: f64) -> Vec<DrawOp> {
        let mut ops = vec![
            DrawOp::Fill("#0d1117".into()),
            DrawOp::FillRect(0.0, 0.0, w, h),
            DrawOp::Fill("#a5d6ff".into()),
            DrawOp::Font("18px monospace".into()),
            DrawOp::FillText(
                "m1-canvas::clip — survivors painted at their literal clipped (x,y,w,h)".into(),
                20.0,
                26.0,
            ),
        ];

        // First pass — compute survivor / dropped counts and paint each
        // survivor as a TRANSLUCENT green rect at its literal clipped
        // bounds. This is the "demonstrates clip()" half of the demo.
        let mut survivors: u32 = 0;
        let mut dropped: u32 = 0;
        for plan in adversarial_plans() {
            if let Some(clipped) = clip(plan.rect, canvas) {
                survivors += 1;
                // Translucent green outline + fill — overlaps read OK.
                ops.push(DrawOp::Fill("rgba(126,231,135,0.40)".into()));
                ops.push(DrawOp::FillRect(clipped.x, clipped.y, clipped.w, clipped.h));
                ops.push(DrawOp::Stroke("rgb(126,231,135)".into()));
                ops.push(DrawOp::LineWidth(2.0));
                ops.push(DrawOp::StrokeRect(
                    clipped.x, clipped.y, clipped.w, clipped.h,
                ));
                // Label inside the rect, top-left.
                ops.push(DrawOp::Fill("#7ee787".into()));
                ops.push(DrawOp::Font("14px monospace".into()));
                ops.push(DrawOp::FillText(
                    format!(
                        "{} → ({:.0},{:.0},{:.0}×{:.0})",
                        plan.label, clipped.x, clipped.y, clipped.w, clipped.h
                    ),
                    clipped.x + 8.0,
                    clipped.y + 18.0,
                ));
            } else {
                dropped += 1;
            }
        }

        // Second pass — small top-right legend listing all 8 inputs
        // with their ✓/✗ marker so dropped ones aren't invisible.
        let legend_x = w - 360.0;
        ops.push(DrawOp::Fill("#161b22".into()));
        ops.push(DrawOp::FillRect(legend_x, 40.0, 340.0, 230.0));
        ops.push(DrawOp::Stroke("#7c3aed".into()));
        ops.push(DrawOp::LineWidth(1.0));
        ops.push(DrawOp::StrokeRect(legend_x, 40.0, 340.0, 230.0));
        ops.push(DrawOp::Fill("#c9d1d9".into()));
        ops.push(DrawOp::Font("13px monospace".into()));
        ops.push(DrawOp::FillText(
            "8 inputs — ✓ survived  ✗ dropped".into(),
            legend_x + 10.0,
            60.0,
        ));
        for (i, plan) in adversarial_plans().into_iter().enumerate() {
            let row_y = 84.0 + (i as f64) * 22.0;
            let survived = clip(plan.rect, canvas).is_some();
            ops.push(DrawOp::Fill(if survived {
                "rgb(126,231,135)".into()
            } else {
                "rgb(255,123,114)".into()
            }));
            ops.push(DrawOp::FillRect(legend_x + 10.0, row_y - 12.0, 14.0, 14.0));
            ops.push(DrawOp::Fill(
                if survived { "#7ee787" } else { "#ff7b72" }.into(),
            ));
            ops.push(DrawOp::Font("13px monospace".into()));
            ops.push(DrawOp::FillText(
                format!("{} {}", if survived { "✓" } else { "✗" }, plan.label),
                legend_x + 32.0,
                row_y,
            ));
        }

        // Footer — claim matches reality (survivor count == green rects on canvas).
        ops.push(DrawOp::Fill("#7c3aed".into()));
        ops.push(DrawOp::Font("15px monospace".into()));
        ops.push(DrawOp::FillText(
            format!(
                "survived: {survivors} (green rects on canvas) · dropped: {dropped} · \
                 contract: wasm-rendering-v1 holds — OK"
            ),
            20.0,
            h - 20.0,
        ));
        let _ = Color::GREEN; // silence unused (referenced via adversarial_plans)
        ops
    }

    /// Helper for the regression test: count green outline rects in the
    /// paint list. Each surviving clip result emits
    /// `Stroke("rgb(126,231,135)")` immediately followed by a `StrokeRect`.
    /// The legend border uses `Stroke("#7c3aed")` so it doesn't match.
    #[must_use]
    pub fn count_survivor_rects(ops: &[DrawOp]) -> usize {
        ops.windows(3)
            .filter(|w| {
                matches!(&w[0], DrawOp::Stroke(c) if c == "rgb(126,231,135)")
                    && matches!(&w[1], DrawOp::LineWidth(_))
                    && matches!(&w[2], DrawOp::StrokeRect(..))
            })
            .count()
    }
}

pub mod counter {
    use super::DrawOp;
    use m2_elm_wasm::{view, Msg, State};

    // Button hit-zones from counter_demo.rs (x, y, w, h, label).
    pub const BUTTONS: &[(f64, f64, f64, f64, &str)] = &[
        (80.0, 320.0, 120.0, 80.0, "−"),
        (340.0, 320.0, 120.0, 80.0, "+"),
        (600.0, 320.0, 200.0, 80.0, "reset"),
    ];

    pub fn hit_test(x: f64, y: f64) -> Option<Msg> {
        for (bx, by, bw, bh, label) in BUTTONS {
            if x >= *bx && x <= bx + bw && y >= *by && y <= by + bh {
                return Some(match *label {
                    "+" => Msg::Increment,
                    "−" => Msg::Decrement,
                    "reset" => Msg::Reset,
                    _ => return None,
                });
            }
        }
        None
    }

    /// BUG #7 fix (gist round 2): on-canvas hint reads `+/- to step, r
    /// to reset` but the previous demo only wired mouse clicks. This
    /// maps keyboard keys to Msg so the keydown listener in
    /// `counter_demo::mount_counter` can dispatch them.
    ///
    /// Returns `None` for keys we don't handle (including the empty
    /// "Ctrl"/"Shift"/etc. modifier-only keys, which arrive as their
    /// own `key()` value).
    pub fn key_to_msg(key: &str) -> Option<Msg> {
        match key {
            "+" | "=" => Some(Msg::Increment),
            "-" | "_" => Some(Msg::Decrement),
            "r" | "R" => Some(Msg::Reset),
            _ => None,
        }
    }

    /// Pure paint-op list for the counter demo. The wasm paint() wrapper
    /// iterates this. Tests snapshot the list — the count value MUST
    /// appear as a FillText op, or BUG #1 has regressed.
    pub fn paint_ops(state: State, w: f64, h: f64) -> Vec<DrawOp> {
        let mut ops = vec![
            // background
            DrawOp::Fill("#0d1117".into()),
            DrawOp::FillRect(0.0, 0.0, w, h),
            // title
            DrawOp::Fill("#a5d6ff".into()),
            DrawOp::Font("20px monospace".into()),
            DrawOp::FillText(
                "Elm counter — init / update / view, painted via Rust → WASM".into(),
                20.0,
                30.0,
            ),
        ];
        // VDOM dump (proves the view nodes show up)
        let nodes = view(state);
        ops.push(DrawOp::Fill("#7aa2f7".into()));
        ops.push(DrawOp::Font("13px monospace".into()));
        for (i, n) in nodes.iter().enumerate() {
            ops.push(DrawOp::FillText(
                format!("{} {}: {:?}", i, n.tag, n.text),
                60.0,
                (90 + i * 18) as f64,
            ));
        }
        // BIG count value — THIS is what bug #1 was about (was a no-op before fix).
        // Layout fix (gist round 2): previously hard-coded x = w/2 - 32 which
        // miscentred multi-char values like `-3`, `10`, `-10` because the
        // minus sign / extra digit sat to the left of the single-char center.
        // 128px monospace ≈ 64px char width — center on string length.
        let big = format!("{}", state.count);
        let big_char_w = 64.0;
        let big_x = w / 2.0 - (big.chars().count() as f64 * big_char_w) / 2.0;
        ops.push(DrawOp::Fill("#ffffff".into())); // approximate; demo derives from view().div.fg
        ops.push(DrawOp::Font("128px monospace".into()));
        ops.push(DrawOp::FillText(big, big_x, 230.0));
        // 18px monospace ≈ 11px char width — center "count" beneath the value.
        let count_label = "count";
        let count_label_w = count_label.len() as f64 * 11.0;
        ops.push(DrawOp::Fill("#565f89".into()));
        ops.push(DrawOp::Font("18px monospace".into()));
        ops.push(DrawOp::FillText(
            count_label.into(),
            w / 2.0 - count_label_w / 2.0,
            260.0,
        ));
        // Buttons. Layout fix (gist round 2): the previous demo hard-coded
        // `bx + bw/2 - 12.0` which assumed a single-char label width.
        // 5-char `reset` spilled past the right edge of its 200px button.
        // Center on actual char count — 48px monospace ≈ 28px char width.
        for (bx, by, bw, bh, label) in BUTTONS {
            ops.push(DrawOp::Fill("#161b22".into()));
            ops.push(DrawOp::FillRect(*bx, *by, *bw, *bh));
            ops.push(DrawOp::Stroke("#7c3aed".into()));
            ops.push(DrawOp::LineWidth(3.0));
            ops.push(DrawOp::StrokeRect(*bx, *by, *bw, *bh));
            ops.push(DrawOp::Fill("#c9d1d9".into()));
            ops.push(DrawOp::Font("48px monospace".into()));
            let label_char_w = 28.0;
            let label_w = label.chars().count() as f64 * label_char_w;
            ops.push(DrawOp::FillText(
                (*label).into(),
                bx + bw / 2.0 - label_w / 2.0,
                by + bh * 0.65,
            ));
        }
        // Footer
        ops.push(DrawOp::Fill("#7c3aed".into()));
        ops.push(DrawOp::Font("16px monospace".into()));
        ops.push(DrawOp::FillText(
            "contract: wasm-lifecycle-v1 holds — View_Pure, EventReplay_Deterministic".into(),
            20.0,
            h - 20.0,
        ));
        ops
    }
}

pub mod process_table {
    use std::cmp::Ordering;

    #[derive(Clone, Debug, PartialEq)]
    pub struct Proc {
        pub pid: u32,
        pub name: &'static str,
        pub cpu: f64,
        pub mem: f64,
        pub history: Vec<f64>,
    }

    #[derive(Clone, Copy, Debug, PartialEq)]
    pub enum SortKey {
        Pid,
        Cpu,
        Mem,
        /// BUG #4 fix (gist round 1): COMMAND column was a no-op.
        Name,
    }

    /// BUG #5 fix (gist round 1): repeated clicks didn't toggle direction.
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub enum SortDir {
        Asc,
        Desc,
    }

    impl SortDir {
        pub fn flip(self) -> Self {
            match self {
                Self::Asc => Self::Desc,
                Self::Desc => Self::Asc,
            }
        }
    }

    /// Default direction when a column first becomes active.
    pub fn default_dir(key: SortKey) -> SortDir {
        match key {
            SortKey::Pid | SortKey::Name => SortDir::Asc,
            SortKey::Cpu | SortKey::Mem => SortDir::Desc,
        }
    }

    /// Click reducer: same column → flip direction, different column →
    /// switch to that column's default direction.
    pub fn next_sort_state(current: (SortKey, SortDir), clicked: SortKey) -> (SortKey, SortDir) {
        if current.0 == clicked {
            (clicked, current.1.flip())
        } else {
            (clicked, default_dir(clicked))
        }
    }

    pub fn make_history(seed: u64) -> Vec<f64> {
        let mut s = seed.wrapping_mul(2654435761);
        (0..32)
            .map(|_| {
                s = s
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                ((s >> 32) as u32 as f64) / (u32::MAX as f64)
            })
            .collect()
    }

    pub fn fixture() -> Vec<Proc> {
        let names: &[&str] = &[
            "wasm-bindgen",
            "rustc",
            "cargo",
            "claude-code",
            "firefox",
            "tmux",
            "ssh-agent",
            "kernel-task",
            "Xorg",
            "pulseaudio",
            "systemd",
            "bash",
        ];
        let cpus = [
            0.34, 0.81, 0.42, 0.12, 0.61, 0.05, 0.02, 0.18, 0.27, 0.04, 0.09, 0.01,
        ];
        let mems = [
            0.21, 0.45, 0.18, 0.08, 0.72, 0.03, 0.01, 0.11, 0.16, 0.06, 0.05, 0.01,
        ];
        names
            .iter()
            .enumerate()
            .map(|(i, &n)| Proc {
                pid: 1000 + i as u32 * 137,
                name: n,
                cpu: cpus[i],
                mem: mems[i],
                history: make_history((i + 1) as u64),
            })
            .collect()
    }

    /// Sort with explicit direction.
    ///
    /// BUG #9 fix (gist round 2): SortKey::Name used raw `str::cmp`,
    /// so `Xorg` (0x58) sorted before `bash` (0x62). Real table sorts
    /// are case-insensitive — compare lower-cased.
    pub fn sort_procs_dir(procs: &mut [Proc], key: SortKey, dir: SortDir) {
        match key {
            SortKey::Pid => procs.sort_by_key(|p| p.pid),
            SortKey::Cpu => {
                procs.sort_by(|a, b| a.cpu.partial_cmp(&b.cpu).unwrap_or(Ordering::Equal))
            }
            SortKey::Mem => {
                procs.sort_by(|a, b| a.mem.partial_cmp(&b.mem).unwrap_or(Ordering::Equal))
            }
            SortKey::Name => procs.sort_by(|a, b| {
                a.name
                    .to_ascii_lowercase()
                    .cmp(&b.name.to_ascii_lowercase())
            }),
        }
        if dir == SortDir::Desc {
            procs.reverse();
        }
    }

    /// BUG #8 fix (gist round 2): the HISTORY column header reads
    /// `(32 ticks)` but the fixture's history was generated once at
    /// mount and never updated. This advances every row's history
    /// by one tick: drop the oldest sample and append a new LCG draw.
    ///
    /// Called from a `requestAnimationFrame` loop in `process_table_demo`
    /// — about once per 250ms so the sparkline actually scrolls.
    pub fn advance_history(procs: &mut [Proc], seed: &mut u64) {
        for p in procs.iter_mut() {
            *seed = seed
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let v = ((*seed >> 32) as u32 as f64) / (u32::MAX as f64);
            if !p.history.is_empty() {
                p.history.remove(0);
            }
            p.history.push(v);
        }
    }

    /// Legacy: sort in the column's natural direction.
    pub fn sort_procs(procs: &mut [Proc], key: SortKey) {
        sort_procs_dir(procs, key, default_dir(key));
    }

    pub const COL_PID: (f64, f64) = (60.0, 80.0);
    pub const COL_NAME: (f64, f64) = (160.0, 200.0);
    pub const COL_CPU: (f64, f64) = (380.0, 80.0);
    pub const COL_MEM: (f64, f64) = (480.0, 80.0);
    pub const HEADER_Y: f64 = 90.0;

    pub fn header_hit(x: f64, y: f64) -> Option<SortKey> {
        if !(HEADER_Y - 22.0..=HEADER_Y + 6.0).contains(&y) {
            return None;
        }
        if (COL_PID.0..COL_PID.0 + COL_PID.1).contains(&x) {
            Some(SortKey::Pid)
        } else if (COL_NAME.0..COL_NAME.0 + COL_NAME.1).contains(&x) {
            // BUG #4 fix: COMMAND column now hit-testable
            Some(SortKey::Name)
        } else if (COL_CPU.0..COL_CPU.0 + COL_CPU.1).contains(&x) {
            Some(SortKey::Cpu)
        } else if (COL_MEM.0..COL_MEM.0 + COL_MEM.1).contains(&x) {
            Some(SortKey::Mem)
        } else {
            None
        }
    }

    /// Initial sort key shipped by the demo. BUG #3 was that this was Cpu
    /// AND the fixture is already CPU-ordered, so the first CPU click was
    /// a silent no-op. The fix is `Pid`. Probar tests assert this returns
    /// a sort key whose ordering differs from CPU's.
    pub const INITIAL_SORT: SortKey = SortKey::Pid;

    /// Initial paint-order — what the demo paints on first mount.
    pub fn initial_paint_order() -> Vec<u32> {
        let mut procs = fixture();
        sort_procs(&mut procs, INITIAL_SORT);
        procs.into_iter().map(|p| p.pid).collect()
    }
}

pub mod showcase {
    #[derive(Clone, Debug, PartialEq)]
    pub struct State {
        pub frame: u64,
        pub seed: u64,
        pub bars: [f64; 8],
        pub targets: [f64; 8],
        pub donut: f64,
        pub particles: Vec<(f64, f64, f64, f64, f64)>,
        pub softmax_in: [f64; 16],
    }

    impl State {
        pub fn new() -> Self {
            Self {
                frame: 0,
                seed: 0x1234_5678_DEAD_BEEF,
                bars: [0.3, 0.5, 0.7, 0.4, 0.6, 0.2, 0.8, 0.5],
                targets: [0.3, 0.5, 0.7, 0.4, 0.6, 0.2, 0.8, 0.5],
                donut: 0.0,
                particles: vec![],
                softmax_in: [0.0; 16],
            }
        }

        pub fn lcg(&mut self) -> f64 {
            self.seed = self
                .seed
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            ((self.seed >> 32) as u32 as f64) / (u32::MAX as f64)
        }

        pub fn tick(&mut self) {
            self.frame += 1;
            for i in 0..8 {
                self.bars[i] += (self.targets[i] - self.bars[i]) * 0.05;
            }
            if self.frame % 60 == 0 {
                for i in 0..8 {
                    self.targets[i] = self.lcg();
                }
            }
            self.donut += 0.02;
            if self.particles.len() < 50 && self.lcg() < 0.2 {
                let vx = (self.lcg() - 0.5) * 4.0;
                let vy = (self.lcg() - 0.5) * 4.0;
                self.particles.push((640.0, 360.0, vx, vy, 1.0));
            }
            self.particles.retain_mut(|p| {
                p.0 += p.2;
                p.1 += p.3;
                p.4 -= 0.01;
                p.4 > 0.0
            });
            for i in 0..16 {
                self.softmax_in[i] = ((self.frame as f64 * 0.05) + i as f64 * 0.4).sin();
            }
        }
    }

    impl Default for State {
        fn default() -> Self {
            Self::new()
        }
    }

    pub fn softmax(xs: &[f64]) -> Vec<f64> {
        let max = xs.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let exps: Vec<f64> = xs.iter().map(|&x| (x - max).exp()).collect();
        let sum: f64 = exps.iter().sum();
        exps.iter().map(|&e| e / sum).collect()
    }
}

pub mod wasm_dash {
    use super::DrawOp;
    use m5_dash::{build_paint_list, Dashboard};

    pub fn lcg(seed: &mut u64) -> f64 {
        *seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((*seed >> 32) as u32 as f64) / (u32::MAX as f64)
    }

    pub fn jitter(dash: &mut Dashboard, seed: &mut u64) {
        for v in dash.cpu_load.iter_mut() {
            let delta = (lcg(seed) - 0.5) * 0.08;
            *v = (*v + delta).clamp(0.05, 0.99);
        }
        let mem_delta = (lcg(seed) - 0.5) * 0.4;
        dash.mem_used_gb = (dash.mem_used_gb + mem_delta).clamp(0.5, dash.mem_total_gb - 0.5);
        dash.event_count += 1;
    }

    /// Pure paint-op list for the wasm-dash demo. The wasm paint()
    /// wrapper iterates this. Tests snapshot the list — bug #2 was
    /// that header rows (`title`, `event-pulse`, `mem-gauge`) painted
    /// only the label + "0%" because fill=0.0. The fix special-cases
    /// each header. The probar test asserts the special-case fired.
    pub fn paint_ops(dash: &Dashboard, _w: f64, _h: f64) -> Vec<DrawOp> {
        let mut ops = Vec::new();
        for cmd in build_paint_list(dash) {
            // background
            ops.push(DrawOp::Fill("#161b22".into()));
            ops.push(DrawOp::FillRect(
                cmd.bounds.x,
                cmd.bounds.y,
                cmd.bounds.w,
                cmd.bounds.h,
            ));

            match cmd.label.as_str() {
                "title" => {
                    ops.push(DrawOp::Fill("#a5d6ff".into()));
                    ops.push(DrawOp::Font("22px monospace".into()));
                    ops.push(DrawOp::FillText(
                        "wasm-from-zero · capstone".into(),
                        cmd.bounds.x + 10.0,
                        cmd.bounds.y + 36.0,
                    ));
                }
                "event-pulse" => {
                    ops.push(DrawOp::Fill("rgb(127,153,255)".into()));
                    ops.push(DrawOp::Font("28px monospace".into()));
                    ops.push(DrawOp::FillText(
                        format!("⚡ {}", dash.event_count),
                        cmd.bounds.x + 10.0,
                        cmd.bounds.y + 40.0,
                    ));
                    ops.push(DrawOp::Fill("#7c3aed".into()));
                    ops.push(DrawOp::Font("11px monospace".into()));
                    ops.push(DrawOp::FillText(
                        "events".into(),
                        cmd.bounds.x + 10.0,
                        cmd.bounds.y + 54.0,
                    ));
                }
                "mem-gauge" => {
                    let pct = dash.mem_used_gb / dash.mem_total_gb;
                    ops.push(DrawOp::Fill("rgb(51,216,127)".into()));
                    ops.push(DrawOp::Font("28px monospace".into()));
                    ops.push(DrawOp::FillText(
                        format!("{:.0}%", pct * 100.0),
                        cmd.bounds.x + 10.0,
                        cmd.bounds.y + 40.0,
                    ));
                    ops.push(DrawOp::Fill("#7c3aed".into()));
                    ops.push(DrawOp::Font("11px monospace".into()));
                    ops.push(DrawOp::FillText(
                        format!("mem · {:.1}/{:.1} GB", dash.mem_used_gb, dash.mem_total_gb),
                        cmd.bounds.x + 10.0,
                        cmd.bounds.y + 54.0,
                    ));
                }
                "mem-bar" => {
                    // BUG #10 fix (gist round 2): mem-bar is a wide
                    // horizontal strip (w=viewport.w, h=40); previously
                    // it fell through to the vertical-fill branch so it
                    // always painted at 100% width regardless of the
                    // gauge percentage. Fill width by gauge, not height.
                    let fill_w = cmd.bounds.w * cmd.fill.clamp(0.0, 1.0);
                    let r = (cmd.fg.r.clamp(0.0, 1.0) * 255.0) as u8;
                    let g = (cmd.fg.g.clamp(0.0, 1.0) * 255.0) as u8;
                    let b = (cmd.fg.b.clamp(0.0, 1.0) * 255.0) as u8;
                    ops.push(DrawOp::Fill(format!("rgb({r},{g},{b})")));
                    ops.push(DrawOp::FillRect(
                        cmd.bounds.x,
                        cmd.bounds.y,
                        fill_w,
                        cmd.bounds.h,
                    ));
                    ops.push(DrawOp::Fill("#c9d1d9".into()));
                    ops.push(DrawOp::Font("13px monospace".into()));
                    ops.push(DrawOp::FillText(
                        cmd.label.clone(),
                        cmd.bounds.x + 6.0,
                        cmd.bounds.y + 16.0,
                    ));
                    ops.push(DrawOp::Font("11px monospace".into()));
                    ops.push(DrawOp::FillText(
                        format!("{:.0}%", cmd.fill * 100.0),
                        cmd.bounds.x + 6.0,
                        cmd.bounds.y + cmd.bounds.h - 6.0,
                    ));
                }
                _ => {
                    let fill_h = cmd.bounds.h * cmd.fill.clamp(0.0, 1.0);
                    let r = (cmd.fg.r.clamp(0.0, 1.0) * 255.0) as u8;
                    let g = (cmd.fg.g.clamp(0.0, 1.0) * 255.0) as u8;
                    let b = (cmd.fg.b.clamp(0.0, 1.0) * 255.0) as u8;
                    ops.push(DrawOp::Fill(format!("rgb({r},{g},{b})")));
                    ops.push(DrawOp::FillRect(
                        cmd.bounds.x,
                        cmd.bounds.y + cmd.bounds.h - fill_h,
                        cmd.bounds.w,
                        fill_h,
                    ));
                    ops.push(DrawOp::Fill("#c9d1d9".into()));
                    // BUG #11 (round 2/3 QA): users read `cpu-0` as
                    // `cpu-8` at small fonts when the hyphen blends
                    // into the `0` glyph. Display label: "CPU N"
                    // (space separator + caps + clearer font stack).
                    let display_label = if let Some(n) = cmd.label.strip_prefix("cpu-") {
                        format!("CPU {n}")
                    } else {
                        cmd.label.clone()
                    };
                    ops.push(DrawOp::Font(
                        "14px ui-monospace, SFMono-Regular, Menlo, Consolas, monospace".into(),
                    ));
                    ops.push(DrawOp::FillText(
                        display_label,
                        cmd.bounds.x + 6.0,
                        cmd.bounds.y + 18.0,
                    ));
                    ops.push(DrawOp::Font(
                        "11px ui-monospace, SFMono-Regular, Menlo, Consolas, monospace".into(),
                    ));
                    ops.push(DrawOp::FillText(
                        format!("{:.0}%", cmd.fill * 100.0),
                        cmd.bounds.x + 6.0,
                        cmd.bounds.y + cmd.bounds.h - 6.0,
                    ));
                }
            }
        }
        ops
    }
}

pub mod shell {
    pub const SUGGESTIONS: &[(&str, &[&str])] = &[
        ("g", &["git status", "git push", "git commit -am"]),
        ("gi", &["git status", "git push", "git commit -am"]),
        ("git", &["git status", "git push", "git commit -am"]),
        ("git ", &["git status", "git push", "git commit -am"]),
        ("git s", &["git status", "git stash", "git show HEAD"]),
        (
            "git p",
            &["git push", "git pull", "git push --force-with-lease"],
        ),
        ("c", &["cargo build", "cargo test", "cargo run --release"]),
        ("ca", &["cargo build", "cargo test", "cargo run --release"]),
        ("car", &["cargo build", "cargo test", "cargo run --release"]),
        (
            "cargo",
            &["cargo build", "cargo test", "cargo run --release"],
        ),
        (
            "cargo ",
            &["cargo build", "cargo test", "cargo run --release"],
        ),
        (
            "cargo b",
            &[
                "cargo build",
                "cargo build --release",
                "cargo build --target wasm32-unknown-unknown",
            ],
        ),
        (
            "cargo t",
            &[
                "cargo test",
                "cargo test --workspace",
                "cargo test -p m5-dash",
            ],
        ),
        (
            "d",
            &["docker ps", "docker run -it ubuntu", "docker compose up"],
        ),
        (
            "doc",
            &["docker ps", "docker run -it ubuntu", "docker compose up"],
        ),
        (
            "docker",
            &["docker ps", "docker run -it ubuntu", "docker compose up"],
        ),
        (
            "docker ",
            &["docker ps", "docker run -it ubuntu", "docker compose up"],
        ),
        (
            "k",
            &[
                "kubectl get pods",
                "kubectl logs -f",
                "kubectl describe pod",
            ],
        ),
        (
            "kub",
            &[
                "kubectl get pods",
                "kubectl logs -f",
                "kubectl describe pod",
            ],
        ),
        ("ls", &["ls -la", "ls -lah --color", "ls -1"]),
        ("m", &["make demo", "make serve", "make wasm"]),
        ("mak", &["make demo", "make serve", "make wasm"]),
        ("make", &["make demo", "make serve", "make wasm"]),
        ("make ", &["make demo", "make serve", "make wasm"]),
        (
            "python",
            &[
                "python3 -m http.server",
                "python -m pytest",
                "python -m venv .venv",
            ],
        ),
        (
            "ssh",
            &["ssh dev", "ssh -i ~/.ssh/key host", "ssh host -p 22"],
        ),
    ];

    /// BUG #13 fix (gist round 2): the previous longest-prefix lookup
    /// matched on the input's leading characters regardless of whether
    /// the rest of the buffer was still a prefix of any actual command.
    /// E.g. `madocker ps` still returned `make demo/serve/wasm` because
    /// `m` was a dict prefix and nothing checked whether `madocker ps`
    /// is mid-typing any of those suggestions.
    ///
    /// New shape: find the longest dict prefix that matches AND has at
    /// least one suggestion of which the full input is itself a prefix
    /// (i.e. the user is mid-typing it). Filter to those matching
    /// suggestions so the panel cleanly empties when the buffer goes
    /// off-script.
    pub fn lookup(prefix: &str) -> Vec<&'static str> {
        if prefix.is_empty() {
            return vec![];
        }
        let mut best: Vec<&'static str> = vec![];
        let mut best_len = 0;
        for (p, sugs) in SUGGESTIONS {
            if !prefix.starts_with(*p) {
                continue;
            }
            if p.len() < best_len {
                continue;
            }
            let filtered: Vec<&'static str> = sugs
                .iter()
                .copied()
                .filter(|s| s.starts_with(prefix))
                .collect();
            if !filtered.is_empty() {
                best = filtered;
                best_len = p.len();
            }
        }
        best
    }

    /// BUG #12 helper: keep the canvas-painted "input" buffer honest by
    /// ignoring any key event that came with a Ctrl/Meta modifier. The
    /// pre-fix handler would, e.g. on `Ctrl+A`, append a literal `a`
    /// because the key payload arrives as just `"a"`.
    #[must_use]
    pub fn should_ignore_modifier_key(ctrl: bool, meta: bool) -> bool {
        ctrl || meta
    }
}
