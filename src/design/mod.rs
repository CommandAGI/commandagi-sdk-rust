//! `commandagi::design` — CommandAGI documents declared in Rust: the same documents, element for element, as the
//! TypeScript SDK's JSX (`commandagi/design`) and the Python SDK's `commandagi.design`.
//!
//! A file is `fn document() -> El { … }`. The sandbox runs it through [`crate::abi::run`], which finds the family by
//! the root's tag ([`declare`]) and answers the family's op graph, or its document beside an empty graph.
//!
//! # The conventions every family module keeps
//!
//! One module per TypeScript vocabulary file, named as the Python module is (`schematic` for `sheet.ts`, `pcb`,
//! `threed`, `fab`, `office`, `twod`, `media`, `ontology`, `business`, `tasks`, `records`). A module is mostly three
//! things, in this order:
//!
//! 1. **Its tags**, with [`element::elements!`]: one `#[track_caller]` constructor per tag, named snake case of the
//!    tag (`-` → `_`, `CapTable` → `cap_table`, a Rust keyword gets a trailing `_`: `type_`, `move_`), the tag written
//!    beside it when the name is not the tag (`brush_stroke = "brush-stroke"`). Its parameters: the tag's positional
//!    attributes (the schematic only), then, for a tag that holds children (`children`) or text (`text`), ONE
//!    parameter: a container takes `impl IntoIterator<Item = El>` (`layer([])` when empty; `fragment(iter)` items
//!    are flattened), a text tag takes [`Text`] (`p("…")`, `p(("The run ", b("passed"), "."))`, `p([])`). A leaf
//!    (`leaf`) takes no children parameter. These are what `sdk/typescript/src/design/signatures.ts` lists for the
//!    module: keep the two the same.
//! 2. **Its attributes**, with [`element::attributes!`]: one trait (`<Module>Attrs`) of chained methods implemented
//!    for [`El`], each named snake case of the TypeScript attribute (`.frozen_rows(1)` sets `frozenRows`). A value is
//!    anything [`Value`] takes: numbers, `"strings"`, `true`/`false`, homogeneous arrays of these (nested), and
//!    anything else as [`json`]`(r#"…"#)`. Two modules may share a method name: a file brings in one module's trait.
//! 3. **Its declaration**: `fn declare(root: El) -> Result<Declared, String>`, which reads the element tree exactly as
//!    the TypeScript SDK reads the JSX tree, gives the same nodes or document, and refuses by name what it cannot read
//!    (a message names the call, [`El::call_text`], and the attribute by its Rust name, [`element::rust_name`]). It
//!    puts each element's site where TypeScript puts its `source` ([`source::meta`] for a node's `meta.source`,
//!    [`source::Sources`] for `meta.sources[<key>]` and a document's `sources[<key>]`, with TypeScript's keys).
//!    Then a `pub(crate) const FAMILY: Family` names its module, its root tags and its declare function, and one line
//!    in [`FAMILIES`] registers it.
//!
//! Each module `pub use`s [`El`], [`json`], [`fragment`] and [`Value`] so `use commandagi::design::<module>::*;` is
//! the file's one import.

pub mod documents;
pub mod element;
pub mod fab;
pub mod ir;
pub mod json;
pub mod schematic;
pub mod source;
pub mod threed;

pub use element::{fragment, json, Child, El, Text, TextPart, Value};
pub use json::Json;

/// What a family declares from a document's root element.
#[derive(Clone, Debug)]
pub enum Declared {
    /// An op graph: `{"id", "nodes", "meta"?, "outputs"?}`.
    Graph(Json),
    /// A document that is not a graph (a workbook, a world): its JSON and each element's site by key.
    Document { format: String, document: Json, sources: Json },
}

impl Declared {
    /// The graph, or none for a document that is not one.
    pub fn graph(self) -> Option<Json> {
        match self {
            Declared::Graph(g) => Some(g),
            Declared::Document { .. } => None,
        }
    }
}

/// One document family: the module, the root tags it declares, and how.
pub struct Family {
    pub module: &'static str,
    pub roots: &'static [&'static str],
    pub declare: fn(El) -> Result<Declared, String>,
}

/// Every family a file's root may be. A root tag names one family.
pub const FAMILIES: &[Family] = &[schematic::FAMILY, fab::FAMILY, threed::FAMILY];

/// Declare a document from its root element: the family of the root's tag reads it.
pub fn declare(root: El) -> Result<Declared, String> {
    root.refusal()?;
    match FAMILIES.iter().find(|f| f.roots.contains(&root.tag)) {
        Some(family) => (family.declare)(root),
        None => {
            let roots: Vec<String> = FAMILIES
                .iter()
                .flat_map(|f| f.roots.iter().map(move |r| format!("{}::{}", f.module, element::fn_name(r))))
                .collect();
            Err(format!("{} is not the root of a document ({})", root.call_text(), roots.join(", ")))
        }
    }
}
