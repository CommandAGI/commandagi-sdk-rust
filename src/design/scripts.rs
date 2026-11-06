//! SCRIPTS IN RUST — a `<name>.script.rs`: one script, anywhere in a Drive (docs/scripts.md in the CommandAGI
//! repository). A script declares what it reads (`source()`) and the files it tails (`tail()`, `derive()`, `wake()`,
//! `discover()`); its run, beside the file, holds the records its parts append. Its id is the file's own path in the
//! account's Drive, so the file never names itself, and a folder may hold several scripts. The same elements, and the
//! same document, as the TypeScript SDK's `scripts.ts`. A class of things is a function; an instance is one call:
//!
//! ```
//! use commandagi::design::scripts::*;
//!
//! fn corporation(cik: &str, ticker: &str, name: &str) -> El {
//!     script([
//!         source().id("filings").adapter("sec-filings").params(json(&format!(r#"{{"cik": "{cik}"}}"#))).every("30m"),
//!         tail().of("../../central-banks/fed/fed.script.tsx").when(json(r#"{"class": "fomc.statement"}"#)),
//!     ])
//!     .title(format!("{name} ({ticker})"))
//!     .class("corporation")
//!     .about(format!("cik:{cik}"))
//! }
//! # commandagi::design::declare(corporation("0001045810", "NVDA", "NVIDIA")).unwrap();
//! ```
//!
//! A tail names the file it tails (`of`: a ref from this script's folder to a script file, a run's `records.jsonl`, or a
//! statistics series by its id, `fred:DGS10`), never a folder, and takes the records its `when` (a record predicate)
//! holds for. A wake wakes the agent on the script's own records (`when`) or on a clock (`every`), never both. `stream`
//! names the stream of the run a part appends to (absent: `records`). A discover's `into` is written `into_`
//! (`Into::into` has the name). Nothing adds a default: what the file does not say, the document does not have;
//! [`script_with_defaults`] gives the document as the platform reads it. A script's `sameAs`, `sources`, `tails`,
//! `derives`, `wakes`, `discovers`, `judges` and `params` are always lists. A script with `param()` parts is a
//! template: each `${id}` placeholder is filled when it is copied, and a placeholder as a whole value passes the checks
//! of the file it will name.
//!
//! A `derive` folds the records of other files into this script's, each port named to a file it tails (`"./"`: its own
//! records); a `discover` gives each thing its records name a folder and a script of its own, an instance of a class
//! (`<file>#<Component>`):
//!
//! ```
//! use commandagi::design::scripts::*;
//!
//! let security = script([
//!     derive().id("spread").op("spread").inputs(json(r#"{"a": "fred:SOFR", "b": "fred:IORB"}"#)).stream("derived"),
//!     discover()
//!         .of(json(r#"["./kev/kev.script.tsx", "./cve-feed/cve-feed.script.tsx"]"#))
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
    /// A script: one file, with its sources, its tails and its wakes.
    script: children;
    /// A source: an adapter of the sources catalog, its params, and how often it reads.
    source: leaf;
    /// A tail: take the records of another file (a script, a run's records, a series) that its `when` holds for.
    tail: leaf;
    /// A wake: wake the agent on the script's own records (`when`), or on a clock (`every`: a weekly brief).
    wake: leaf;
    /// A derived stream: an operator folds the records of other files into records of this script.
    derive: leaf;
    /// A discover: each thing a record names that has no folder yet gets one, an instance of a class.
    discover: leaf;
    /// A judge: when a turn or a task's turn ends, a model reads the trace with a rubric and records a verdict.
    judge: leaf;
    /// A template's parameter: its `${id}` placeholders are filled when the template is copied.
    param: leaf;
}

attributes! {
    /// The fields of a script and its parts, chained on their elements.
    pub trait ScriptsAttrs {
        id; title; class; about; readme; same_as;
        adapter; params; every; via; exit; host; mode; retain; since;
        session; snapshot; profile; keep; max_cost_per_day; unit; vantage;
        /// The connected account it reads as (`gmail@jacobfv123@gmail.com`); the api worker (via="worker") reads it.
        connection;
        /// The stream of the run a part appends to (absent: `records`).
        stream;
        prompt; agent; max_spend;
        op; inputs;
        into_ = "into"; of; when; slug; props; name; max; visibility;
        rubric; model;
        /// A param's.
        type_; label; default; required; choices; accept;
    }
}

