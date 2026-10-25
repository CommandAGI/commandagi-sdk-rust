//! THE TWO DOCUMENTS OF COMMANDAGI'S SCOUTS IN RUST — `<name>.scout.rs` and `<name>.replacement.rs`. The same
//! elements, and the same documents, as the TypeScript SDK's `scouts.ts` (`<name>.scout.tsx`,
//! `<name>.replacement.tsx`) and the Python SDK's `scouts.py` (`docs/scouts.md`, in the CommandAGI repository). A
//! comparison declares one `scout`, its `configuration`s, its `workload`s with their `case`s, and its `judge`s; a
//! replacement-cost analysis declares one `replacement`, its `layer`s of `capability`s with their `evidence`, and its
//! `decision`. A record is an element and its fields are its attributes:
//!
//! ```
//! use commandagi::design::scouts::*;
//!
//! fn document() -> El {
//!     scout([
//!         configuration().id("sonnet").model("anthropic/claude-sonnet-5-5").effort("low").incumbent(true),
//!         configuration().id("glm").model("dashscope/glm-5.3"),
//!         workload([case()
//!             .id("rate").from_("synthetic").prompt("What is 4.2% as a decimal?")
//!             .check(json(r#"{"number": 0.042, "tolerance": 0.0005}"#))])
//!             .id("knowledge").title("Knowledge work").sample(20).needs(["structured"]),
//!         judge().model("openai/gpt-5.6-terra"),
//!     ])
//!     .title("GLM against Sonnet").budget(10).repeats(3)
//! }
//! # commandagi::design::declare(document()).unwrap();
//! ```
//!
//! A case's `from` and a replacement's `by` are Rust keywords, so their methods are `.from_()` and `.by_()`; a
//! capability's `ref` is `.ref_()`. References are refs: a path relative to the document's folder, or an absolute
//! address. Nothing adds a default: what the file does not say, the document does not have. A run gives back
//! `{format, document, sources}` beside an empty graph.

use super::documents::{declare_document, DocTree, TagRule, Vocabulary};
use super::element::{attributes, elements, fn_name, rust_name};
use super::{Declared, Family};
pub use super::{fragment, json, El, Json, Value};

elements! {
    /// A comparison: its `configuration`s, its `workload`s with their cases, and its `judge`s.
    scout: children;
    /// One configuration: `.id("sonnet").model("anthropic/claude-sonnet-5-5").effort("low")`.
    configuration: leaf;
    /// One workload, holding its cases.
    workload: children;
    /// One case, scored by `check` or by `rubric` (one of the two).
    case: leaf;
    /// One judge of a rubric.
    judge: leaf;
    /// A replacement-cost analysis: its layers of capabilities, and its decision.
    replacement: children;
    /// One layer of a product: visible, invisible, integrations, operations or economics.
    layer: children;
    /// One capability, holding the evidence for what it claims.
    capability: children;
    /// One piece of evidence for a capability.
    evidence: leaf;
    /// What replacing the product would take.
    decision: leaf;
    /// What an agent does better than the product.
    differentiation: leaf;
    /// What the product charges and what it earns.
    economics: leaf;
}

attributes! {
    /// The fields of a comparison, its configurations, workloads, cases and judges, and of a replacement analysis,
    /// its layers, capabilities, evidence and decision, chained on their elements.
    pub trait ScoutsAttrs {
        title; budget; repeats; margin; relative_margin; error_floor; min_calls; min_cases; resamples; alpha; seed;
        max_cases; readme;
        id; model; harness; effort; instructions; max_tokens; incumbent;
        needs; sample; context;
        from_; prompt; system; check; rubric; tools; turns; source; terms;
        product; vendor; url; as_of; by_;
        kind; need; have; run; labour; ref_; note;
        claim; choice; because; price; customers; text;
    }
}

pub(crate) const FAMILY: Family = Family { module: "scouts", roots: &["scout", "replacement"], declare };

