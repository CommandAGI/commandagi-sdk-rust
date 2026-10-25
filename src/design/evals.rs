//! AN EVAL AND A METRIC IN RUST — `<name>.eval.rs` and `<name>.metric.rs`. The same elements, and the same documents,
//! as the TypeScript SDK's `evals.ts` (`<name>.eval.tsx`, `<name>.metric.tsx`) and the Python SDK's `evals.py`. An eval
//! declares one `eval`, each of its cases one `case`; a metric declares one `metric`. A record is an element and its
//! fields are its attributes:
//!
//! ```
//! use commandagi::design::evals::*;
//!
//! fn document() -> El {
//!     eval([
//!         case().slug("undock-and-lap").name("Undock and lap").task("../Tasks/Undock and lap.task.tsx")
//!             .environment(json(r#"{"kind": "synthetic", "seed": 41}"#)),
//!         case().slug("open-a-terminal").name("Open a terminal").instruction("Open a terminal window.")
//!             .time_limit_ms(120000).environment(json(r#"{"kind": "none"}"#)),
//!     ])
//!     .domain("robot").slug("yard-shakedown").version("3").metrics(["Fleet availability.metric.tsx"]).repeats(3)
//! }
//! # commandagi::design::declare(document()).unwrap();
//! # commandagi::design::declare(
//! #     metric().slug("fleet-availability").probe(json(r#"{"kind": "event", "eventKind": "telemetry"}"#)).aggregation("rate"),
//! # ).unwrap();
//! ```
//!
//! References are refs: a path relative to the eval file's folder, or an absolute address. `metrics` names the metric
//! files the eval reads; a case's `task` names the `.task.tsx` whose title and text the agent is handed. A case with no
//! task file says its `instruction` itself; a case says one of the two. `objective` and `scoring`'s `weights` name metric
//! files by ref as well: `{"kind": "metric", "metric": "<ref>"}`, `{"kind": "linear", "terms": [{"metric": "<ref>",
//! "weight": 2}], "bias": 0}` or a `function`; the weights are keyed by ref. A metric's `probe` names a task file
//! (`{"kind": "task", "task": "<ref>"}`, an `alignment`'s `reference.task`) or another metric file (a `composite` input's
//! `metric`) by ref too; a probe that names one by node id (`taskId`, `metricId`) is refused, as the platform's routes
//! and reader refuse it. `frozen` is the digest an eval is frozen at
//! (`sha256:<hex>`). Nothing adds a default: what the file does not say, the document does not have. A run gives back
//! `{format, document, sources}` beside an empty graph; `sources` names each case's site by its slug
//! (`case#undock-and-lap`).

use super::documents::{declare_document, DocTree, TagRule, Vocabulary};
use super::element::{attributes, elements, fn_name, rust_name};
use super::{Declared, Family};
pub use super::{fragment, json, El, Json, Value};

elements! {
    /// An eval: one `case()` each.
    eval: children;
    /// One case: `.slug("undock-and-lap").name("Undock and lap").task("../Tasks/Undock and lap.task.tsx")`.
    case: leaf;
    /// A metric: `.slug("fleet-availability").probe(json(…)).aggregation("rate")`.
    metric: leaf;
}

attributes! {
    /// The fields of an eval, a case and a metric, chained on their elements.
    pub trait EvalsAttrs {
        domain; slug; version; description; metrics; scoring; objective; axes; repeats; published; frozen; official; rung;
        provenance;
        name; task; instruction; acceptance; acceptance_metric; hints; max_steps; time_limit_ms; environment; config; seed;
        weight; tags;
        probe; aggregation; role; direction; unit; target; pass; thread_id;
    }
}

pub(crate) const FAMILY: Family = Family { module: "evals", roots: &["eval", "metric"], declare };

pub const EVAL_FIELDS: &[&str] = &[
    "domain", "slug", "version", "description", "metrics", "scoring", "objective", "axes", "repeats", "published", "frozen", "official", "rung", "provenance",
];
pub const EVAL_CASE_FIELDS: &[&str] = &[
    "slug", "name", "task", "instruction", "acceptance", "acceptanceMetric", "hints", "maxSteps", "timeLimitMs", "environment", "config", "seed", "weight", "tags",
];
pub const METRIC_FIELDS: &[&str] = &["slug", "description", "probe", "aggregation", "role", "direction", "unit", "target", "pass", "threadId"];

const EVAL: Vocabulary = Vocabulary {
    format: "eval",
    noun: "an eval",
    root: "eval",
    tags: &[
        TagRule::new("eval", &[]).required(&["slug"]).attrs(EVAL_FIELDS),
        TagRule::new("case", &["eval"]).key("slug").required(&["slug", "name", "environment"]).attrs(EVAL_CASE_FIELDS),
    ],
    from_tree: eval_of,
};

