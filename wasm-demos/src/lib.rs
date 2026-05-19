//! Gallery of working WASM demos for paiml/wasm-from-zero.
//!
//! Each `mount_*` function is a `#[wasm_bindgen]` entry painted via the
//! same `m1-canvas` + `m3-components` + `m3-charts` primitives the course
//! teaches. Zero hand-written JavaScript — `wasm-bindgen` emits the
//! ES-module loader.

// Pure logic — testable on native via probar snapshots.
pub mod logic;

// Real aprender-shell Markov model loader (vendored from
// presentar::browser::shell_autocomplete) — see `shell_model.rs` for why.
pub mod shell_model;

#[cfg(target_arch = "wasm32")]
mod canvas_demo;
#[cfg(target_arch = "wasm32")]
mod counter_demo;
#[cfg(target_arch = "wasm32")]
mod process_table_demo;
#[cfg(target_arch = "wasm32")]
mod shell_demo;
#[cfg(target_arch = "wasm32")]
mod showcase_demo;
#[cfg(target_arch = "wasm32")]
mod wasm_dash_demo;

// Re-export the mount fns so wasm-bindgen finds them in the generated .js.
#[cfg(target_arch = "wasm32")]
pub use canvas_demo::mount_canvas;
#[cfg(target_arch = "wasm32")]
pub use counter_demo::mount_counter;
#[cfg(target_arch = "wasm32")]
pub use process_table_demo::mount_process_table;
#[cfg(target_arch = "wasm32")]
pub use shell_demo::mount_shell;
#[cfg(target_arch = "wasm32")]
pub use showcase_demo::mount_showcase;
#[cfg(target_arch = "wasm32")]
pub use wasm_dash_demo::mount_wasm_dash;

#[cfg(target_arch = "wasm32")]
pub(crate) mod canvas_helpers {
    use wasm_bindgen::prelude::*;
    use wasm_bindgen::JsCast;

    pub fn get_canvas_ctx(
        canvas_id: &str,
    ) -> Result<
        (
            web_sys::HtmlCanvasElement,
            web_sys::CanvasRenderingContext2d,
        ),
        JsValue,
    > {
        let win = web_sys::window().ok_or_else(|| JsValue::from_str("no window"))?;
        let doc = win
            .document()
            .ok_or_else(|| JsValue::from_str("no document"))?;
        let canvas: web_sys::HtmlCanvasElement = doc
            .get_element_by_id(canvas_id)
            .ok_or_else(|| JsValue::from_str(&format!("no canvas #{canvas_id}")))?
            .dyn_into()
            .map_err(|_| JsValue::from_str("not a canvas"))?;
        let ctx: web_sys::CanvasRenderingContext2d = canvas
            .get_context("2d")?
            .ok_or_else(|| JsValue::from_str("no 2d context"))?
            .dyn_into()
            .map_err(|_| JsValue::from_str("not Canvas2D"))?;
        Ok((canvas, ctx))
    }

    pub fn rgb(r: f64, g: f64, b: f64) -> String {
        let to_byte = |x: f64| (x.clamp(0.0, 1.0) * 255.0) as u8;
        format!("rgb({},{},{})", to_byte(r), to_byte(g), to_byte(b))
    }
}
