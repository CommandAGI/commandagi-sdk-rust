//! MONITOR BLUEPRINTS IN RUST — a `.monitors.rs`: many monitors declared at once (docs/monitors.md in the CommandAGI
//! repository). The same elements, and the same document, as the TypeScript SDK's `monitors.ts` and the Python SDK's
//! `monitors.py`. A class of things is a function; its instances are data:
//!
//! ```
//! use commandagi::design::monitors::*;
//!
//! fn series(id: &str, title: &str, code: &str) -> El {
//!     monitor([source().id("fred").adapter("fred").params(json(&format!(r#"{{"series": "{code}"}}"#))).every("6h")])
//!         .id(format!("economy/{id}"))
//!         .title(title)
//!         .class("economic-series")
//!         .about(format!("fred:{code}"))
//! }
//!
//! fn document() -> El {
//!     monitors([
//!         fragment([("cpi", "CPI, all items", "CPIAUCSL"), ("unrate", "Unemployment rate", "UNRATE")].map(|(i, t, c)| series(i, t, c))),
//!         monitor([
//!             source().id("dgs10").adapter("fred").params(json(r#"{"series": "DGS10"}"#)).every("6h"),
//!             follows().monitor("economy/cpi").match_("fred.*"),
//!         ])
//!         .id("economy/treasury-yields")
//!         .title("Treasury yields")
//!         .class("interest-rate"),
//!     ])
//!     .id("economy")
//! }
//! # commandagi::design::declare(document()).unwrap();
//! ```
//!
//! A follow's `match` is written `match_` (a Rust keyword). Nothing adds a default: what the file does not say, the
//! document does not have. A monitor's `sources`, `follows` and `wakes` and the blueprint's `monitors` are always
//! lists.

use super::documents::{declare_document, DocTree, TagRule, Vocabulary};
use super::element::{attributes, elements, fn_name};
use super::{Declared, Family};
pub use super::{fragment, json, El, Json, Value};

elements! {
    /// A blueprint: its monitors.
    monitors: children;
    /// A monitor: one thread, with its sources, its follows and its wakes.
    monitor: children;
    /// A source: an adapter of the sources catalog, its params, and how often it reads.
    source: leaf;
    /// A follow: copy matching records of another monitor into this one.
    follows: leaf;
    /// A wake: wake the thread's agent when a matching record arrives.
    wake: leaf;
}

attributes! {
    /// The fields of a blueprint, a monitor and its parts, chained on their elements.
    pub trait MonitorsAttrs {
        id; title; class; about; visibility; readme;
        adapter; params; every; via; exit;
        monitor; match_; prompt;
    }
}

pub(crate) const FAMILY: Family = Family { module: "monitors", roots: &["monitors"], declare };

pub const BLUEPRINT_FIELDS: &[&str] = &["id", "title"];
pub const MONITOR_FIELDS: &[&str] = &["id", "title", "class", "about", "visibility", "readme"];
pub const SOURCE_FIELDS: &[&str] = &["id", "adapter", "params", "every", "via", "exit"];
pub const FOLLOW_FIELDS: &[&str] = &["monitor", "match"];
pub const WAKE_FIELDS: &[&str] = &["match", "prompt"];
pub const VISIBILITIES: &[&str] = &["public", "unlisted", "private"];
pub const VIAS: &[&str] = &["worker", "host", "vpn"];
/// The shortest and the longest `every` a source may have, in seconds (30s and 7d).
pub const EVERY_BOUNDS: (u64, u64) = (30, 7 * 86_400);

const MONITORS: Vocabulary = Vocabulary {
    format: "monitors",
    noun: "a monitor blueprint",
    root: "monitors",
    tags: &[
        TagRule::new("monitors", &[]).required(&["id"]).attrs(BLUEPRINT_FIELDS),
        TagRule::new("monitor", &["monitors"]).key("id").required(&["id"]).attrs(MONITOR_FIELDS),
        TagRule::new("source", &["monitor"]).key("id").required(&["id", "adapter", "every"]).attrs(SOURCE_FIELDS),
        TagRule::new("follows", &["monitor"]).key("monitor").required(&["monitor"]).attrs(FOLLOW_FIELDS),
        TagRule::new("wake", &["monitor"]).required(&["prompt"]).attrs(WAKE_FIELDS),
    ],
    from_tree: monitors_of,
};

fn declare(root: El) -> Result<Declared, String> {
    declare_document(&root, &MONITORS)
}

/// A duration (`30s`, `15m`, `6h`, `7d`) in seconds, or none when it is not one.
pub fn duration_seconds(text: &str) -> Option<u64> {
    let (n, unit) = text.split_at(text.len().checked_sub(1)?);
    let scale = match unit {
        "s" => 1,
        "m" => 60,
        "h" => 3600,
        "d" => 86_400,
        _ => return None,
    };
    if n.is_empty() || n.starts_with('0') || !n.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    n.parse::<u64>().ok().map(|v| v * scale)
}