const METRIC: Vocabulary = Vocabulary {
    format: "metric",
    noun: "a metric",
    root: "metric",
    tags: &[TagRule::new("metric", &[]).required(&["slug", "probe", "aggregation"]).attrs(METRIC_FIELDS)],
    from_tree: metric_of,
};

fn declare(root: El) -> Result<Declared, String> {
    declare_document(&root, if root.tag == "eval" { &EVAL } else { &METRIC })
}

/// Whether a text is a frozen eval's digest: `sha256:` and 64 lowercase hex digits.
pub fn is_digest(s: &str) -> bool {
    s.strip_prefix("sha256:").is_some_and(|hex| hex.len() == 64 && hex.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)))
}

/// How a message names an element: `case().slug("c")`.
fn call(t: &DocTree) -> String {
    match t.attr_of("slug") {
        Some(v @ Json::Str(_)) => format!("{}().slug({})", fn_name(t.tag), v.text()),
        _ => format!("{}()", fn_name(t.tag)),
    }
}

fn said(v: Option<&Json>) -> String {
    v.map(Json::text).unwrap_or_else(|| "undefined".into())
}

fn text(t: &DocTree, k: &str) -> Result<(), String> {
    match t.attr_of(k) {
        Some(Json::Str(s)) if !s.trim().is_empty() => Ok(()),
        Some(v) => Err(format!("{} {} is text, not {}", call(t), rust_name(k), v.text())),
        None => Ok(()),
    }
}

fn object(t: &DocTree, k: &str) -> Result<(), String> {
    match t.attr_of(k) {
        Some(Json::Obj(_)) | None => Ok(()),
        Some(v) => Err(format!("{} {} is an object, not {}", call(t), rust_name(k), v.text())),
    }
}

fn texts(t: &DocTree, k: &str) -> Result<(), String> {
    match t.attr_of(k) {
        Some(Json::Arr(items)) if items.iter().all(|s| matches!(s, Json::Str(s) if !s.trim().is_empty())) => Ok(()),
        Some(_) => Err(format!("{} {} is a list of texts", call(t), rust_name(k))),
        None => Ok(()),
    }
}

fn number(t: &DocTree, k: &str, integer: bool) -> Result<(), String> {
    match t.attr_of(k) {
        Some(Json::Num(n)) if n.is_finite() && (!integer || n.fract() == 0.0) => Ok(()),
        Some(v) => Err(format!("{} {} is a {}number, not {}", call(t), rust_name(k), if integer { "whole " } else { "" }, v.text())),
        None => Ok(()),
    }
}

fn flag(t: &DocTree, k: &str) -> Result<(), String> {
    match t.attr_of(k) {
        Some(Json::Bool(_)) | None => Ok(()),
        Some(_) => Err(format!("{} {} is true or false", call(t), rust_name(k))),
    }
}

/// Whether an object field has a text `kind` (a case's world, a metric's probe).
fn has_kind(v: Option<&Json>) -> bool {
    matches!(v.and_then(|v| v.get("kind")), Some(Json::Str(_)))
}

/// The fields a tree says, in the vocabulary's order.
fn fields(t: &DocTree, keys: &[&str]) -> Json {
    let mut out = Json::obj();
    for k in keys {
        if let Some(v) = t.attr_of(k) {
            out.set(k, v.clone());
        }
    }
    out
}

/// One case as its element declares it, or why it does not read (the TypeScript SDK's `evalCaseOf`).
pub fn eval_case_of(t: &DocTree) -> Result<Json, String> {
    for k in ["slug", "name", "task", "instruction", "acceptanceMetric"] {
        text(t, k)?;
    }
    for k in ["acceptance", "environment", "config"] {
        object(t, k)?;
    }
    if !has_kind(t.attr_of("environment")) {
        return Err(format!("{} environment is a world ({{ kind: \"synthetic\", seed: 41 }}), not {}", call(t), said(t.attr_of("environment"))));
    }
    let task = t.attr_of("task").is_some();
    if task == t.attr_of("instruction").is_some() {
        return Err(format!(
            "{} says what the agent is handed once: a task file (task) or an instruction, not {}",
            call(t),
            if task { "both" } else { "neither" }
        ));
    }
    texts(t, "hints")?;
    texts(t, "tags")?;
    number(t, "maxSteps", true)?;
    number(t, "timeLimitMs", true)?;
    number(t, "seed", true)?;
    number(t, "weight", false)?;
    Ok(fields(t, EVAL_CASE_FIELDS))
}

