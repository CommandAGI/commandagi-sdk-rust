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
//! `match_` (a Rust keyword), and a discover's `into` is written `into_` (`Into::into` has the name). Nothing adds a
//! default: what the file does not say, the document does not have. A monitor's `sameAs`, `sources`, `follows`, `derives`, `wakes` and `discovers` are always lists.
//!
//! A `derive` folds other monitors' records into this one's, each port named to a folder; a `discover` gives each thing
//! its records name a folder and a thread of its own, an instance of a class (`<file>#<Component>`):
//!
//! ```
//! use commandagi::design::monitors::*;
//!
//! let security = monitor([
//!     derive().id("vwap").op("vwap").inputs(json(r#"{"trades": "../trades"}"#)).params(json(r#"{"window": "1m"}"#)),
//!     discover()
//!         .of(json(r#"["./kev", "./cve-feed"]"#))
//!         .into_("./exploits")
//!         .when(json(r#"[{"class": "security.kev"}]"#))
//!         .slug(json(r#"{"paths": ["data.cve", "data.id"], "pattern": "^CVE-"}"#))
//!         .class("../exploit.tsx#Exploit")
//!         .props(json(r#"{"cve": "$slug"}"#)),
//! ]);
//! # commandagi::design::declare(security).unwrap();
//! ```

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
    /// A derived stream: an operator folds the records of other monitors into records of this one.
    derive: leaf;
    /// A discover: each thing a record names that has no folder yet gets one, an instance of a class.
    discover: leaf;
}

attributes! {
    /// The fields of a monitor and its parts, chained on their elements.
    pub trait MonitorsAttrs {
        id; title; class; about; readme; same_as;
        adapter; params; every; via; exit; host; mode; retain; since;
        session; snapshot; profile; keep; max_cost_per_day; unit; vantage;
        /// The connected account it reads as (`gmail@jacobfv123@gmail.com`); the api worker (via="worker") reads it.
        connection;
        monitor; match_; prompt;
        op; inputs;
        into_ = "into"; of; when; slug; props; name; max;
    }
}

pub(crate) const FAMILY: Family = Family { module: "monitors", roots: &["monitor"], declare };

pub const MONITOR_FIELDS: &[&str] = &["title", "class", "about", "readme", "sameAs"];
pub const SOURCE_FIELDS: &[&str] = &[
    "id", "adapter", "params", "every", "via", "exit", "host", "mode", "retain", "since", "session", "snapshot", "profile",
    "keep", "maxCostPerDay", "unit", "vantage", "connection",
];
pub const FOLLOW_FIELDS: &[&str] = &["monitor", "match", "when"];
pub const WAKE_FIELDS: &[&str] = &["match", "prompt"];
pub const DERIVE_FIELDS: &[&str] = &["id", "op", "inputs", "params", "since"];
pub const DISCOVER_FIELDS: &[&str] = &["into", "of", "when", "slug", "class", "props", "name", "max"];
/// The most new folders a discover writes a tick.
pub const DISCOVER_MAX: u32 = 50;
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
        TagRule::new("derive", &["monitor"]).key("id").required(&["id", "op", "inputs"]).attrs(DERIVE_FIELDS),
        TagRule::new("discover", &["monitor"]).required(&["slug", "class"]).attrs(DISCOVER_FIELDS),
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
    for k in ["id", "adapter", "exit", "host", "retain", "since", "session", "snapshot", "profile", "keep", "unit", "vantage", "connection"] {
        word(t, k, "id")?;
    }
    let w = call(t, "id");
    if t.attr("connection").is_some() && t.attr("via").and_then(|v| v.as_str()).unwrap_or("worker") != "worker" {
        return Err(format!("{w} connection names a connected account, which the api worker reads (via=\"worker\")"));
    }
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

