//! READING AN ELEMENT'S ATTRIBUTES — the checks the families that read each attribute by its kind share (office,
//! twod, media): an attribute is text, a number or true/false, and a message refuses it by its Rust name, at the call
//! that wrote it (`cell().at("A1")`), as TypeScript refuses it by its prop at the JSX element.

use super::element::{fn_name, rust_name, El};
use super::json::Json;

/// How a message names an element: its call, with the first of `keys` it has as text (`cell().at("A1")`, `rect()`).
pub(crate) fn call(el: &El, keys: &[&str]) -> String {
    for k in keys {
        if let Some(v @ Json::Str(_)) = el.attr(k) {
            return format!("{}().{}({})", fn_name(el.tag), rust_name(k), v.text());
        }
    }
    format!("{}()", fn_name(el.tag))
}

/// A tag as a message names it: `sheet()`.
pub(crate) fn tag(t: &str) -> String {
    format!("{}()", fn_name(t))
}

/// Attribute names as a message lists them: their Rust names, joined.
pub(crate) fn names(props: &[&str]) -> String {
    props.iter().map(|p| rust_name(p)).collect::<Vec<_>>().join(", ")
}

/// A text attribute, or none.
pub(crate) fn text(el: &El, prop: &str, at: &str) -> Result<Option<String>, String> {
    match el.attr(prop) {
        None => Ok(None),
        Some(Json::Str(s)) => Ok(Some(s.clone())),
        Some(v) => Err(format!("{at}: {} is text, not {}", rust_name(prop), v.text())),
    }
}

/// A finite number attribute, or none.
pub(crate) fn number(el: &El, prop: &str, at: &str) -> Result<Option<f64>, String> {
    match el.attr(prop) {
        None => Ok(None),
        Some(Json::Num(n)) if n.is_finite() => Ok(Some(*n)),
        Some(v) => Err(format!("{at}: {} is a number, not {}", rust_name(prop), v.text())),
    }
}

/// A true-or-false attribute, or none.
pub(crate) fn boolean(el: &El, prop: &str, at: &str) -> Result<Option<bool>, String> {
    match el.attr(prop) {
        None => Ok(None),
        Some(Json::Bool(b)) => Ok(Some(*b)),
        Some(v) => Err(format!("{at}: {} is true or false, not {}", rust_name(prop), v.text())),
    }
}

/// An object's entries without the absent ones (TypeScript's `defined`).
pub(crate) fn defined(entries: Vec<(&str, Option<Json>)>) -> Json {
    Json::Obj(entries.into_iter().filter_map(|(k, v)| v.map(|v| (k.to_string(), v))).collect())
}

/// A JSON object's number of entries (0 for anything else).
pub(crate) fn len(j: &Json) -> usize {
    match j {
        Json::Obj(e) => e.len(),
        _ => 0,
    }
}