pub(crate) const FAMILY: Family = Family { module: "scripts", roots: &["script"], declare };

pub const SCRIPT_FIELDS: &[&str] = &["title", "class", "about", "readme", "sameAs"];
pub const SOURCE_FIELDS: &[&str] = &[
    "id", "adapter", "params", "every", "via", "exit", "host", "mode", "retain", "since", "session", "snapshot", "profile",
    "keep", "maxCostPerDay", "unit", "vantage", "connection", "stream",
];
pub const TAIL_FIELDS: &[&str] = &["of", "when", "stream"];
pub const WAKE_FIELDS: &[&str] = &["when", "every", "prompt", "agent", "maxSpend"];
pub const DERIVE_FIELDS: &[&str] = &["id", "op", "inputs", "params", "since", "stream"];
pub const DISCOVER_FIELDS: &[&str] = &["into", "of", "when", "slug", "class", "props", "name", "max", "visibility"];
pub const JUDGE_FIELDS: &[&str] = &["id", "of", "rubric", "model"];
/// What a judge judges: each turn of the thread's agents, or each turn on a mounted task.
pub const JUDGE_OF: &[&str] = &["turn", "task"];
/// A template's parameter (docs/scripts.md § the catalogue): its `${id}` placeholders are filled when it is copied.
pub const SCRIPT_PARAM_FIELDS: &[&str] = &["id", "type", "label", "default", "required", "choices", "accept"];
/// What a parameter is: the same types a dashboard template's parameters have.
pub const SCRIPT_PARAM_TYPES: &[&str] = &["file", "device", "world", "unit", "channel", "run", "stream", "number", "string", "choice"];
/// The most new folders a discover writes a tick.
pub const DISCOVER_MAX: u32 = 50;
/// Who may read a file a discover writes (docs/scripts.md § a script file).
pub const VISIBILITIES: &[&str] = &["private", "unlisted", "public"];
/// The paths a source's reads take (docs/sources.md § fetch paths).
pub const VIAS: &[&str] = &["worker", "host", "vpn", "browser", "vm", "android-vm", "phone"];
/// The paths that launch a machine from a snapshot (a logged-in state), and so cost money per read.
pub const VM_VIAS: &[&str] = &["vm", "android-vm"];
/// `poll` reads at each `every`; `stream` holds a connection open on a host.
pub const MODES: &[&str] = &["poll", "stream"];
/// The shortest and the longest `every` a source may have, in seconds (30s and 7d).
pub const EVERY_BOUNDS: (u64, u64) = (30, 7 * 86_400);

const SCRIPT: Vocabulary = Vocabulary {
    format: "script",
    noun: "a script",
    root: "script",
    tags: &[
        TagRule::new("script", &[]).attrs(SCRIPT_FIELDS),
        TagRule::new("source", &["script"]).key("id").required(&["id", "adapter", "every"]).attrs(SOURCE_FIELDS),
        TagRule::new("tail", &["script"]).key("of").required(&["of"]).attrs(TAIL_FIELDS),
        TagRule::new("derive", &["script"]).key("id").required(&["id", "op", "inputs"]).attrs(DERIVE_FIELDS),
        TagRule::new("wake", &["script"]).required(&["prompt"]).attrs(WAKE_FIELDS),
        TagRule::new("discover", &["script"]).required(&["slug", "class"]).attrs(DISCOVER_FIELDS),
        TagRule::new("judge", &["script"]).key("id").required(&["id", "of", "rubric", "model"]).attrs(JUDGE_FIELDS),
        TagRule::new("param", &["script"]).key("id").required(&["id", "type"]).attrs(SCRIPT_PARAM_FIELDS),
    ],
    // A tail names a file by its ref: the platform checks that the file is there and has a run.
    from_tree: script_of,
};

fn declare(root: El) -> Result<Declared, String> {
    declare_document(&root, &SCRIPT)
}

/// A script's file name: `<name>.script.tsx` (or `.jsx`, `.py`, `.rs`), anywhere in a Drive.
pub fn script_file_ok(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    !name.contains('/')
        && [".script.tsx", ".script.jsx", ".script.py", ".script.rs"].iter().any(|e| lower.strip_suffix(e).is_some_and(|stem| !stem.is_empty()))
}