pub const SCOUT_FIELDS: &[&str] = &[
    "title", "budget", "repeats", "margin", "relativeMargin", "errorFloor", "minCalls", "minCases", "resamples", "alpha",
    "seed", "maxCases", "readme",
];
pub const CONFIGURATION_FIELDS: &[&str] = &["id", "model", "harness", "effort", "instructions", "maxTokens", "incumbent"];
pub const WORKLOAD_FIELDS: &[&str] = &["id", "title", "needs", "sample", "context"];
pub const CASE_FIELDS: &[&str] = &["id", "from", "prompt", "system", "check", "rubric", "tools", "turns", "source", "terms"];
pub const JUDGE_FIELDS: &[&str] = &["model", "effort"];

pub const REPLACEMENT_FIELDS: &[&str] = &["product", "vendor", "url", "asOf", "by", "readme"];
pub const LAYER_FIELDS: &[&str] = &["kind", "title"];
pub const CAPABILITY_FIELDS: &[&str] = &["id", "title", "need", "have", "effort", "run", "labour", "ref", "note"];
pub const EVIDENCE_FIELDS: &[&str] = &["source", "claim"];
pub const DECISION_FIELDS: &[&str] = &["choice", "because"];
pub const ECONOMICS_FIELDS: &[&str] = &["price", "customers", "margin", "note"];

/// Where a case's data came from (docs/scouts.md § the corpus). A customer's runs and third-party records are never
/// cases.
pub const CASE_ORIGINS: &[&str] = &["public", "synthetic", "own-run"];
/// What a workload needs of a model: a hard gate, never a score.
pub const WORKLOAD_NEEDS: &[&str] = &["tools", "structured", "image"];
/// The checks a case may name, cheapest first (a judge's rubric only where no check exists).
pub const CHECK_KINDS: &[&str] = &["exact", "oneOf", "number", "contains", "regex", "json", "letter", "literal", "tool", "program"];

pub const LAYER_KINDS: &[&str] = &["visible", "invisible", "integrations", "operations", "economics"];
pub const CAPABILITY_NEEDS: &[&str] = &["must", "should", "later"];
pub const CAPABILITY_HAVES: &[&str] = &["built", "partial", "none"];
pub const CAPABILITY_EFFORTS: &[&str] = &["S", "M", "L", "XL"];
pub const EVIDENCE_CLAIMS: &[&str] = &["verified", "checked", "reported"];
pub const DECISION_CHOICES: &[&str] = &["build", "buy", "integrate", "skip"];

const SCOUT: Vocabulary = Vocabulary {
    format: "scout",
    noun: "a scout's comparison",
    root: "scout",
    tags: &[
        TagRule::new("scout", &[]).required(&["budget"]).attrs(SCOUT_FIELDS),
        TagRule::new("configuration", &["scout"]).key("id").required(&["id", "model"]).attrs(CONFIGURATION_FIELDS),
        TagRule::new("workload", &["scout"]).key("id").required(&["id"]).attrs(WORKLOAD_FIELDS),
        TagRule::new("case", &["workload"]).key("id").required(&["id", "from", "prompt"]).attrs(CASE_FIELDS),
        TagRule::new("judge", &["scout"]).key("model").required(&["model"]).attrs(JUDGE_FIELDS),
    ],
    from_tree: scout_of,
};