/// A subject: `<scheme>:<id>` (`cik:0001045810`, `ticker:NVDA`, `fomc:`).
pub fn subject_ok(s: &str) -> bool {
    let Some((scheme, id)) = s.split_once(':') else { return false };
    let b = scheme.as_bytes();
    !b.is_empty()
        && b[0].is_ascii_lowercase()
        && b.iter().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || *c == b'-')
        && !id.contains(char::is_whitespace)
}

fn ident_ok(s: &str) -> bool {
    let b = s.as_bytes();
    !b.is_empty() && (b[0].is_ascii_alphabetic() || b[0] == b'_') && b.iter().all(|c| c.is_ascii_alphanumeric() || *c == b'_')
}

/// A data path: words joined by `.` (`data.cve`, `fromName`).
fn data_path_ok(s: &str) -> bool {
    s.split('.').all(ident_ok)
}

/// A word: letters, digits, `_` and `-`, not starting with `_` or `-`.
fn word_ok(s: &str) -> bool {
    let b = s.as_bytes();
    !b.is_empty() && b[0].is_ascii_alphanumeric() && b.iter().all(|c| c.is_ascii_alphanumeric() || *c == b'_' || *c == b'-')
}

/// A follow's filter: `name=value` terms joined by `,` (`conversation=123`).
/// A class: its file (`.ts`, `.tsx`, `.js`, `.jsx`), `#`, its exported name (`../exploit.tsx#Exploit`).
fn class_ref_ok(s: &str) -> bool {
    let Some((file, name)) = s.split_once('#') else { return false };
    let stem_ok = [".tsx", ".jsx", ".ts", ".js"].iter().any(|e| file.strip_suffix(e).is_some_and(|stem| !stem.is_empty()));
    let n = name.as_bytes();
    stem_ok
        && !file.contains(char::is_whitespace)
        && !n.is_empty()
        && n.len() <= 60
        && n[0].is_ascii_uppercase()
        && n.iter().all(|c| c.is_ascii_alphanumeric() || *c == b'_')
}

const WHEN_OPS: &[&str] = &["eq", "ne", "gt", "gte", "lt", "lte", "in", "glob", "contains", "exists"];

/// The shape of a `when` (the platform reads its meaning: packages/domain/world/predicate.js in the repository).
fn when_shape(when: &Json) -> Option<String> {
    let list: Vec<&Json> = match when {
        Json::Arr(a) => a.iter().collect(),
        one => vec![one],
    };
    if list.is_empty() || list.len() > 16 {
        return Some("one condition, or a list of one to 16".into());
    }
    for c in list {
        let Json::Obj(terms) = c else {
            return Some(format!("a condition is an object of terms ({{\"class\": \"security.kev\"}}), not {}", c.text()));
        };
        for (name, test) in terms {
            let named = matches!(name.as_str(), "class" | "about" | "key" | "title")
                || name.strip_prefix("data.").is_some_and(data_path_ok);
            if !named {
                return Some(format!("\"{name}\" is class, about, key, title or a data path (data.cvss3)"));
            }
            if let Json::Obj(ops) = test {
                if let Some((op, _)) = ops.iter().find(|(op, _)| !WHEN_OPS.contains(&op.as_str())) {
                    return Some(format!("{name}: {op} is not an operator ({})", WHEN_OPS.join(", ")));
                }
            }
        }
    }
    None
}

fn derive_of(t: &DocTree) -> Result<Json, String> {
    for k in ["id", "op", "since"] {
        word(t, k, "id")?;
    }
    let w = call(t, "id");
    if !str_attr(t, "id").is_some_and(word_ok) {
        return Err(format!("{w} id is a word (letters, digits, _ and -)"));
    }
    let Some(Json::Obj(inputs)) = t.attr("inputs").filter(|i| matches!(i, Json::Obj(e) if !e.is_empty())) else {
        let said = t.attr("inputs").map(Json::text).unwrap_or_else(|| "undefined".into());
        return Err(format!("{w} inputs maps each port to a monitor's folder ({{\"quotes\": \"../quotes\"}}), not {said}"));
    };
    for (port, folder) in inputs {
        if !word_ok(port) {
            return Err(format!("{w} a port is a word, not \"{port}\""));
        }
        match folder {
            Json::Str(f) if !f.is_empty() => {}
            other => return Err(format!("{w} inputs.{port} is a monitor's folder, not {}", other.text())),
        }
    }
    if let Some(p) = t.attr("params").filter(|p| !matches!(p, Json::Null | Json::Obj(_))) {
        return Err(format!("{w} params is an object ({{\"window\": \"1m\"}}), not {}", p.text()));
    }
    if let Some(d) = str_attr(t, "since") {
        if !day_ok(d) {
            return Err(format!("{w} since is a day (\"2024-01-01\"), not \"{d}\""));
        }
    }
    Ok(said(t, DERIVE_FIELDS))
}

