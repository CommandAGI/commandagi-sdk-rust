//! HOW THE SANDBOX RUNS A FILE — the WebAssembly interface of a compiled `.rs` document.
//!
//! The local host compiles the file into a `cdylib` for `wasm32-unknown-unknown` with this wrapper around it:
//!
//! ```text
//! include!("user.rs");
//! #[no_mangle] pub extern "C" fn commandagi_run() -> u32 { ::commandagi::abi::run(document) }
//! #[no_mangle] pub extern "C" fn commandagi_out_ptr() -> u32 { ::commandagi::abi::out_ptr() }
//! #[no_mangle] pub extern "C" fn commandagi_out_len() -> u32 { ::commandagi::abi::out_len() }
//! ```
//!
//! The module has no imports. `commandagi_run` returns 1 and leaves the answer as UTF-8 at `commandagi_out_ptr`
//! (`commandagi_out_len` bytes): `{"ok":true,"graph","params","sites"}`, and for a document that is not a graph also
//! `"document": {"format","document","sources"}` beside the empty graph `{"id":"Document","nodes":{}}`. Or it returns
//! 0 and leaves `{"ok":false,"error"}`. A panic traps (a WebAssembly module cannot unwind); its hook leaves the error
//! first, so the sandbox reads it after the trap.

use crate::design::{declare, source, Declared, El, Json};
use std::cell::RefCell;

thread_local! {
    static OUT: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
}

fn set_out(text: String) {
    OUT.with(|o| *o.borrow_mut() = text.into_bytes());
}

fn failed(error: String) -> String {
    Json::obj().with("ok", false).with("error", error).text()
}

/// The answer to a declaration: what `run` leaves, as JSON.
pub fn answer(declared: Result<Declared, String>) -> Result<Json, String> {
    let out = Json::obj().with("ok", true);
    Ok(match declared? {
        Declared::Graph(graph) => out.with("graph", graph).with("params", Json::obj()).with("sites", source::sites()),
        Declared::Document { format, document, sources } => out
            .with("graph", Json::obj().with("id", "Document").with("nodes", Json::obj()))
            .with("params", Json::obj())
            .with("sites", source::sites())
            .with("document", Json::obj().with("format", format).with("document", document).with("sources", sources)),
    })
}

/// Run the file's `document()` and leave the answer.
pub fn run(document: impl FnOnce() -> El) -> u32 {
    std::panic::set_hook(Box::new(|info| {
        let what = info
            .payload()
            .downcast_ref::<&str>()
            .map(|s| s.to_string())
            .or_else(|| info.payload().downcast_ref::<String>().cloned())
            .unwrap_or_else(|| "the file panicked".to_string());
        let at = info.location().map(|l| format!(" (line {})", l.line())).unwrap_or_default();
        set_out(failed(format!("{what}{at}")));
    }));
    set_out(failed("the file did not finish".into()));
    match answer(declare(document())) {
        Ok(answer) => {
            set_out(answer.text());
            1
        }
        Err(error) => {
            set_out(failed(error));
            0
        }
    }
}

pub fn out_ptr() -> u32 {
    OUT.with(|o| o.borrow().as_ptr() as usize as u32)
}

pub fn out_len() -> u32 {
    OUT.with(|o| o.borrow().len() as u32)
}

/// The answer left by the last `run`, as text (what the sandbox reads; for tests).
pub fn out_text() -> String {
    OUT.with(|o| String::from_utf8_lossy(&o.borrow()).into_owned())
}
