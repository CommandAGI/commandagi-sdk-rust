//! A FACT THE FILE DOES NOT KNOW, OR KNOWS ONLY ABOUT (the TypeScript SDK's `facts.ts`). Any attribute of a company, a
//! brand and their children may hold, instead of its value, one of two plain objects:
//!
//! ```
//! use commandagi::design::business::*;
//! use commandagi::design::facts::{about, unknown};
//!
//! let _ = entity().name("Exocortex Labs").jurisdiction(unknown("whether Exocortex Labs is a legal entity is open"));
//! let _ = obligation().id("ship").name("Ship the first 100").due(about("2027-12-31"));
//! ```
//!
//! `{"unknown": "<why>"}` says the fact exists and its value is not known, and why. It is not an absent attribute:
//! absent says nothing. A required attribute may be unknown. `{"about": <value>}` is a value known only roughly, of the
//! attribute's own kind. A reader shows each as written and lists every unknown as an open fact; nothing puts a
//! default or a placeholder code in its place.

use super::Json;

/// An unknown fact, and why it is unknown (what is open, what would settle it).
pub fn unknown(why: &str) -> Json {
    Json::obj().with("unknown", why)
}

/// A value known only roughly.
pub fn about(value: impl Into<Json>) -> Json {
    Json::obj().with("about", value.into())
}

/// `v` as a fact marker, `Ok(None)` for a plain value; a malformed marker is refused by name. `check` holds an
/// `about`'s value to the attribute's own kind.
pub(crate) fn fact_marker(v: &Json, what: &str, check: impl Fn(&Json) -> Result<Json, String>) -> Result<Option<Json>, String> {
    let Json::Obj(fields) = v else { return Ok(None) };
    if !fields.iter().any(|(k, _)| k == "unknown" || k == "about") {
        return Ok(None);
    }
    match fields.as_slice() {
        [(k, Json::Str(why))] if k == "unknown" && !why.trim().is_empty() => Ok(Some(unknown(why.trim()))),
        [(k, value)] if k == "about" && *value != Json::Null => Ok(Some(Json::obj().with("about", check(value)?))),
        _ => Err(format!("{what}: an unknown fact is {{\"unknown\": \"<why>\"}} and an approximate one {{\"about\": <value>}}, each alone, not {}", v.text())),
    }
}