fn discover_of(t: &DocTree) -> Result<Json, String> {
    for k in ["into", "class", "name"] {
        word(t, k, "into")?;
    }
    let w = format!("discover().into(\"{}\")", str_attr(t, "into").unwrap_or("./"));
    let shown = |k: &str| t.attr(k).map(Json::text).unwrap_or_else(|| "undefined".into());
    let of_ok = match t.attr("of") {
        None | Some(Json::Null) => true,
        Some(Json::Str(s)) => !s.is_empty(),
        Some(Json::Arr(a)) => !a.is_empty() && a.iter().all(|x| matches!(x, Json::Str(s) if !s.is_empty())),
        _ => false,
    };
    if !of_ok {
        return Err(format!("{w} of is a monitor's folder or a list of them ([\"../kev\", \"../cve-feed\"]), not {}", shown("of")));
    }
    if let Some(when) = t.attr("when").filter(|v| !matches!(v, Json::Null)) {
        if let Some(bad) = when_shape(when) {
            return Err(format!("{w} when: {bad}"));
        }
    }
    let slug_ok = match t.attr("slug") {
        Some(Json::Str(s)) => data_path_ok(s),
        Some(slug @ Json::Obj(entries)) => {
            matches!(slug.get("paths"), Some(Json::Arr(p)) if !p.is_empty() && p.iter().all(|x| x.as_str().is_some_and(data_path_ok)))
                && entries.iter().all(|(k, _)| k == "paths" || k == "pattern")
        }
        _ => false,
    };
    if !slug_ok {
        return Err(format!("{w} slug is a data path (data.cve) or {{ paths: [...], pattern }}, not {}", shown("slug")));
    }
    if !str_attr(t, "class").is_some_and(class_ref_ok) {
        return Err(format!("{w} class is the class file and its exported name (\"../exploit.tsx#Exploit\"), not {}", shown("class")));
    }
    if str_attr(t, "name").is_some_and(|n| !data_path_ok(n)) {
        return Err(format!("{w} name is a data path (fromName), not {}", shown("name")));
    }
    match t.attr("props") {
        None | Some(Json::Null) => {}
        Some(Json::Obj(props)) => {
            for (k, v) in props {
                let kb = k.as_bytes();
                let key_ok = !kb.is_empty() && kb[0].is_ascii_lowercase() && kb.iter().all(u8::is_ascii_alphanumeric);
                if !key_ok || !v.as_str().is_some_and(|p| p == "$slug" || data_path_ok(p)) {
                    return Err(format!("{w} props.{k} is a data path or \"$slug\", not {}", v.text()));
                }
            }
        }
        Some(other) => return Err(format!("{w} props maps each prop to a data path ({{\"conversation\": \"conversation\"}}), not {}", other.text())),
    }
    if let Some(m) = t.attr("max").filter(|v| !matches!(v, Json::Null)) {
        if !matches!(m, Json::Num(n) if n.fract() == 0.0 && *n >= 1.0 && *n <= DISCOVER_MAX as f64) {
            return Err(format!("{w} max is a whole number from 1 to {DISCOVER_MAX}, not {}", m.text()));
        }
    }
    Ok(said(t, DISCOVER_FIELDS))
}

