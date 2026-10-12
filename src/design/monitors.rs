//! MONITORS IN RUST — a `monitor.rs`: one monitor, in the folder of the thing it follows (docs/monitors.md in the
//! CommandAGI repository). Its id is that folder's path in the account's Drive, never written in the file. The same
//! elements, and the same document, as the TypeScript SDK's `monitors.ts` and the Python SDK's `monitors.py`. A class of
//! things is a function; an instance is one call:
//!
//! ```
//! use commandagi::design::monitors::*;
//!
//! fn corporation(cik: &str, ticker: &str, name: &str) -> El {
//!     monitor([
//!         source().id("filings").adapter("sec-filings").params(json(&format!(r#"{{"cik": "{cik}"}}"#))).every("15m"),
//!         follows().monitor("../../central-banks/fed").match_("fomc.*"),
//!     ])
//!     .title(format!("{name} ({ticker})"))
//!     .class("corporation")
//!     .about(format!("cik:{cik}"))
//! }
//! # commandagi::design::declare(corporation("0001045810", "NVDA", "NVIDIA")).unwrap();
//! ```
//!
//! A follow names the followed monitor's folder, relative to this monitor's folder. A follow's `match` is written
//! `match_` (a Rust keyword). Nothing adds a default: what the file does not say, the document does not have. A
//! monitor's `sources`, `follows` and `wakes` are always lists.

use super::documents::{declare_document, DocTree, TagRule, Vocabulary};
use super::element::{attributes, elements, fn_name};
use super::{Declared, Family};
pub use super::{fragment, json, El, Json, Value};

elements! {
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
    /// The fields of a monitor and its parts, chained on their elements.
    pub trait MonitorsAttrs {
        id; title; class; about; readme;
        adapter; params; every; via; exit; host; mode; retain; since;
        session; snapshot; profile; keep; max_cost_per_day; unit;
        monitor; match_; prompt;
    }
}

pub(crate) const FAMILY: Family = Family { module: "monitors", roots: &["monitor"], declare };

pub const MONITOR_FIELDS: &[&str] = &["title", "class", "about", "readme"];
pub const SOURCE_FIELDS: &[&str] = &[
    "id", "adapter", "params", "every", "via", "exit", "host", "mode", "retain", "since", "session", "snapshot", "profile",
    "keep", "maxCostPerDay", "unit",
];
pub const FOLLOW_FIELDS: &[&str] = &["monitor", "match"];
pub const WAKE_FIELDS: &[&str] = &["match", "prompt"];
/// The paths a source's reads take (docs/sources.md § fetch paths).
pub const VIAS: &[&str] = &["worker", "host", "vpn", "browser", "vm", "android-vm", "phone"];
/// The paths that launch a machine from a snapshot (a logged-in state), and so cost money per read.
pub const VM_VIAS: &[&str] = &["vm", "android-vm"];
/// `poll` reads at each `every`; `stream` holds a connection open on a host.
pub const MODES: &[&str] = &["poll", "stream"];
/// The shortest and the longest `every` a source may have, in seconds (30s and 7d).
pub const EVERY_BOUNDS: (u64, u64) = (30, 7 * 86_400);

const MONITOR: Vocabulary = Vocabulary {
    format: "monitor",
    noun: "a monitor",
    root: "monitor",
    tags: &[
        TagRule::new("monitor", &[]).attrs(MONITOR_FIELDS),
        TagRule::new("source", &["monitor"]).key("id").required(&["id", "adapter", "every"]).attrs(SOURCE_FIELDS),
        TagRule::new("follows", &["monitor"]).key("monitor").required(&["monitor"]).attrs(FOLLOW_FIELDS),
        TagRule::new("wake", &["monitor"]).required(&["prompt"]).attrs(WAKE_FIELDS),
    ],
    from_tree: monitor_of,
};

fn declare(root: El) -> Result<Declared, String> {
    declare_document(&root, &MONITOR)
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

/// How a message names an element: `source().id("fred")`.
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

fn str_attr<'a>(t: &'a DocTree, k: &str) -> Option<&'a str> {
    match t.attr(k) {
        Some(Json::Str(s)) => Some(s.as_str()),
        _ => None,
    }
}

fn has(t: &DocTree, k: &str) -> bool {
    !matches!(t.attr(k), None | Some(Json::Null))
}

/// `all`, `derived` or `sample:<duration>`.
fn retain_ok(r: &str) -> bool {
    r == "all" || r == "derived" || r.strip_prefix("sample:").and_then(duration_seconds).is_some()
}