/// A stream of the run a part appends to (`records`, `ticks`): a lowercase word of at most 40 characters.
pub fn stream_name_ok(s: &str) -> bool {
    let b = s.as_bytes();
    !b.is_empty() && b.len() <= 40 && b[0].is_ascii_lowercase() && b.iter().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || *c == b'-')
}

/// A statistics series by its id (`fred:DGS10`): a scheme and no `/`.
fn series_ok(r: &str) -> bool {
    let Some((scheme, id)) = r.split_once(':') else { return false };
    let b = scheme.as_bytes();
    let first_ok = id.chars().next().is_some_and(|c| !c.is_whitespace() && c != '/' && c != ':');
    !b.is_empty()
        && b[0].is_ascii_lowercase()
        && b.iter().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || *c == b'-')
        && first_ok
        && !id.contains(|c: char| c.is_whitespace() || c == '/')
}

/// A template's parameter as a whole value: `${of}`.
fn placeholder_ok(r: &str) -> bool {
    r.strip_prefix("${").and_then(|x| x.strip_suffix('}')).is_some_and(ident_ok)
}

/// A tailed file: a script file or a run's records file, by a ref or an address.
fn tailed_file_ok(r: &str) -> bool {
    let last = r.rsplit('/').next().unwrap_or(r);
    script_file_ok(last) || last.eq_ignore_ascii_case("records.jsonl")
}

/// Why a ref cannot be what a tail (a derive's port, a discover's `of`) reads, or none: it names a file, never a
/// folder. The TypeScript SDK's `tailRefProblem`.
pub fn tail_ref_problem(r: &str) -> Option<String> {
    // A template's placeholder is a file once its copy fills it (docs/scripts.md § the catalogue).
    if series_ok(r) || tailed_file_ok(r) || placeholder_ok(r) {
        return None;
    }
    if r == "." || r == "./" {
        return Some("names this script's folder; a tail names another file".into());
    }
    Some(format!(
        "names a file: a script (\"../nvidia/nvidia.script.tsx\"), a run's records.jsonl, or a series (\"fred:DGS10\"), not {}",
        Json::Str(r.into()).text()
    ))
}

/// A part's `stream`, when it says one: a word.
fn stream_of(t: &DocTree) -> Result<(), String> {
    match t.attr_of("stream") {
        None | Some(Json::Null) => Ok(()),
        Some(Json::Str(s)) if stream_name_ok(s) => Ok(()),
        Some(v) => Err(format!("{} stream is a word (\"records\", \"ticks\"), not {}", call(t, if t.tag == "tail" { "of" } else { "id" }), v.text())),
    }
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
    match t.attr_of(key) {
        Some(v @ Json::Str(_)) => format!("{}().{}({})", fn_name(t.tag), fn_name(key), v.text()),
        _ => format!("{}()", fn_name(t.tag)),
    }
}

fn word(t: &DocTree, k: &str, key: &str) -> Result<(), String> {
    match t.attr_of(k) {
        None | Some(Json::Null) => Ok(()),
        Some(Json::Str(s)) if !s.is_empty() => Ok(()),
        Some(v) => Err(format!("{} {} is a word, not {}", call(t, key), fn_name(k), v.text())),
    }
}

fn one_of(t: &DocTree, k: &str, values: &[&str]) -> Result<(), String> {
    match t.attr_of(k) {
        None | Some(Json::Null) => Ok(()),
        Some(Json::Str(s)) if values.contains(&s.as_str()) => Ok(()),
        Some(v) => Err(format!("{} {} is {}, not {}", call(t, "id"), k, values.iter().map(|x| format!("\"{x}\"")).collect::<Vec<_>>().join(", "), v.text())),
    }
}

/// The fields an element says, in the vocabulary's order.
fn said(t: &DocTree, keys: &[&str]) -> Json {
    let mut out = Json::obj();
    for k in keys {
        if let Some(v) = t.attr_of(k).filter(|v| !matches!(v, Json::Null)) {
            out.set(k, v.clone());
        }
    }
    out
}

fn str_attr<'a>(t: &'a DocTree, k: &str) -> Option<&'a str> {
    match t.attr_of(k) {
        Some(Json::Str(s)) => Some(s.as_str()),
        _ => None,
    }
}