const REPLACEMENT: Vocabulary = Vocabulary {
    format: "replacement",
    noun: "a replacement-cost analysis",
    root: "replacement",
    tags: &[
        TagRule::new("replacement", &[]).required(&["product", "asOf"]).attrs(REPLACEMENT_FIELDS),
        // Two layers may be of one kind (the visible editor and the visible website): a layer is told apart by its title.
        TagRule::new("layer", &["replacement"]).key("title").required(&["kind", "title"]).attrs(LAYER_FIELDS),
        TagRule::new("capability", &["layer"]).key("id").required(&["id", "title", "need", "have"]).attrs(CAPABILITY_FIELDS),
        TagRule::new("evidence", &["capability"]).key("source").required(&["source", "claim"]).attrs(EVIDENCE_FIELDS),
        TagRule::new("decision", &["replacement"]).single().required(&["choice", "because"]).attrs(DECISION_FIELDS),
        TagRule::new("differentiation", &["replacement"]).single().required(&["text"]).attrs(&["text"]),
        TagRule::new("economics", &["replacement"]).single().attrs(ECONOMICS_FIELDS),
    ],
    from_tree: replacement_of,
};

fn declare(root: El) -> Result<Declared, String> {
    declare_document(&root, if root.tag == "scout" { &SCOUT } else { &REPLACEMENT })
}

/// How a message names an element: `configuration().id("sonnet")`.
fn call(t: &DocTree) -> String {
    match t.attr("id") {
        Some(v @ Json::Str(_)) => format!("{}().id({})", fn_name(t.tag), v.text()),
        _ => format!("{}()", fn_name(t.tag)),
    }
}

fn said(v: Option<&Json>) -> String {
    v.map(Json::text).unwrap_or_else(|| "undefined".into())
}

fn text(t: &DocTree, k: &str) -> Result<(), String> {
    match t.attr(k) {
        Some(Json::Str(s)) if !s.trim().is_empty() => Ok(()),
        Some(v) => Err(format!("{} {} is text, not {}", call(t), rust_name(k), v.text())),
        None => Ok(()),
    }
}

fn texts(t: &DocTree, k: &str) -> Result<(), String> {
    match t.attr(k) {
        Some(Json::Arr(items)) if items.iter().all(|s| matches!(s, Json::Str(s) if !s.trim().is_empty())) => Ok(()),
        Some(_) => Err(format!("{} {} is a list of texts", call(t), rust_name(k))),
        None => Ok(()),
    }
}

/// A number above 0 (of 0 or more with `zero`), whole when `integer`.
fn positive(t: &DocTree, k: &str, integer: bool, zero: bool) -> Result<(), String> {
    match t.attr(k) {
        Some(Json::Num(n)) if n.is_finite() && (!integer || n.fract() == 0.0) && (if zero { *n >= 0.0 } else { *n > 0.0 }) => Ok(()),
        Some(v) => Err(format!(
            "{} {} is a {}number{}, not {}",
            call(t),
            rust_name(k),
            if integer { "whole " } else { "" },
            if zero { " of 0 or more" } else { " above 0" },
            v.text()
        )),
        None => Ok(()),
    }
}

fn one_of(t: &DocTree, k: &str, allowed: &[&str]) -> Result<(), String> {
    match t.attr(k) {
        Some(Json::Str(s)) if allowed.contains(&s.as_str()) => Ok(()),
        Some(v) => Err(format!("{} {} is one of {}, not {}", call(t), rust_name(k), allowed.join(", "), v.text())),
        None => Ok(()),
    }
}

fn flag(t: &DocTree, k: &str) -> Result<(), String> {
    match t.attr(k) {
        Some(Json::Bool(_)) | None => Ok(()),
        Some(_) => Err(format!("{} {} is true or false", call(t), rust_name(k))),
    }
}

/// The fields a tree says, in the vocabulary's order.
fn fields(t: &DocTree, keys: &[&str]) -> Json {
    let mut out = Json::obj();
    for k in keys {
        if let Some(v) = t.attr(k) {
            out.set(k, v.clone());
        }
    }
    out
}

/// The child elements of one tag.
fn parts<'a, 'b>(t: &'a DocTree, tag: &'b str) -> impl Iterator<Item = &'a DocTree> + use<'a, 'b> {
    t.children.iter().filter(move |c| c.tag == tag)
}

