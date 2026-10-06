//! LETTERS AND POSTCARDS IN RUST — a `.letter.rs` and a `.postcard.rs`, the paper mail that CommandAGI's postal mail
//! prints and mails. The same elements, and the same documents, as the TypeScript SDK's `postal.ts` and the Python
//! SDK's `postal.py`:
//!
//! ```
//! use commandagi::design::postal::*;
//!
//! fn document() -> El {
//!     letter([
//!         to().name("Ada Lovelace").line1("12 St James's Square").city("London").postal_code("SW1Y 4JH").country("GB"),
//!         from().name("Northwind Survey").line1("1 Main St").city("Portland").region("OR").postal_code("97201").country("US"),
//!         paragraph().text("Dear Ada,"),
//!         paragraph().text("Thank you for your order. It ships on Monday."),
//!     ])
//!     .mail_class("first")
//! }
//! # commandagi::design::declare(document()).unwrap();
//! ```
//!
//! The rule of the ontology's files: a record is an element and its fields are the element's attributes. The
//! addresses are one `to()` and one `from()`; the body is `paragraph().text(…)` children, in order. A postcard has no
//! options and one `front().image(…)`: a ref to a JPEG or PNG relative to the postcard's folder. Nothing adds a
//! default. A run gives back `{format, document, sources}` beside an empty graph; `sources` names each element's site
//! by its path (`""`, `to`, `paragraph@0`).

use super::documents::{declare_document, DocTree, TagRule, Vocabulary};
use super::element::{attributes, elements, rust_name};
use super::{Declared, Family};
pub use super::{fragment, json, El, Json, Value};

elements! {
    /// A letter: its addresses and its paragraphs.
    letter: children;
    /// A postcard: its addresses, its front image and the message's paragraphs.
    postcard: children;
    /// Where the piece goes: `.name("Ada Lovelace").line1(…).city(…).postal_code(…).country("GB")`.
    to: leaf;
    /// Who sends it, as `to()`.
    from: leaf;
    /// A postcard's front: `.image("Austin.jpg")`, a ref relative to the postcard's folder.
    front: leaf;
    /// One paragraph: `.text("Dear Ada,")`.
    paragraph: leaf;
}

attributes! {
    /// The fields of a letter, an address, a front and a paragraph, chained on their elements.
    pub trait PostalAttrs {
        color; double_sided; mail_class;
        name; company; line1; line2; city; region; postal_code; country;
        image; text;
    }
}

pub(crate) const FAMILY: Family = Family { module: "postal", roots: &["letter", "postcard"], declare };

/// The fields of an address (`LetterAddress`).
pub const ADDRESS_FIELDS: &[&str] = &["name", "company", "line1", "line2", "city", "region", "postalCode", "country"];
/// The options of a letter.
pub const LETTER_FIELDS: &[&str] = &["color", "doubleSided", "mailClass"];

const LETTER: Vocabulary = Vocabulary {
    format: "letter",
    noun: "a letter",
    root: "letter",
    tags: &[
        TagRule::new("letter", &[]).attrs(LETTER_FIELDS),
        TagRule::new("to", &["letter"]).single().attrs(ADDRESS_FIELDS),
        TagRule::new("from", &["letter"]).single().attrs(ADDRESS_FIELDS),
        TagRule::new("paragraph", &["letter"]).required(&["text"]).attrs(&["text"]),
    ],
    from_tree: letter_of,
};

const POSTCARD: Vocabulary = Vocabulary {
    format: "postcard",
    noun: "a postcard",
    root: "postcard",
    tags: &[
        TagRule::new("postcard", &[]).attrs(&[]),
        TagRule::new("to", &["postcard"]).single().attrs(ADDRESS_FIELDS),
        TagRule::new("from", &["postcard"]).single().attrs(ADDRESS_FIELDS),
        TagRule::new("front", &["postcard"]).single().required(&["image"]).attrs(&["image"]),
        TagRule::new("paragraph", &["postcard"]).required(&["text"]).attrs(&["text"]),
    ],
    from_tree: postcard_of,
};

fn declare(root: El) -> Result<Declared, String> {
    declare_document(&root, if root.tag == "letter" { &LETTER } else { &POSTCARD })
}

fn check(t: &DocTree) -> Result<(), String> {
    for k in ["color", "doubleSided"] {
        if matches!(t.attr(k), Some(v) if !matches!(v, Json::Bool(_))) {
            return Err(format!("letter(): {} is true or false", rust_name(k)));
        }
    }
    if matches!(t.attr("mailClass"), Some(v) if *v != Json::from("first") && *v != Json::from("standard")) {
        return Err(r#"letter(): mail_class is "first" or "standard""#.into());
    }
    for c in &t.children {
        if c.tag == "paragraph" && !matches!(c.attr("text"), Some(Json::Str(_))) {
            return Err("paragraph(): text is text".into());
        }
        if c.tag == "to" || c.tag == "from" {
            if let Some((k, _)) = c.attrs.iter().find(|(_, v)| !matches!(v, Json::Str(_))) {
                return Err(format!("{}(): {} is text", c.tag, rust_name(k)));
            }
        }
    }
    Ok(())
}

/// What a letter and a postcard share (the addresses, a front, the paragraphs), after `out` (a letter's options).
fn piece_of(t: &DocTree, mut out: Json) -> Result<Json, String> {
    check(t)?;
    for side in ["to", "from"] {
        if let Some(a) = t.single(side) {
            out.set(side, a.record());
        }
    }
    if let Some(f) = t.single("front") {
        match f.attr("image") {
            Some(image @ Json::Str(_)) => {
                out.set("front", image.clone());
            }
            _ => return Err("front(): image is the ref of a JPEG or PNG".into()),
        }
    }
    let paragraphs: Vec<Json> = t.children.iter().filter(|c| c.tag == "paragraph").filter_map(|c| c.attr("text").cloned()).collect();
    Ok(out.with("paragraphs", paragraphs))
}

fn letter_of(t: &DocTree) -> Result<Json, String> {
    let mut head = Json::obj();
    for k in LETTER_FIELDS {
        // An empty text is not said, as the TypeScript SDK drops it.
        if let Some(v) = t.attr(k).filter(|v| **v != Json::from("")) {
            head.set(k, v.clone());
        }
    }
    piece_of(t, head)
}

fn postcard_of(t: &DocTree) -> Result<Json, String> {
    piece_of(t, Json::obj())
}