fn has(t: &DocTree, k: &str) -> bool {
    !matches!(t.attr_of(k), None | Some(Json::Null))
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
    if t.attr_of("connection").is_some() && t.attr_of("via").and_then(|v| v.as_str()).unwrap_or("worker") != "worker" {
        return Err(format!("{w} connection names a connected account, which the api worker reads (via=\"worker\")"));
    }
    if let Some(p) = t.attr_of("params") {
        if !matches!(p, Json::Obj(_)) {
            return Err(format!("{w} params is an object ({{\"series\": \"CPIAUCSL\"}}), not {}", p.text()));
        }
    }
    let every = t.attr_of("every");
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
    let cost = t.attr_of("maxCostPerDay").filter(|c| !matches!(c, Json::Null));
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
    let Some(Json::Obj(inputs)) = t.attr_of("inputs").filter(|i| matches!(i, Json::Obj(e) if !e.is_empty())) else {
        let said = t.attr_of("inputs").map(Json::text).unwrap_or_else(|| "undefined".into());
        return Err(format!("{w} inputs maps each port to a file it tails ({{\"quotes\": \"../quotes/quotes.script.tsx\"}}), not {said}"));
    };
    for (port, r) in inputs {
        if !word_ok(port) {
            return Err(format!("{w} a port is a word, not \"{port}\""));
        }
        let Json::Str(r) = r.clone() else { return Err(format!("{w} inputs.{port} is a file it tails, not {}", r.text())) };
        if r.is_empty() {
            return Err(format!("{w} inputs.{port} is a file it tails, not \"\""));
        }
        // A derive may fold its own script's records: "./".
        if r != "." && r != "./" {
            if let Some(bad) = tail_ref_problem(&r) {
                return Err(format!("{w} inputs.{port} {bad}"));
            }
        }
    }
    if let Some(p) = t.attr_of("params").filter(|p| !matches!(p, Json::Null | Json::Obj(_))) {
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
    let shown = |k: &str| t.attr_of(k).map(Json::text).unwrap_or_else(|| "undefined".into());
    let refs: Option<Vec<&str>> = match t.attr_of("of") {
        None | Some(Json::Null) => Some(Vec::new()),
        Some(Json::Str(s)) if !s.is_empty() => Some(vec![s.as_str()]),
        Some(Json::Arr(a)) if !a.is_empty() => a.iter().map(|x| x.as_str().filter(|s| !s.is_empty())).collect(),
        _ => None,
    };
    let Some(refs) = refs else {
        return Err(format!("{w} of is a script file or a list of them ([\"../kev/kev.script.tsx\"]), not {}", shown("of")));
    };
    for r in refs {
        if let Some(bad) = tail_ref_problem(r) {
            return Err(format!("{w} of {bad}"));
        }
    }
    if let Some(when) = t.attr_of("when").filter(|v| !matches!(v, Json::Null)) {
        if let Some(bad) = when_shape(when) {
            return Err(format!("{w} when: {bad}"));
        }
    }
    let slug_ok = match t.attr_of("slug") {
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
    if !str_attr(t, "class").is_some_and(|c| class_ref_ok(c) || placeholder_ok(c)) {
        return Err(format!("{w} class is the class file and its exported name (\"../exploit.tsx#Exploit\"), not {}", shown("class")));
    }
    if str_attr(t, "name").is_some_and(|n| !data_path_ok(n)) {
        return Err(format!("{w} name is a data path (fromName), not {}", shown("name")));
    }
    match t.attr_of("props") {
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
    if let Some(m) = t.attr_of("max").filter(|v| !matches!(v, Json::Null)) {
        if !matches!(m, Json::Num(n) if n.fract() == 0.0 && *n >= 1.0 && *n <= DISCOVER_MAX as f64) {
            return Err(format!("{w} max is a whole number from 1 to {DISCOVER_MAX}, not {}", m.text()));
        }
    }
    if let Some(v) = t.attr_of("visibility").filter(|v| !matches!(v, Json::Null)) {
        if !v.as_str().is_some_and(|s| VISIBILITIES.contains(&s)) {
            return Err(format!("{w} visibility is {}, not {}", VISIBILITIES.join(", "), v.text()));
        }
    }
    Ok(said(t, DISCOVER_FIELDS))
}

fn tail_of(c: &DocTree) -> Result<Json, String> {
    word(c, "of", "of")?;
    let w = call(c, "of");
    if let Some(bad) = c.attr_of("when").filter(|v| !matches!(v, Json::Null)).and_then(when_shape) {
        return Err(format!("{w} when: {bad}"));
    }
    if let Some(bad) = str_attr(c, "of").and_then(tail_ref_problem) {
        return Err(format!("{w} {bad}"));
    }
    Ok(said(c, TAIL_FIELDS))
}

/// A wake on the script's own records (`when`, a record predicate) or on a clock (`every`, from 30s to 7d): never both.
fn wake_of(c: &DocTree) -> Result<Json, String> {
    word(c, "prompt", "every")?;
    word(c, "agent", "every")?;
    let w = call(c, "every");
    if let Some(spend) = c.attr_of("maxSpend").filter(|v| !matches!(v, Json::Null)) {
        if !matches!(spend, Json::Num(n) if n.is_finite() && *n > 0.0) {
            return Err(format!("{w} maxSpend is a price in USD (0.5), not {}", spend.text()));
        }
    }
    let when = c.attr_of("when").filter(|v| !matches!(v, Json::Null));
    if let Some(bad) = when.and_then(when_shape) {
        return Err(format!("{w} when: {bad}"));
    }
    if let Some(every) = c.attr_of("every").filter(|v| !matches!(v, Json::Null)) {
        if when.is_some() {
            return Err(format!("{w} wakes on records (when) or on a clock (every), not both"));
        }
        let ok = matches!(every, Json::Str(s) if duration_seconds(s).is_some_and(|n| n >= EVERY_BOUNDS.0 && n <= EVERY_BOUNDS.1));
        if !ok {
            return Err(format!("wake() every is a duration from 30s to 7d, not {}", every.text()));
        }
    }
    Ok(said(c, WAKE_FIELDS))
}

fn script_of(t: &DocTree) -> Result<Json, String> {
    for k in SCRIPT_FIELDS.iter().filter(|k| **k != "sameAs") {
        word(t, k, "title")?;
    }
    match t.attr_of("sameAs") {
        None | Some(Json::Null) => {}
        Some(Json::Arr(a)) if a.iter().all(|s| s.as_str().is_some_and(subject_ok)) => {}
        Some(v) => return Err(format!("script() sameAs is a list of subjects (\"ticker:NVDA\"), not {}", v.text())),
    }
    for c in &t.children {
        stream_of(c)?;
    }
    let mut sources = Vec::new();
    let mut tails = Vec::new();
    let mut derives = Vec::new();
    let mut wakes = Vec::new();
    let mut discovers = Vec::new();
    let mut judges = Vec::new();
    let mut params: Vec<Json> = Vec::new();
    for c in &t.children {
        match c.tag {
            "source" => sources.push(source_of(c)?),
            "tail" => tails.push(tail_of(c)?),
            "derive" => derives.push(derive_of(c)?),
            "wake" => wakes.push(wake_of(c)?),
            "discover" => discovers.push(discover_of(c)?),
            "judge" => judges.push(judge_of(c)?),
            _ => {
                let p = param_of(c)?;
                if params.iter().any(|q| q.get("id") == p.get("id")) {
                    return Err(format!("{} is declared twice", call(c, "id")));
                }
                params.push(p);
            }
        }
    }
    let mut out = said(t, SCRIPT_FIELDS);
    out.set("sources", Json::Arr(sources));
    out.set("tails", Json::Arr(tails));
    out.set("derives", Json::Arr(derives));
    out.set("wakes", Json::Arr(wakes));
    out.set("discovers", Json::Arr(discovers));
    out.set("judges", Json::Arr(judges));
    out.set("params", Json::Arr(params));
    Ok(out)
}

/// A template's parameter, as a dashboard template's `param()` is.
fn param_of(t: &DocTree) -> Result<Json, String> {
    let w = call(t, "id");
    let id_ok = str_attr(t, "id").is_some_and(|i| i.len() <= 40 && ident_ok(i));
    if !id_ok {
        let shown = t.attr_of("id").map(Json::text).unwrap_or_else(|| "undefined".into());
        return Err(format!("{w} id is letters, digits and _, not {shown}"));
    }
    one_of(t, "type", SCRIPT_PARAM_TYPES)?;
    word(t, "label", "id")?;
    if let Some(d) = t.attr_of("default").filter(|v| !matches!(v, Json::Null)) {
        if !matches!(d, Json::Str(_)) && !matches!(d, Json::Num(n) if n.is_finite()) {
            return Err(format!("{w} default is a text or a number, not {}", d.text()));
        }
    }
    if t.attr_of("required").is_some_and(|v| !matches!(v, Json::Null | Json::Bool(_))) {
        return Err(format!("{w} required is true or false"));
    }
    for k in ["choices", "accept"] {
        if let Some(v) = t.attr_of(k).filter(|v| !matches!(v, Json::Null)) {
            if !matches!(v, Json::Arr(a) if !a.is_empty() && a.iter().all(|x| x.as_str().is_some_and(|s| !s.is_empty()))) {
                return Err(format!("{w} {k} is a list of texts"));
            }
        }
    }
    if str_attr(t, "type") == Some("choice") && !has(t, "choices") {
        return Err(format!("{w} a choice names its choices"));
    }
    Ok(said(t, SCRIPT_PARAM_FIELDS))
}

/// A judge of the script's run (docs/scripts.md § how a trigger runs a script): when a span of the thread's trace ends,
/// the model reads it with the rubric, and its verdict is a record of the thread (an opinion, never evidence).
fn judge_of(t: &DocTree) -> Result<Json, String> {
    for k in ["id", "rubric", "model"] {
        word(t, k, "id")?;
    }
    let w = call(t, "id");
    if !str_attr(t, "id").is_some_and(word_ok) {
        return Err(format!("{w} id is a word (letters, digits, _ and -)"));
    }
    one_of(t, "of", JUDGE_OF)?;
    if !str_attr(t, "rubric").is_some_and(|r| r.to_ascii_lowercase().ends_with(".md")) {
        let shown = t.attr_of("rubric").map(Json::text).unwrap_or_else(|| "undefined".into());
        return Err(format!("{w} rubric is a .md file (\"./rubric.md\"), not {shown}"));
    }
    Ok(said(t, JUDGE_FIELDS))
}

/// A script as the platform reads it: what the file does not say, filled (`via: "worker"`, `mode: "poll"`, `stream:
/// "records"`, the others null; a stream source's `retain` "derived"). The TypeScript SDK's `scriptWithDefaults`.
pub fn script_with_defaults(m: &Json) -> Json {
    let or_null = |o: &Json, k: &str| o.get(k).cloned().unwrap_or(Json::Null);
    let records = |o: &Json| o.get("stream").cloned().unwrap_or_else(|| Json::Str("records".into()));
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
            out.set("stream", records(s));
            out
        })
        .collect();
    let tails = list("tails")
        .iter()
        .map(|f| Json::obj().with("of", or_null(f, "of")).with("when", or_null(f, "when")).with("stream", records(f)))
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
                .with("stream", records(d))
        })
        .collect();
    let wakes = list("wakes")
        .iter()
        .map(|w| {
            Json::obj()
                .with("when", or_null(w, "when"))
                .with("every", or_null(w, "every"))
                .with("prompt", or_null(w, "prompt"))
                .with("agent", or_null(w, "agent"))
                .with("maxSpend", or_null(w, "maxSpend"))
        })
        .collect();
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
                .with("visibility", or_null(d, "visibility"))
        })
        .collect();
    let judges = list("judges")
        .iter()
        .map(|j| Json::obj().with("id", or_null(j, "id")).with("of", or_null(j, "of")).with("rubric", or_null(j, "rubric")).with("model", or_null(j, "model")))
        .collect();
    let params = list("params")
        .iter()
        .map(|p| {
            Json::obj()
                .with("id", or_null(p, "id"))
                .with("type", or_null(p, "type"))
                .with("label", or_null(p, "label"))
                .with("default", or_null(p, "default"))
                .with("required", p.get("required").cloned().unwrap_or(Json::Bool(true)))
                .with("choices", or_null(p, "choices"))
                .with("accept", or_null(p, "accept"))
        })
        .collect();
    let mut out = Json::obj();
    for k in SCRIPT_FIELDS {
        out.set(k, if *k == "sameAs" { m.get(k).cloned().unwrap_or_else(|| Json::Arr(Vec::new())) } else { or_null(m, k) });
    }
    out.set("sources", Json::Arr(sources));
    out.set("tails", Json::Arr(tails));
    out.set("derives", Json::Arr(derives));
    out.set("wakes", Json::Arr(wakes));
    out.set("discovers", Json::Arr(discovers));
    out.set("judges", Json::Arr(judges));
    out.set("params", Json::Arr(params));
    out
}