/// How a message names an element: `monitor().id("economy/cpi")`.
fn call(t: &DocTree, key: &str) -> String {
    match t.attr(key) {
        Some(v @ Json::Str(_)) => format!("{}().{}({})", fn_name(t.tag), fn_name(key), v.text()),
        _ => format!("{}()", fn_name(t.tag)),
    }
}

fn word(t: &DocTree, k: &str, key: &str) -> Result<(), String> {
    match t.attr(k) {
        None | Some(Json::Null) => Ok(()),
        Some(Json::Str(s)) if !s.is_empty() => Ok(()),
        Some(v) => Err(format!("{} {} is a word, not {}", call(t, key), fn_name(k), v.text())),
    }
}

fn one_of(t: &DocTree, k: &str, values: &[&str]) -> Result<(), String> {
    match t.attr(k) {
        None | Some(Json::Null) => Ok(()),
        Some(Json::Str(s)) if values.contains(&s.as_str()) => Ok(()),
        Some(v) => Err(format!("{} {} is {}, not {}", call(t, "id"), k, values.iter().map(|x| format!("\"{x}\"")).collect::<Vec<_>>().join(", "), v.text())),
    }
}

/// The fields an element says, in the vocabulary's order.
fn said(t: &DocTree, keys: &[&str]) -> Json {
    let mut out = Json::obj();
    for k in keys {
        if let Some(v) = t.attr(k).filter(|v| !matches!(v, Json::Null)) {
            out.set(k, v.clone());
        }
    }
    out
}

fn source_of(t: &DocTree) -> Result<Json, String> {
    for k in ["id", "adapter", "exit"] {
        word(t, k, "id")?;
    }
    if let Some(p) = t.attr("params") {
        if !matches!(p, Json::Obj(_)) {
            return Err(format!("{} params is an object ({{\"series\": \"CPIAUCSL\"}}), not {}", call(t, "id"), p.text()));
        }
    }
    let every = t.attr("every");
    let seconds = match every {
        Some(Json::Str(s)) => duration_seconds(s),
        _ => None,
    };
    let Some(seconds) = seconds else {
        let said = every.map(Json::text).unwrap_or_else(|| "undefined".into());
        return Err(format!("{} every is a duration (\"30s\", \"15m\", \"6h\", \"7d\"), not {said}", call(t, "id")));
    };
    if seconds < EVERY_BOUNDS.0 || seconds > EVERY_BOUNDS.1 {
        return Err(format!("{} every is from 30s to 7d, not {}", call(t, "id"), every.map(Json::text).unwrap_or_default()));
    }
    one_of(t, "via", VIAS)?;
    if t.attr("exit").is_some() && !matches!(t.attr("via"), Some(Json::Str(v)) if v == "vpn") {
        return Err(format!("{} exit names a country for via(\"vpn\") only", call(t, "id")));
    }
    Ok(said(t, SOURCE_FIELDS))
}

fn monitor_of(t: &DocTree) -> Result<Json, String> {
    for k in ["id", "title", "class", "about", "readme"] {
        word(t, k, "id")?;
    }
    one_of(t, "visibility", VISIBILITIES)?;
    let mut sources = Vec::new();
    let mut follows = Vec::new();
    let mut wakes = Vec::new();
    for c in &t.children {
        match c.tag {
            "source" => sources.push(source_of(c)?),
            "follows" => {
                word(c, "monitor", "monitor")?;
                word(c, "match", "monitor")?;
                if c.attr("monitor").is_some() && c.attr("monitor") == t.attr("id") {
                    return Err(format!("{} follows itself; a follow names another monitor", call(t, "id")));
                }
                follows.push(said(c, FOLLOW_FIELDS));
            }
            _ => {
                word(c, "match", "match")?;
                word(c, "prompt", "match")?;
                wakes.push(said(c, WAKE_FIELDS));
            }
        }
    }
    let mut out = said(t, MONITOR_FIELDS);
    out.set("sources", Json::Arr(sources));
    out.set("follows", Json::Arr(follows));
    out.set("wakes", Json::Arr(wakes));
    Ok(out)
}

fn monitors_of(t: &DocTree) -> Result<Json, String> {
    word(t, "id", "id")?;
    word(t, "title", "id")?;
    let mut out = said(t, BLUEPRINT_FIELDS);
    out.set("monitors", Json::Arr(t.children.iter().map(monitor_of).collect::<Result<Vec<_>, _>>()?));
    Ok(out)
}