/// A metric named in `objective`: a ref (`metric`), never a node id (`metricId`).
fn metric_named(t: &DocTree, at: &str, v: &Json) -> Result<(), String> {
    if !matches!(v, Json::Obj(_)) {
        return Err(format!("{} {} is an object, not {}", call(t), at, v.text()));
    }
    if v.get("metricId").is_some() {
        return Err(format!("{} {} names its metric file by ref (metric), not by node id (metricId)", call(t), at));
    }
    match v.get("metric") {
        Some(Json::Str(s)) if !s.trim().is_empty() => Ok(()),
        m => Err(format!("{} {}.metric is a metric file's ref, not {}", call(t), at, said(m))),
    }
}

fn objective(t: &DocTree) -> Result<(), String> {
    let Some(o) = t.attr_of("objective") else { return Ok(()) };
    let kind = match o.get("kind") {
        Some(Json::Str(k)) => k.as_str(),
        _ => return Err(format!("{} objective is {{ kind: \"metric\" | \"linear\" | \"function\", … }}, not {}", call(t), o.text())),
    };
    if kind == "metric" {
        metric_named(t, "objective", o)?;
    }
    if kind == "linear" {
        let terms = match o.get("terms") {
            Some(Json::Arr(terms)) if !terms.is_empty() => terms,
            _ => return Err(format!("{} objective.terms is a list of {{ metric, weight }}", call(t))),
        };
        for (i, term) in terms.iter().enumerate() {
            metric_named(t, &format!("objective.terms[{i}]"), term)?;
            if let Some(w) = term.get("weight").filter(|w| !matches!(w, Json::Num(n) if n.is_finite())) {
                return Err(format!("{} objective.terms[{i}].weight is a number, not {}", call(t), w.text()));
            }
        }
    }
    Ok(())
}

fn weights(t: &DocTree) -> Result<(), String> {
    match t.attr_of("scoring").and_then(|s| s.get("weights")) {
        None => Ok(()),
        Some(Json::Obj(entries)) if entries.iter().all(|(_, w)| matches!(w, Json::Num(n) if n.is_finite())) => Ok(()),
        Some(w) => Err(format!("{} scoring.weights is {{ \"<metric file ref>\": <number> }}, not {}", call(t), w.text())),
    }
}

fn eval_of(t: &DocTree) -> Result<Json, String> {
    for k in ["slug", "domain", "version", "description", "rung"] {
        text(t, k)?;
    }
    for k in ["scoring", "objective", "provenance"] {
        object(t, k)?;
    }
    objective(t)?;
    weights(t)?;
    texts(t, "metrics")?;
    texts(t, "axes")?;
    number(t, "repeats", true)?;
    for k in ["published", "official"] {
        flag(t, k)?;
    }
    if let Some(v) = t.attr_of("frozen").filter(|v| !matches!(v, Json::Str(s) if is_digest(s))) {
        return Err(format!("{} frozen is the digest the eval is frozen at (sha256:<hex>), not {}", call(t), v.text()));
    }
    let cases = t.children.iter().filter(|c| c.tag == "case").map(eval_case_of).collect::<Result<Vec<_>, _>>()?;
    Ok(fields(t, EVAL_FIELDS).with("cases", Json::Arr(cases)))
}

/// A probe names a task or another metric by ref from the metric file (`task`, `reference.task`, an input's
/// `metric`), never by node id.
fn probe_refs(t: &DocTree) -> Result<(), String> {
    let probe = match t.attr("probe") {
        Some(probe) => probe,
        None => return Ok(()),
    };
    if probe.get("taskId").is_some() {
        return Err(format!("{} probe names its task file by ref (task), not by node id (taskId)", call(t)));
    }
    if probe.get("reference").and_then(|r| r.get("taskId")).is_some() {
        return Err(format!("{} probe names its task file by ref (reference.task), not by node id (taskId)", call(t)));
    }
    if let Some(Json::Arr(inputs)) = probe.get("inputs") {
        if inputs.iter().any(|i| i.get("metricId").is_some()) {
            return Err(format!("{} probe names each input's metric file by ref (metric), not by node id (metricId)", call(t)));
        }
    }
    Ok(())
}

fn metric_of(t: &DocTree) -> Result<Json, String> {
    for k in ["slug", "description", "aggregation", "role", "direction", "unit", "threadId"] {
        text(t, k)?;
    }
    for k in ["probe", "pass"] {
        object(t, k)?;
    }
    if !has_kind(t.attr_of("probe")) {
        return Err(format!("{} probe is what to measure ({{ kind: \"event\", eventKind: \"telemetry\" }}), not {}", call(t), said(t.attr_of("probe"))));
    }
    probe_refs(t)?;
    number(t, "target", false)?;
    Ok(fields(t, METRIC_FIELDS))
}