/// `YYYY-MM-DD` with a month 01–12 and a day 01–31.
fn day_ok(d: &str) -> bool {
    let b = d.as_bytes();
    if b.len() != 10 || b[4] != b'-' || b[7] != b'-' || ![0, 1, 2, 3, 5, 6, 8, 9].iter().all(|&i| b[i].is_ascii_digit()) {
        return false;
    }
    let (m, day): (u32, u32) = (d[5..7].parse().unwrap_or(0), d[8..10].parse().unwrap_or(0));
    (1..=12).contains(&m) && (1..=31).contains(&day)
}

/// A phone unit: `<world.tsx ref>#<uid>` (`../../worlds/fleet/world.tsx#android-01`).
pub fn unit_ref_ok(u: &str) -> bool {
    let Some((world, uid)) = u.split_once('#') else { return false };
    let ub = uid.as_bytes();
    world.ends_with("/world.tsx")
        && world.len() > "/world.tsx".len()
        && !world.contains(char::is_whitespace)
        && !uid.is_empty()
        && uid.len() <= 48
        && (ub[0].is_ascii_lowercase() || ub[0].is_ascii_digit())
        && ub.iter().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || *c == b'-')
}

fn source_of(t: &DocTree) -> Result<Json, String> {
    for k in ["id", "adapter", "exit", "host", "retain", "since", "session", "snapshot", "profile", "keep", "unit"] {
        word(t, k, "id")?;
    }
    let w = call(t, "id");
    if let Some(p) = t.attr("params") {
        if !matches!(p, Json::Obj(_)) {
            return Err(format!("{w} params is an object ({{\"series\": \"CPIAUCSL\"}}), not {}", p.text()));
        }
    }
    let every = t.attr("every");
    let seconds = match every {
        Some(Json::Str(s)) => duration_seconds(s),
        _ => None,
    };
    let Some(seconds) = seconds else {
        let said = every.map(Json::text).unwrap_or_else(|| "undefined".into());
        return Err(format!("{w} every is a duration (\"30s\", \"15m\", \"6h\", \"7d\"), not {said}"));
    };
    if seconds < EVERY_BOUNDS.0 || seconds > EVERY_BOUNDS.1 {
        return Err(format!("{w} every is from 30s to 7d, not {}", every.map(Json::text).unwrap_or_default()));
    }
    one_of(t, "via", VIAS)?;
    let via = str_attr(t, "via");
    if has(t, "exit") && via != Some("vpn") {
        return Err(format!("{w} exit names an exit country (\"DE\") or an exit (\"fsn-1\"), for via=\"vpn\" only"));
    }
    one_of(t, "mode", MODES)?;
    let on_vm = via.is_some_and(|v| VM_VIAS.contains(&v));
    if has(t, "host") && !matches!(via, Some("host" | "browser" | "phone")) && !on_vm {
        return Err(format!("{w} host names the host that reads it, for via=\"host\", \"browser\", \"phone\", \"vm\" or \"android-vm\" only"));
    }
    if has(t, "profile") && via != Some("browser") {
        return Err(format!("{w} profile names a browser profile of the host, for via=\"browser\" only"));
    }
    if has(t, "session") && via != Some("browser") && !on_vm {
        return Err(format!("{w} session names a logged-in state, for via=\"browser\", \"vm\" or \"android-vm\" only"));
    }
    if has(t, "session") && has(t, "profile") {
        return Err(format!("{w} names its state once: a session or a profile"));
    }
    if has(t, "snapshot") && !on_vm {
        return Err(format!("{w} snapshot is what a via=\"vm\" or \"android-vm\" read launches from"));
    }
    if has(t, "session") && has(t, "snapshot") {
        return Err(format!("{w} names its state once: a session or a snapshot"));
    }
    let v = via.unwrap_or("");
    if on_vm && !has(t, "session") && !has(t, "snapshot") {
        return Err(format!("{w} via=\"{v}\" launches from a snapshot: it names a session or a snapshot"));
    }
    let cost = t.attr("maxCostPerDay").filter(|c| !matches!(c, Json::Null));
    if cost.is_some() && !on_vm {
        return Err(format!("{w} maxCostPerDay bounds what a via=\"vm\" or \"android-vm\" source spends"));
    }
    if on_vm && cost.is_none() {
        return Err(format!("{w} via=\"{v}\" costs money: it says maxCostPerDay (USD a day)"));
    }
    if let Some(c) = cost {
        if !matches!(c, Json::Num(n) if n.is_finite() && *n > 0.0) {
            return Err(format!("{w} maxCostPerDay is a price in USD (2.5), not {}", c.text()));
        }
    }
    if has(t, "keep") && !on_vm {
        return Err(format!("{w} keep is how long a via=\"vm\" or \"android-vm\" machine stays up"));
    }
    if let Some(k) = str_attr(t, "keep") {
        if !duration_seconds(k).is_some_and(|s| s >= EVERY_BOUNDS.0 && s <= EVERY_BOUNDS.1) {
            return Err(format!("{w} keep is a duration from 30s to 7d, not \"{k}\""));
        }
    }
    if has(t, "unit") && via != Some("phone") {
        return Err(format!("{w} unit names the phone a via=\"phone\" read runs on"));
    }
    if via == Some("phone") && !has(t, "unit") {
        return Err(format!("{w} via=\"phone\" names its phone: unit=\"<world.tsx>#<uid>\""));
    }
    if let Some(u) = str_attr(t, "unit") {
        if !unit_ref_ok(u) {
            return Err(format!("{w} unit is \"<world.tsx ref>#<uid>\" (\"../../worlds/fleet/world.tsx#android-01\"), not \"{u}\""));
        }
    }
    let stream = str_attr(t, "mode") == Some("stream");
    if stream && via != Some("host") {
        return Err(format!("{w} mode=\"stream\" holds a connection open, which runs on a host (via=\"host\")"));
    }
    if has(t, "retain") && !stream {
        return Err(format!("{w} retain is for mode=\"stream\" only"));
    }
    if let Some(r) = str_attr(t, "retain") {
        if !retain_ok(r) {
            return Err(format!("{w} retain is \"all\", \"derived\" or \"sample:<duration>\", not \"{r}\""));
        }
    }
    if let Some(d) = str_attr(t, "since") {
        if !day_ok(d) {
            return Err(format!("{w} since is a day (\"2024-01-01\"), not \"{d}\""));
        }
        if stream {
            return Err(format!("{w} since is for a polled source; a stream starts when it connects"));
        }
    }
    Ok(said(t, SOURCE_FIELDS))
}