fn monitor_of(t: &DocTree) -> Result<Json, String> {
    for k in MONITOR_FIELDS.iter().filter(|k| **k != "sameAs") {
        word(t, k, "title")?;
    }
    match t.attr("sameAs") {
        None | Some(Json::Null) => {}
        Some(Json::Arr(a)) if a.iter().all(|s| s.as_str().is_some_and(subject_ok)) => {}
        Some(v) => return Err(format!("monitor() sameAs is a list of subjects (\"ticker:NVDA\"), not {}", v.text())),
    }
    let mut sources = Vec::new();
    let mut follows = Vec::new();
    let mut derives = Vec::new();
    let mut wakes = Vec::new();
    let mut discovers = Vec::new();
    for c in &t.children {
        match c.tag {
            "source" => sources.push(source_of(c)?),
            "follows" => {
                for k in ["monitor", "match"] {
                    word(c, k, "monitor")?;
                }
                if let Some(when) = c.attr("when").filter(|v| !matches!(v, Json::Null)) {
                    if let Some(bad) = when_shape(when) {
                        return Err(format!("{} when: {bad}", call(c, "monitor")));
                    }
                }
                if matches!(c.attr("monitor"), Some(Json::Str(m)) if m == "." || m == "./") {
                    return Err(format!("{} names this monitor; a follow names another monitor's folder", call(c, "monitor")));
                }
                follows.push(said(c, FOLLOW_FIELDS));
            }
            "derive" => derives.push(derive_of(c)?),
            "discover" => discovers.push(discover_of(c)?),
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
    out.set("derives", Json::Arr(derives));
    out.set("wakes", Json::Arr(wakes));
    out.set("discovers", Json::Arr(discovers));
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
            for k in ["since", "session", "snapshot", "profile", "keep", "maxCostPerDay", "unit", "vantage", "connection"] {
                out.set(k, or_null(s, k));
            }
            out
        })
        .collect();
    let follows = list("follows")
        .iter()
        .map(|f| Json::obj().with("monitor", or_null(f, "monitor")).with("match", or_null(f, "match")).with("when", or_null(f, "when")))
        .collect();
    let derives = list("derives")
        .iter()
        .map(|d| {
            Json::obj()
                .with("id", or_null(d, "id"))
                .with("op", or_null(d, "op"))
                .with("inputs", or_null(d, "inputs"))
                .with("params", d.get("params").cloned().unwrap_or_else(Json::obj))
                .with("since", or_null(d, "since"))
        })
        .collect();
    let wakes = list("wakes").iter().map(|w| Json::obj().with("match", or_null(w, "match")).with("prompt", or_null(w, "prompt"))).collect();
    let discovers = list("discovers")
        .iter()
        .map(|d| {
            let of = match d.get("of") {
                None | Some(Json::Null) => Json::Arr(Vec::new()),
                Some(s @ Json::Str(_)) => Json::Arr(vec![s.clone()]),
                Some(a) => a.clone(),
            };
            Json::obj()
                .with("into", d.get("into").cloned().unwrap_or_else(|| Json::Str("./".into())))
                .with("of", of)
                .with("when", or_null(d, "when"))
                .with("slug", or_null(d, "slug"))
                .with("class", or_null(d, "class"))
                .with("props", d.get("props").cloned().unwrap_or_else(Json::obj))
                .with("name", or_null(d, "name"))
                .with("max", d.get("max").cloned().unwrap_or(Json::Num(10.0)))
        })
        .collect();
    let mut out = Json::obj();
    for k in MONITOR_FIELDS {
        out.set(k, if *k == "sameAs" { m.get(k).cloned().unwrap_or_else(|| Json::Arr(Vec::new())) } else { or_null(m, k) });
    }
    out.set("sources", Json::Arr(sources));
    out.set("follows", Json::Arr(follows));
    out.set("derives", Json::Arr(derives));
    out.set("wakes", Json::Arr(wakes));
    out.set("discovers", Json::Arr(discovers));
    out
}
