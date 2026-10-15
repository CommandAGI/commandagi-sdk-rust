//! AGENTS IN RUST — a `harness.rs` and an `agent.rs`: the same elements, and the same documents, as the TypeScript
//! SDK's JSX (`agents.ts`) and the Python SDK's `commandagi.design.agents` (docs/agents.md in the CommandAGI repository):
//!
//! ```
//! use commandagi::design::agents::*;
//!
//! // cloud://global/agents/harnesses/openai-realtime/harness.rs — a model that converses live
//! fn document() -> El {
//!     harness()
//!         .kind("api")
//!         .label("OpenAI Realtime API")
//!         .provider("openai")
//!         .runs(["worker", "host"])
//!         .realtime(json(r#"{"in": ["audio", "video", "text"], "out": ["audio", "text", "tools"],
//!             "audio": {"in": ["g711-ulaw@8000", "pcm16@24000"], "out": ["g711-ulaw@8000", "pcm16@24000"]},
//!             "fps": 1, "hz": 2, "minutes": 60}"#))
//! }
//! # commandagi::design::declare(document()).unwrap();
//! ```
//!
//! ```
//! use commandagi::design::agents::*;
//!
//! // agents/secretary/agent.rs in an account's Drive — a fork
//! fn document() -> El {
//!     agent().parent("cloud://global/agents/claude-code").name("Joe's secretary").instructions("Keep Joe's calendar.")
//! }
//! # commandagi::design::declare(document()).unwrap();
//! ```
//!
//! A harness is the program that runs an agent and talks to a model: a CLI agent, an API provider's loop, or a program
//! with no model. A base instance names its harness and has no parent; every other instance is a fork: it names its
//! parent and keeps its parent's harness. A file never names itself: a harness's id and an agent's id are their
//! folders' names. `realtime` and `persona` are objects, written as [`json`] with the TypeScript names as keys
//! (`"videoMinutes"`). Nothing adds a default: what the file does not say, the document does not have.

use super::documents::{declare_document, DocTree, TagRule, Vocabulary};
use super::element::{attributes, elements, rust_name};
use super::{Declared, Family};
pub use super::{fragment, json, El, Json, Value};

elements! {
    /// A harness: `.kind("cli" | "api" | "program").label(…)` and what its kind needs.
    harness: leaf;
    /// An agent: a base (`.harness("claude-code")`) or a fork (`.parent("cloud://global/agents/claude-code")`).
    agent: leaf;
}

attributes! {
    /// The fields of a harness and of an agent, chained on their elements.
    pub trait AgentsAttrs {
        kind; label; provider;
        /// `kind("cli")` only: the program the host or the VM kernel starts.
        cli;
        /// Where it runs today: `["host", "vm", "worker"]`; absent or empty: declared only.
        runs;
        /// `kind("api")` only: the tasks its model does, one in a turn.
        tasks;
        /// `kind("api")` with no tasks: what its model takes and gives live, as [`json`].
        realtime;
        family; experimental; homepage; about;
        /// A base instance's harness (a harness's id).
        harness;
        /// A fork's parent: a ref to the parent agent's folder.
        parent;
        name; model; effort; instructions; voice; voice_mode; tools; persona; color; icon; gender;
    }
}

pub(crate) const FAMILY: Family = Family { module: "agents", roots: &["harness", "agent"], declare };

/// What runs an agent: a CLI program, an API provider's loop, or a program with no model.
pub const HARNESS_KINDS: &[&str] = &["cli", "api", "program"];
/// Where a harness runs: this machine's host, a cloud computer (the thread's VM kernel), or the api worker's loop.
pub const HARNESS_RUNTIMES: &[&str] = &["host", "vm", "worker"];
/// What a single-task model does in one turn.
pub const HARNESS_TASKS: &[&str] = &["classify", "decide", "extract", "summarize", "act"];
/// What a realtime harness's model takes live, and gives live (docs/realtime-agents.md).
pub const REALTIME_IN: &[&str] = &["audio", "video", "text"];
pub const REALTIME_OUT: &[&str] = &["audio", "text", "tools"];
/// G.711 μ-law (a telephone call) and 16-bit little-endian PCM, mono, at a rate.
pub const AUDIO_FORMATS: &[&str] = &["g711-ulaw@8000", "pcm16@8000", "pcm16@16000", "pcm16@24000"];
pub const HARNESS_FIELDS: &[&str] = &["kind", "label", "provider", "cli", "runs", "tasks", "realtime", "family", "experimental", "homepage", "about"];
pub const AGENT_FIELDS: &[&str] =
    &["harness", "parent", "name", "model", "effort", "instructions", "voice", "voiceMode", "tools", "persona", "color", "icon", "gender"];