fn monitor_of(t: &DocTree) -> Result<Json, String> {
    for k in MONITOR_FIELDS {
        word(t, k, "title")?;
    }
    let mut sources = Vec::new();
    let mut follows = Vec::new();
    let mut wakes = Vec::new();
    for c in &t.children {
        match c.tag {
            "source" => sources.push(source_of(c)?),
            "follows" => {
                word(c, "monitor", "monitor")?;
                word(c, "match", "monitor")?;
                if matches!(c.attr("monitor"), Some(Json::Str(m)) if m == "." || m == "./") {
                    return Err(format!("{} names this monitor; a follow names another monitor's folder", call(c, "monitor")));
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

/// A monitor as the platform reads it: what the file does not say, filled (`via: "worker"`, `mode: "poll"`, the others
/// null; a stream's `retain` "derived"). The TypeScript SDK's `monitorWithDefaults`.
pub fn monitor_with_defaults(m: &Json) -> Json {
    let or_null = |o: &Json, k: &str| o.get(k).cloned().unwrap_or(Json::Null);
    let list = |k: &str| match m.get(k) {
        Some(Json::Arr(a)) => a.clone(),
        _ => Vec::new(),
    };
    let sources = list("sources")
        .iter()
        .map(|s| {
            let stream = s.get("mode").and_then(Json::as_str) == Some("stream");
            let mut out = Json::obj();
            out.set("id", or_null(s, "id"));
            out.set("adapter", or_null(s, "adapter"));
            out.set("params", s.get("params").cloned().unwrap_or_else(Json::obj));
            out.set("every", or_null(s, "every"));
            out.set("via", s.get("via").cloned().unwrap_or_else(|| Json::Str("worker".into())));
            out.set("exit", or_null(s, "exit"));
            out.set("host", or_null(s, "host"));
            out.set("mode", s.get("mode").cloned().unwrap_or_else(|| Json::Str("poll".into())));
            out.set("retain", if stream { s.get("retain").cloned().unwrap_or_else(|| Json::Str("derived".into())) } else { Json::Null });
            for k in ["since", "session", "snapshot", "profile", "keep", "maxCostPerDay", "unit"] {
                out.set(k, or_null(s, k));
            }
            out
        })
        .collect();
    let follows = list("follows").iter().map(|f| Json::obj().with("monitor", or_null(f, "monitor")).with("match", or_null(f, "match"))).collect();
    let wakes = list("wakes").iter().map(|w| Json::obj().with("match", or_null(w, "match")).with("prompt", or_null(w, "prompt"))).collect();
    let mut out = Json::obj();
    for k in MONITOR_FIELDS {
        out.set(k, or_null(m, k));
    }
    out.set("sources", Json::Arr(sources));
    out.set("follows", Json::Arr(follows));
    out.set("wakes", Json::Arr(wakes));
    out
}