/// The parts of a tool's name: letters, digits, `_` and `-`, at most 64 of them.
fn is_tool_name(s: &str) -> bool {
    !s.is_empty() && s.len() <= 64 && s.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
}

/// A day as a replacement analysis writes it (`2026-10-14`).
fn is_day(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() == 10
        && b[4] == b'-'
        && b[7] == b'-'
        && b.iter().enumerate().all(|(i, c)| i == 4 || i == 7 || c.is_ascii_digit())
}

/// A case's `check`: exactly one kind, and each kind's value of its own shape.
fn check_of(t: &DocTree) -> Result<(), String> {
    let Some(c) = t.attr("check") else { return Ok(()) };
    if !matches!(c, Json::Obj(_)) {
        return Err(format!("{} check is an object ({{ exact: \"42\" }}), not {}", call(t), c.text()));
    }
    let kinds: Vec<&str> = match c {
        Json::Obj(entries) => entries.iter().filter(|(k, _)| k != "tolerance").map(|(k, _)| k.as_str()).collect(),
        _ => unreachable!(),
    };
    if kinds.len() != 1 || !CHECK_KINDS.contains(&kinds[0]) {
        let extra = if kinds.is_empty() { String::new() } else { format!(" (it has {})", kinds.join(" and ")) };
        return Err(format!("{} check is exactly one of {}{}", call(t), CHECK_KINDS.join(", "), extra));
    }
    let kind = kinds[0];
    let tolerance = c.get("tolerance");
    if tolerance.is_some() && kind != "number" {
        return Err(format!("{} check: tolerance goes with number", call(t)));
    }
    let number_ok = match c.get("number") {
        None => true,
        Some(Json::Num(n)) if n.is_finite() => match tolerance {
            None => true,
            Some(Json::Num(t)) => t.is_finite() && *t >= 0.0,
            Some(_) => false,
        },
        Some(_) => false,
    };
    if !number_ok {
        return Err(format!("{} check: number and tolerance are numbers", call(t)));
    }
    if let Some(v) = c.get("exact").filter(|v| !matches!(v, Json::Str(_))) {
        let _ = v;
        return Err(format!("{} check: exact is text", call(t)));
    }
    for k in ["oneOf", "contains"] {
        match c.get(k) {
            Some(Json::Arr(items)) if !items.is_empty() && items.iter().all(|s| matches!(s, Json::Str(s) if !s.is_empty())) => {}
            Some(_) => return Err(format!("{} check: {} is a list of texts", call(t), k)),
            None => {}
        }
    }
    match c.get("regex") {
        Some(Json::Str(s)) if !s.is_empty() => {}
        Some(_) => return Err(format!("{} check: regex is text", call(t))),
        None => {}
    }
    if c.get("json").is_some_and(|j| !matches!(j, Json::Obj(_))) {
        return Err(format!("{} check: json is the object the answer must hold", call(t)));
    }
    match c.get("letter") {
        Some(Json::Str(s)) if s.len() == 1 && s.bytes().next().is_some_and(|b| b.is_ascii_uppercase()) => {}
        Some(_) => return Err(format!("{} check: letter is one capital letter", call(t))),
        None => {}
    }
    for k in ["literal", "tool"] {
        match c.get(k) {
            Some(Json::Str(s)) if !s.is_empty() => {}
            Some(_) => return Err(format!("{} check: {} is text", call(t), k)),
            None => {}
        }
    }
    if c.get("tool").is_some() && t.attr("tools").is_none() {
        return Err(format!("{} check: tool needs the case's tools", call(t)));
    }
    if let Some(p) = c.get("program") {
        if !matches!(p.get("data"), Some(Json::Arr(data)) if !data.is_empty()
            && data.iter().all(|d| matches!(d, Json::Arr(pair) if pair.len() == 2
                && pair.iter().all(|n| matches!(n, Json::Num(v) if v.is_finite())))))
        {
            return Err(format!("{} check: program is {{ data: [[x, y], …] }}", call(t)));
        }
        match p.get("tolerance") {
            None => {}
            Some(Json::Num(v)) if v.is_finite() && *v >= 0.0 => {}
            Some(_) => return Err(format!("{} check: program tolerance is a number of 0 or more", call(t))),
        }
    }
    Ok(())
}