pub const VOICE_MODES: &[&str] = &["off", "push-to-talk", "open-mic"];
pub const REALTIME_FIELDS: &[&str] = &["in", "out", "audio", "fps", "hz", "minutes", "videoMinutes"];

const HARNESS: Vocabulary = Vocabulary {
    format: "harness",
    noun: "an agent harness",
    root: "harness",
    tags: &[TagRule::new("harness", &[]).required(&["kind", "label"]).attrs(HARNESS_FIELDS)],
    from_tree: harness_of,
};

const AGENT: Vocabulary = Vocabulary { format: "agent", noun: "an agent", root: "agent", tags: &[TagRule::new("agent", &[]).attrs(AGENT_FIELDS)], from_tree: agent_of };

fn declare(root: El) -> Result<Declared, String> {
    declare_document(&root, if root.tag == "harness" { &HARNESS } else { &AGENT })
}

fn quoted(values: &[&str]) -> String {
    values.iter().map(|v| format!("\"{v}\"")).collect::<Vec<_>>().join(", ")
}

/// A list of distinct values, each one of `allowed` (and not empty, unless `empty`).
fn list_of(v: Option<&Json>, allowed: &[&str], empty: bool) -> bool {
    let Some(Json::Arr(items)) = v else { return false };
    let words: Vec<&str> = items.iter().filter_map(Json::as_str).collect();
    (empty || !items.is_empty()) && words.len() == items.len() && words.iter().all(|w| allowed.contains(w)) && (1..words.len()).all(|i| !words[..i].contains(&words[i]))
}

fn positive(v: Option<&Json>) -> bool {
    matches!(v, Some(Json::Num(n)) if n.is_finite() && *n > 0.0)
}

fn word(t: &DocTree, k: &str) -> Result<(), String> {
    match t.attr(k) {
        Some(Json::Str(s)) if !s.is_empty() => Ok(()),
        None => Ok(()),
        Some(v) => Err(format!("{}() {} is a word, not {}", t.tag, rust_name(k), v.text())),
    }
}

fn one_of(t: &DocTree, k: &str, values: &[&str]) -> Result<(), String> {
    match t.attr(k) {
        None => Ok(()),
        Some(Json::Str(s)) if values.contains(&s.as_str()) => Ok(()),
        Some(v) => Err(format!("{}() {} is {}, not {}", t.tag, rust_name(k), quoted(values), v.text())),
    }
}

