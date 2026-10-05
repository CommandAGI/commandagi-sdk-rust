//! `commandagi::design` — CommandAGI documents declared in Rust, as the op graphs the CommandAGI editors store.
//!
//! One module per document family, each declaring the editor's own nodes (no second model): [`sheet`] is the
//! circuit editor's schematic sheet. A file's `document()` returns a [`Document`]; the sandbox declares it
//! ([`declare`]) and hands the graph to the editor. Each family after the schematic adds a module here with its own
//! constructors and a `Document` impl, the same way.

pub mod ir;
pub mod json;
pub mod sheet;
pub mod source;

pub use json::Json;

/// What a file's `document()` returns: something that declares one op graph.
pub trait Document {
    /// The op graph, or why it cannot be declared.
    fn declare(self) -> Result<Json, String>;
}

/// The op graph `document` declares.
pub fn declare<D: Document>(document: D) -> Result<Json, String> {
    document.declare()
}