/// A case's `tools`: each a name, a JSON Schema of its arguments, and the answers its arguments give.
fn tools_of(t: &DocTree) -> Result<(), String> {
    let Some(tools) = t.attr("tools") else { return Ok(()) };
    let Json::Arr(tools) = tools else {
        return Err(format!("{} tools is a list of {{ name, parameters, results }}", call(t)));
    };
    if tools.is_empty() {
        return Err(format!("{} tools is a list of {{ name, parameters, results }}", call(t)));
    }
    let mut names: Vec<&str> = Vec::new();
    for tool in tools {
        let name = match tool.get("name") {
            Some(Json::Str(s)) if is_tool_name(s) => s.as_str(),
            _ => return Err(format!("{} a tool's name is a word of letters, digits, _ and -", call(t))),
        };
        if names.contains(&name) {
            return Err(format!("{} has two tools named {}", call(t), name));
        }
        names.push(name);
        if !matches!(tool.get("parameters"), Some(Json::Obj(_))) {
            return Err(format!("{} tool {}: parameters is a JSON Schema object", call(t), name));
        }
        if let Some(Json::Arr(results)) = tool.get("results") {
            for r in results {
                let result_ok = matches!(r.get("result"), Some(Json::Str(_)))
                    && match r.get("when") {
                        None => true,
                        Some(Json::Obj(when)) => when.iter().all(|(_, v)| matches!(v, Json::Str(_))),
                        Some(_) => false,
                    };
                if !result_ok {
                    return Err(format!(
                        "{} tool {}: a result is {{ when?: {{ <argument>: <text it contains> }}, result: <text> }}",
                        call(t),
                        name
                    ));
                }
            }
        }
    }
    positive(t, "turns", true, false)
}

/// One case as its element declares it, or why it does not read (the TypeScript SDK's `caseOf`).
pub fn case_of(t: &DocTree) -> Result<Json, String> {
    for k in ["id", "prompt", "system", "rubric", "source"] {
        text(t, k)?;
    }
    one_of(t, "from", CASE_ORIGINS)?;
    check_of(t)?;
    tools_of(t)?;
    if t.attr("turns").is_some() && t.attr("tools").is_none() {
        return Err(format!("{} turns goes with tools", call(t)));
    }
    if t.attr("check").is_some() == t.attr("rubric").is_some() {
        return Err(format!("{} is scored by exactly one of check (a program) or rubric (a judge panel)", call(t)));
    }
    texts(t, "terms")?;
    Ok(fields(t, CASE_FIELDS))
}

