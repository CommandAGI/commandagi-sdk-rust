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
//! The module has no imports. `commandagi_run` returns 1 and leaves `{"ok":true,"graph","params","sites"}` as UTF-8 at
//! `commandagi_out_ptr` (`commandagi_out_len` bytes), or returns 0 and leaves `{"ok":false,"error"}`. A panic traps
//! (a WebAssembly module cannot unwind); its hook leaves the error first, so the sandbox reads it after the trap.

use crate::design::{declare, source, Document, Json};
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

/// Run the file's `document()` and leave the answer.
pub fn run<D: Document>(document: impl FnOnce() -> D) -> u32 {
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
    match declare(document()) {
        Ok(graph) => {
            let answer = Json::obj().with("ok", true).with("graph", graph).with("params", Json::obj()).with("sites", source::sites());
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