fn check_realtime(v: &Json) -> Result<(), String> {
    let no = |why: String| Err(format!("harness() realtime {why}"));
    let Json::Obj(entries) = v else { return no(format!(r#"is an object ({{"in": ["audio"], "out": ["audio", "tools"], …}}), not {}"#, v.text())) };
    for (k, _) in entries {
        if !REALTIME_FIELDS.contains(&k.as_str()) {
            return no(format!("has no field {k}; its fields are {}", REALTIME_FIELDS.join(", ")));
        }
    }
    if !list_of(v.get("in"), REALTIME_IN, false) {
        return no(format!("in is a list of {}", quoted(REALTIME_IN)));
    }
    if !list_of(v.get("out"), REALTIME_OUT, false) {
        return no(format!("out is a list of {}", quoted(REALTIME_OUT)));
    }
    let says = |side: &str, what: &str| matches!(v.get(side), Some(Json::Arr(items)) if items.iter().any(|x| x.as_str() == Some(what)));
    let audio = v.get("audio");
    if says("in", "audio") || says("out", "audio") {
        if !matches!(audio, Some(Json::Obj(_))) {
            return no(r#"audio names the formats it takes and gives: {"in": ["pcm16@24000"], "out": ["pcm16@24000"]}"#.into());
        }
        for side in ["in", "out"] {
            if says(side, "audio") && !list_of(audio.and_then(|a| a.get(side)), AUDIO_FORMATS, false) {
                return no(format!("audio.{side} is a list of {}", quoted(AUDIO_FORMATS)));
            }
        }
    } else if audio.is_some() {
        return no("audio is for a harness that takes or gives audio".into());
    }
    if says("in", "video") && !positive(v.get("fps")) {
        return no("fps is the most video frames a second it takes (a harness that takes video says it)".into());
    }
    if says("out", "tools") && !positive(v.get("hz")) {
        return no("hz is the most tool calls a second it decides (a harness that calls tools says it)".into());
    }
    for k in ["fps", "hz", "minutes", "videoMinutes"] {
        if let Some(x) = v.get(k) {
            if !positive(Some(x)) {
                return no(format!("{k} is a positive number, not {}", x.text()));
            }
        }
    }
    Ok(())
}

/// The fields of a record in the order of `fields` (the TypeScript SDK's `said`).
fn said(t: &DocTree, fields: &[&str]) -> Json {
    let mut out = Json::obj();
    for k in fields {
        if let Some(v) = t.attr(k) {
            out.set(k, v.clone());
        }
    }
    out
}

fn harness_of(t: &DocTree) -> Result<Json, String> {
    for k in ["label", "provider", "cli", "family", "homepage", "about"] {
        word(t, k)?;
    }
    one_of(t, "kind", HARNESS_KINDS)?;
    if let Some(v) = t.attr("experimental") {
        if !matches!(v, Json::Bool(_)) {
            return Err(format!("harness() experimental is true or false, not {}", v.text()));
        }
    }
    if let Some(v) = t.attr("runs") {
        if !list_of(Some(v), HARNESS_RUNTIMES, true) {
            return Err(format!("harness() runs is a list of {}, not {}", quoted(HARNESS_RUNTIMES), v.text()));
        }
    }
    if let Some(v) = t.attr("tasks") {
        if !list_of(Some(v), HARNESS_TASKS, false) {
            return Err(format!("harness() tasks is a list of {}, not {}", quoted(HARNESS_TASKS), v.text()));
        }
    }
    let kind = t.attr("kind").and_then(Json::as_str);
    if t.attr("tasks").is_some() && kind != Some("api") {
        return Err(r#"harness() tasks is for kind("api") only: a model that does one task in a turn"#.into());
    }
    if kind == Some("cli") && t.attr("cli").is_none() {
        return Err(r#"harness().kind("cli") names the program it starts: .cli("claude")"#.into());
    }
    if kind != Some("cli") && t.attr("cli").is_some() {
        return Err(r#"harness() cli is for kind("cli") only"#.into());
    }
    if kind == Some("api") && t.attr("provider").is_none() {
        return Err(r#"harness().kind("api") names its provider in the model catalog: .provider("anthropic")"#.into());
    }
    if let Some(r) = t.attr("realtime") {
        if kind != Some("api") || t.attr("tasks").is_some() {
            return Err(r#"harness() realtime is for kind("api") with no tasks: a model that converses live"#.into());
        }
        check_realtime(r)?;
    }
    Ok(said(t, HARNESS_FIELDS))
}

fn agent_of(t: &DocTree) -> Result<Json, String> {
    for k in ["harness", "parent", "name", "model", "effort", "instructions", "voice", "color", "icon", "gender"] {
        word(t, k)?;
    }
    one_of(t, "voiceMode", VOICE_MODES)?;
    let (base, fork) = (t.attr("harness").is_some(), t.attr("parent").is_some());
    if base == fork {
        return Err(if base {
            "agent() names a harness (a base instance) or a parent (a fork), not both: a fork keeps its parent's harness".into()
        } else {
            r#"agent() names its harness (a base instance: .harness("claude-code")) or its parent (a fork: .parent("cloud://global/agents/claude-code"))"#.into()
        });
    }
    if let Some(Json::Str(h)) = t.attr("harness") {
        let id = h.len() <= 80 && h.bytes().enumerate().all(|(i, c)| c.is_ascii_lowercase() || c.is_ascii_digit() || (i > 0 && c == b'-'));
        if !id {
            return Err(format!(r#"agent() harness is a harness's id ("claude-code"), not {}"#, Json::Str(h.clone()).text()));
        }
    }
    if let Some(v) = t.attr("tools") {
        if !matches!(v, Json::Arr(items) if items.iter().all(|x| matches!(x, Json::Str(s) if !s.is_empty()))) {
            return Err(format!("agent() tools is a list of tool names, not {}", v.text()));
        }
    }
    if let Some(v) = t.attr("persona") {
        if !matches!(v, Json::Obj(_)) {
            return Err(format!(r#"agent() persona is an object ({{"tone": "warm"}}), not {}"#, v.text()));
        }
    }
    Ok(said(t, AGENT_FIELDS))
}
