//! Demo 1 — Canvas2D primitives (M1 visual proof).
//!
//! Paints `m1-canvas::clip` over 8 adversarial inputs (clean / overflow /
//! NaN / inf / negative / zero) and shows a green or red swatch per row
//! depending on whether the input survived. The footer announces the
//! survivor count.
//!
//! Bug history (https://gist.github.com/noahgift/630218c2e66bc4b4b5e1c54cec8f5610):
//!   BUG #6 (round 2) — footer claimed `survived: 3` but only 2 green
//!                       rectangles were visible because the previous
//!                       demo painted a separate scaled preview band
//!                       that placed `overflow-bottom` at y=330 with
//!                       a text label saying `out=(60,540,…)`. Visible
//!                       rect count diverged from the asserted survivor
//!                       count, so the contract assertion was unsound.
//!                       Fix: drop the preview band; paint one inline
//!                       green swatch per survivor right next to its row.

use crate::canvas_helpers::get_canvas_ctx;
use crate::logic::canvas::paint_ops;
use crate::logic::replay;
use m1_canvas::CanvasDims;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn mount_canvas(canvas_id: &str) -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    let (_canvas, ctx) = get_canvas_ctx(canvas_id)?;
    let w = _canvas.width() as f64;
    let h = _canvas.height() as f64;
    let dims = CanvasDims {
        width: w,
        height: h,
    };
    let ops = paint_ops(dims, w, h);
    replay(&ctx, &ops)?;
    Ok(())
}