fn scout_of(t: &DocTree) -> Result<Json, String> {
    for k in ["title", "readme"] {
        text(t, k)?;
    }
    positive(t, "budget", false, false)?;
    for k in ["repeats", "minCalls", "minCases", "resamples", "maxCases"] {
        positive(t, k, true, false)?;
    }
    for k in ["margin", "relativeMargin", "errorFloor"] {
        positive(t, k, false, true)?;
    }
    positive(t, "alpha", false, false)?;
    if let Some(Json::Num(a)) = t.attr("alpha") {
        if a.is_finite() && *a >= 1.0 {
            return Err("<scout> alpha is below 1".into());
        }
    }
    if let Some(v) = t.attr("seed") {
        match v {
            Json::Num(n) if n.fract() == 0.0 && n.abs() <= 9_007_199_254_740_991.0 => {}
            _ => return Err("<scout> seed is a whole number".into()),
        }
    }
    let mut configurations = Vec::new();
    for c in parts(t, "configuration") {
        for k in ["id", "model", "harness", "effort", "instructions"] {
            text(c, k)?;
        }
        positive(c, "maxTokens", true, false)?;
        flag(c, "incumbent")?;
        configurations.push(fields(c, CONFIGURATION_FIELDS));
    }
    let incumbents = configurations.iter().filter(|c| matches!(c.get("incumbent"), Some(Json::Bool(true)))).count();
    if incumbents > 1 {
        return Err("<scout> has one incumbent configuration".into());
    }
    let mut workloads = Vec::new();
    for w in parts(t, "workload") {
        text(w, "id")?;
        text(w, "title")?;
        positive(w, "sample", true, false)?;
        positive(w, "context", true, false)?;
        match w.attr("needs") {
            None => {}
            Some(Json::Arr(needs)) if needs.iter().all(|n| matches!(n, Json::Str(s) if WORKLOAD_NEEDS.contains(&s.as_str()))) => {}
            Some(_) => return Err(format!("{} needs is a list of {}", call(w), WORKLOAD_NEEDS.join(", "))),
        }
        let cases = w.children.iter().map(case_of).collect::<Result<Vec<_>, _>>()?;
        workloads.push(fields(w, WORKLOAD_FIELDS).with("cases", Json::Arr(cases)));
    }
    let mut judges = Vec::new();
    for j in parts(t, "judge") {
        text(j, "model")?;
        text(j, "effort")?;
        judges.push(fields(j, JUDGE_FIELDS));
    }
    Ok(fields(t, SCOUT_FIELDS).with("configurations", Json::Arr(configurations)).with("workloads", Json::Arr(workloads)).with("judges", Json::Arr(judges)))
}

fn replacement_of(t: &DocTree) -> Result<Json, String> {
    for k in ["product", "vendor", "url", "by", "readme"] {
        text(t, k)?;
    }
    match t.attr("asOf") {
        Some(Json::Str(s)) if is_day(s) => {}
        v => return Err(format!("<replacement> asOf is a day (2026-10-14), not {}", said(v))),
    }
    let mut layers = Vec::new();
    for layer in parts(t, "layer") {
        one_of(layer, "kind", LAYER_KINDS)?;
        text(layer, "title")?;
        let mut capabilities = Vec::new();
        for c in &layer.children {
            for k in ["id", "title", "ref", "note"] {
                text(c, k)?;
            }
            one_of(c, "need", CAPABILITY_NEEDS)?;
            one_of(c, "have", CAPABILITY_HAVES)?;
            one_of(c, "effort", CAPABILITY_EFFORTS)?;
            positive(c, "run", false, true)?;
            positive(c, "labour", false, true)?;
            let mut evidence = Vec::new();
            for e in &c.children {
                text(e, "source")?;
                one_of(e, "claim", EVIDENCE_CLAIMS)?;
                evidence.push(fields(e, EVIDENCE_FIELDS));
            }
            capabilities.push(fields(c, CAPABILITY_FIELDS).with("evidence", Json::Arr(evidence)));
        }
        layers.push(fields(layer, LAYER_FIELDS).with("capabilities", Json::Arr(capabilities)));
    }
    let mut out = fields(t, REPLACEMENT_FIELDS).with("layers", Json::Arr(layers));
    if let Some(decision) = t.single("decision") {
        one_of(decision, "choice", DECISION_CHOICES)?;
        text(decision, "because")?;
        out.set("decision", fields(decision, DECISION_FIELDS));
    }
    if let Some(differentiation) = t.single("differentiation") {
        text(differentiation, "text")?;
        if let Some(Json::Str(text)) = differentiation.attr("text") {
            out.set("differentiation", Json::Str(text.clone()));
        }
    }
    if let Some(economics) = t.single("economics") {
        for k in ECONOMICS_FIELDS {
            text(economics, k)?;
        }
        out.set("economics", fields(economics, ECONOMICS_FIELDS));
    }
    Ok(out)
}
