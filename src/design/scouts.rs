//! SCOUTS IN RUST — the two documents of CommandAGI's scouts (``docs/scouts.md`` in the CommandAGI repository), the
//! same elements as the TypeScript SDK's `scouts.ts` and the Python SDK's `scouts.py`: a comparison
//! (`<name>.scout.rs`) and a replacement-cost analysis (`<name>.replacement.rs`). A record is an element and its fields
//! are its attributes (snake_case methods of the native names: `.max_tokens(…)` is `maxTokens`, `.from_(…)` is `from`):
//!
//! ```
//! use commandagi::design::scouts::*;
//!
//! fn document() -> El {
//!     scout([
//!         configuration().id("sonnet").model("anthropic/claude-sonnet-5-5").effort("low").incumbent(true),
//!         configuration().id("glm").model("dashscope/glm-5.3"),
//!         workload([
//!             case().id("rate").from_("synthetic").prompt("What is 4.2% as a decimal?")
//!                 .check(json(r#"{"number": 0.042, "tolerance": 0.0005}"#)),
//!             case().id("page").from_("own-run").source("runs/thread-1/streams/chat@a/records.jsonl#12")
//!                 .prompt("Write the provider page.").rubric("Names the price and the terms."),
//!         ])
//!         .id("knowledge").title("Knowledge work").needs(["structured"]).sample(20),
//!         judge().model("openai/gpt-5.6-terra"),
//!     ])
//!     .budget(10).repeats(3).title("GLM against Sonnet")
//! }
//! # commandagi::design::declare(document()).unwrap();
//! ```
//!
//! Nothing adds a default: what the file does not say, the document does not have (the runner and the reader fill their
//! own). Structure and enums are checked here; the meaning (a model the catalog lists, a ref that resolves, the sums) is
//! the reader's. A `regex` check's pattern is text here: this crate has no dependencies, so no regex engine compiles it —
//! the run does, and a pattern that does not compile fails there.

use super::documents::{declare_document, DocTree, TagRule, Vocabulary};
use super::element::{attributes, elements, fn_name, rust_name};
use super::{Declared, Family};
pub use super::{fragment, json, El, Json, Value};

elements! {
    /// A comparison: its configurations, its workloads with their cases, and its judges.
    scout: children;
    /// One configuration: a harness, a model, an effort and instructions.
    configuration: leaf;
    /// One workload: the cases measured on every configuration.
    workload: children;
    /// One case: a prompt, a scorer, and the tools it offers.
    case: leaf;
    /// A judge of the rubric-scored cases.
    judge: leaf;
    /// A replacement-cost analysis: what replacing a product would cost.
    replacement: children;
    /// One layer of it (`visible`, `invisible`, `integrations`, `operations`, `economics`).
    layer: children;
    /// One capability of a layer: what we need of it, what we have, what the rest costs.
    capability: children;
    /// What a claim rests on.
    evidence: leaf;
    /// The decision: `build`, `buy`, `integrate` or `skip`.
    decision: leaf;
    /// What only we would do.
    differentiation: leaf;
    /// The price, the customers and the margin.
    economics: leaf;
}

attributes! {
    /// The fields of a comparison and of a replacement-cost analysis, chained on their elements.
    pub trait ScoutsAttrs {
        title; budget; repeats; margin; relative_margin; error_floor; min_calls; min_cases; resamples; alpha; seed;
        max_cases; readme;
        id; model; harness; effort; instructions; max_tokens; incumbent;
        needs; sample; context;
        from_; prompt; system; check; rubric; tools; turns; source; terms;
        product; vendor; url; as_of; by_; kind;
        need; have; run; labour; ref_; note; claim; choice; because; text; price; customers;
    }
}

pub(crate) const FAMILY: Family = Family { module: "scouts", roots: &["scout", "replacement"], declare };

pub const SCOUT_FIELDS: &[&str] = &[
    "title", "budget", "repeats", "margin", "relativeMargin", "errorFloor", "minCalls", "minCases", "resamples", "alpha", "seed", "maxCases", "readme",
];
pub const CONFIGURATION_FIELDS: &[&str] = &["id", "model", "harness", "effort", "instructions", "maxTokens", "incumbent"];
pub const WORKLOAD_FIELDS: &[&str] = &["id", "title", "needs", "sample", "context"];
pub const CASE_FIELDS: &[&str] = &["id", "from", "prompt", "system", "check", "rubric", "tools", "turns", "source", "terms"];
pub const JUDGE_FIELDS: &[&str] = &["model", "effort"];

/// Where a case's data came from (docs/scouts.md § the corpus). A customer's runs and third-party records are never cases.
pub const CASE_ORIGINS: &[&str] = &["public", "synthetic", "own-run"];
/// What a workload needs of a model: a hard gate, never a score.
pub const WORKLOAD_NEEDS: &[&str] = &["tools", "structured", "image"];
/// The checks a case may name, cheapest first (a judge's rubric only where no check exists).
pub const CHECK_KINDS: &[&str] = &["exact", "oneOf", "number", "contains", "regex", "json", "letter", "literal", "tool", "program"];

pub const REPLACEMENT_FIELDS: &[&str] = &["product", "vendor", "url", "asOf", "by", "readme"];
pub const LAYER_FIELDS: &[&str] = &["kind", "title"];
pub const CAPABILITY_FIELDS: &[&str] = &["id", "title", "need", "have", "effort", "run", "labour", "ref", "note"];
pub const EVIDENCE_FIELDS: &[&str] = &["source", "claim"];
pub const DECISION_FIELDS: &[&str] = &["choice", "because"];
pub const ECONOMICS_FIELDS: &[&str] = &["price", "customers", "margin", "note"];

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
    match t.attr_of("id") {
        Some(v @ Json::Str(_)) => format!("{}().id({})", fn_name(t.tag), v.text()),
        _ => format!("{}()", fn_name(t.tag)),
    }
}

fn said(v: Option<&Json>) -> String {
    v.map(Json::text).unwrap_or_else(|| "undefined".into())
}

fn text(t: &DocTree, k: &str) -> Result<(), String> {
    match t.attr_of(k) {
        Some(Json::Str(s)) if !s.is_empty() => Ok(()),
        Some(v) => Err(format!("{} {} is text, not {}", call(t), rust_name(k), v.text())),
        None => Ok(()),
    }
}

fn one_of(t: &DocTree, k: &str, allowed: &[&str]) -> Result<(), String> {
    match t.attr_of(k) {
        Some(Json::Str(s)) if allowed.contains(&s.as_str()) => Ok(()),
        Some(v) => Err(format!("{} {} is one of {}, not {}", call(t), rust_name(k), allowed.join(", "), v.text())),
        None => Ok(()),
    }
}

/// A number above 0, or (`zero`) of 0 or more; `integer` makes it a whole one.
fn positive(t: &DocTree, k: &str, integer: bool, zero: bool) -> Result<(), String> {
    match t.attr_of(k) {
        Some(Json::Num(n)) if n.is_finite() && (if zero { *n >= 0.0 } else { *n > 0.0 }) && (!integer || n.fract() == 0.0) => Ok(()),
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

fn is_number(v: &Json) -> bool {
    matches!(v, Json::Num(n) if n.is_finite())
}

/// The check a case is scored by: exactly one kind, and the shape each kind needs.
fn check_of(t: &DocTree) -> Result<(), String> {
    let Some(c) = t.attr_of("check") else { return Ok(()) };
    let Json::Obj(entries) = c else {
        return Err(format!("{} check is an object ({{ exact: \"42\" }}), not {}", call(t), c.text()));
    };
    let kinds: Vec<&str> = entries.iter().map(|(k, _)| k.as_str()).filter(|k| *k != "tolerance").collect();
    if kinds.len() != 1 || !CHECK_KINDS.contains(&kinds[0]) {
        return Err(format!(
            "{} check is exactly one of {}{}",
            call(t),
            CHECK_KINDS.join(", "),
            if kinds.is_empty() { String::new() } else { format!(" (it has {})", kinds.join(" and ")) }
        ));
    }
    let kind = kinds[0];
    if c.get("tolerance").is_some() && kind != "number" {
        return Err(format!("{} check: tolerance goes with number", call(t)));
    }
    if let Some(n) = c.get("number") {
        let tolerance_ok = c.get("tolerance").is_none_or(|x| matches!(x, Json::Num(n) if *n >= 0.0));
        if !is_number(n) || !tolerance_ok {
            return Err(format!("{} check: number and tolerance are numbers", call(t)));
        }
    }
    if let Some(v) = c.get("exact").filter(|v| !matches!(v, Json::Str(_))) {
        return Err(format!("{} check: exact is text, not {}", call(t), v.text()));
    }
    for k in ["oneOf", "contains"] {
        if let Some(v) = c.get(k) {
            let ok = matches!(v, Json::Arr(items) if !items.is_empty() && items.iter().all(|s| matches!(s, Json::Str(s) if !s.is_empty())));
            if !ok {
                return Err(format!("{} check: {} is a list of texts", call(t), k));
            }
        }
    }
    if let Some(v) = c.get("regex").filter(|v| !matches!(v, Json::Str(s) if !s.is_empty())) {
        return Err(format!("{} check: regex is text, not {}", call(t), v.text()));
    }
    if let Some(v) = c.get("json").filter(|v| !matches!(v, Json::Obj(_))) {
        return Err(format!("{} check: json is the object the answer must hold, not {}", call(t), v.text()));
    }
    if let Some(v) = c.get("letter") {
        let ok = matches!(v, Json::Str(s) if s.len() == 1 && s.chars().next().is_some_and(|c| c.is_ascii_uppercase()));
        if !ok {
            return Err(format!("{} check: letter is one capital letter, not {}", call(t), v.text()));
        }
    }
    for k in ["literal", "tool"] {
        if let Some(v) = c.get(k).filter(|v| !matches!(v, Json::Str(s) if !s.is_empty())) {
            return Err(format!("{} check: {} is text, not {}", call(t), k, v.text()));
        }
    }
    if c.get("tool").is_some() && t.attr_of("tools").is_none() {
        return Err(format!("{} check: tool needs the case's tools", call(t)));
    }
    if let Some(p) = c.get("program") {
        let Json::Obj(_) = p else {
            return Err(format!("{} check: program is {{ data: [[x, y], …] }}, not {}", call(t), p.text()));
        };
        let data_ok = matches!(p.get("data"), Some(Json::Arr(rows)) if !rows.is_empty() && rows.iter().all(|r| {
            matches!(r, Json::Arr(pair) if pair.len() == 2 && pair.iter().all(is_number))
        }));
        if !data_ok {
            return Err(format!("{} check: program is {{ data: [[x, y], …] }}", call(t)));
        }
        if let Some(v) = p.get("tolerance").filter(|v| !matches!(v, Json::Num(n) if *n >= 0.0)) {
            return Err(format!("{} check: program tolerance is a number of 0 or more, not {}", call(t), v.text()));
        }
    }
    Ok(())
}

/// The tools a case offers: each a name, a parameters schema, and the answers its results give.
fn tools_of(t: &DocTree) -> Result<(), String> {
    let Some(tools) = t.attr_of("tools") else { return Ok(()) };
    let Json::Arr(items) = tools else {
        return Err(format!("{} tools is a list of {{ name, parameters, results }}, not {}", call(t), tools.text()));
    };
    if items.is_empty() {
        return Err(format!("{} tools is a list of {{ name, parameters, results }}", call(t)));
    }
    let mut names: Vec<&str> = Vec::new();
    for tool in items {
        let Json::Obj(_) = tool else {
            return Err(format!("{} a tool is {{ name, parameters, results }}, not {}", call(t), tool.text()));
        };
        let name = match tool.get("name") {
            Some(Json::Str(s)) if !s.is_empty() && s.len() <= 64 && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') => s.as_str(),
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
                let ok = matches!(r, Json::Obj(_))
                    && matches!(r.get("result"), Some(Json::Str(_)))
                    && r.get("when").is_none_or(|w| matches!(w, Json::Obj(entries) if entries.iter().all(|(_, v)| matches!(v, Json::Str(_)))));
                if !ok {
                    return Err(format!(
                        "{} tool {}: a result is {{ when?: {{ <argument>: <text it contains> }}, result: <text> }}",
                        call(t),
                        name
                    ));
                }
            }
        } else if tool.get("results").is_some() {
            return Err(format!("{} tool {}: results is a list", call(t), name));
        }
    }
    positive(t, "turns", true, false)
}

fn case_of(t: &DocTree) -> Result<Json, String> {
    for k in ["id", "prompt", "system", "rubric", "source"] {
        text(t, k)?;
    }
    one_of(t, "from", CASE_ORIGINS)?;
    check_of(t)?;
    tools_of(t)?;
    if t.attr_of("turns").is_some() && t.attr_of("tools").is_none() {
        return Err(format!("{} turns goes with tools", call(t)));
    }
    if t.attr_of("check").is_some() == t.attr_of("rubric").is_some() {
        return Err(format!("{} is scored by exactly one of check (a program) or rubric (a judge panel)", call(t)));
    }
    if let Some(v) = t.attr_of("terms").filter(|v| !matches!(v, Json::Arr(items) if items.iter().all(|s| matches!(s, Json::Str(s) if !s.is_empty())))) {
        return Err(format!("{} terms is a list of term ids, not {}", call(t), v.text()));
    }
    Ok(fields(t, CASE_FIELDS))
}

fn configuration_of(c: &DocTree) -> Result<Json, String> {
    for k in ["id", "model", "harness", "effort", "instructions"] {
        text(c, k)?;
    }
    positive(c, "maxTokens", true, false)?;
    if let Some(v) = c.attr_of("incumbent").filter(|v| !matches!(v, Json::Bool(_))) {
        return Err(format!("{} incumbent is true or false, not {}", call(c), v.text()));
    }
    Ok(fields(c, CONFIGURATION_FIELDS))
}

fn workload_of(w: &DocTree) -> Result<Json, String> {
    text(w, "id")?;
    text(w, "title")?;
    positive(w, "sample", true, false)?;
    positive(w, "context", true, false)?;
    if let Some(v) = w.attr_of("needs").filter(|v| {
        !matches!(v, Json::Arr(items) if items.iter().all(|n| matches!(n, Json::Str(s) if WORKLOAD_NEEDS.contains(&s.as_str()))))
    }) {
        return Err(format!("{} needs is a list of {}, not {}", call(w), WORKLOAD_NEEDS.join(", "), v.text()));
    }
    let cases = w.children.iter().map(case_of).collect::<Result<Vec<_>, _>>()?;
    Ok(fields(w, WORKLOAD_FIELDS).with("cases", Json::Arr(cases)))
}

fn judge_of(j: &DocTree) -> Result<Json, String> {
    text(j, "model")?;
    text(j, "effort")?;
    Ok(fields(j, JUDGE_FIELDS))
}

fn scout_of(t: &DocTree) -> Result<Json, String> {
    text(t, "title")?;
    text(t, "readme")?;
    positive(t, "budget", false, false)?;
    for k in ["repeats", "minCalls", "minCases", "resamples", "maxCases"] {
        positive(t, k, true, false)?;
    }
    for k in ["margin", "relativeMargin", "errorFloor"] {
        positive(t, k, false, true)?;
    }
    positive(t, "alpha", false, false)?;
    if let Some(a) = t.attr_of("alpha").and_then(Json::as_f64).filter(|a| *a >= 1.0) {
        return Err(format!("<scout> alpha is below 1, not {}", a));
    }
    if let Some(v) = t.attr_of("seed").filter(|v| !matches!(v, Json::Num(n) if n.fract() == 0.0 && n.abs() <= 9_007_199_254_740_991.0)) {
        return Err(format!("<scout> seed is a whole number, not {}", v.text()));
    }
    let configurations = t.children.iter().filter(|c| c.tag == "configuration").map(configuration_of).collect::<Result<Vec<_>, _>>()?;
    let incumbents = configurations.iter().filter(|c| matches!(c.get("incumbent"), Some(Json::Bool(true)))).count();
    if incumbents > 1 {
        return Err("<scout> has one incumbent configuration".into());
    }
    let workloads = t.children.iter().filter(|w| w.tag == "workload").map(workload_of).collect::<Result<Vec<_>, _>>()?;
    let judges = t.children.iter().filter(|j| j.tag == "judge").map(judge_of).collect::<Result<Vec<_>, _>>()?;
    Ok(fields(t, SCOUT_FIELDS)
        .with("configurations", Json::Arr(configurations))
        .with("workloads", Json::Arr(workloads))
        .with("judges", Json::Arr(judges)))
}

fn capability_of(c: &DocTree) -> Result<Json, String> {
    for k in ["id", "title", "ref", "note"] {
        text(c, k)?;
    }
    one_of(c, "need", CAPABILITY_NEEDS)?;
    one_of(c, "have", CAPABILITY_HAVES)?;
    one_of(c, "effort", CAPABILITY_EFFORTS)?;
    positive(c, "run", false, true)?;
    positive(c, "labour", false, true)?;
    let evidence = c
        .children
        .iter()
        .filter(|e| e.tag == "evidence")
        .map(|e| {
            text(e, "source")?;
            one_of(e, "claim", EVIDENCE_CLAIMS)?;
            Ok(fields(e, EVIDENCE_FIELDS))
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(fields(c, CAPABILITY_FIELDS).with("evidence", Json::Arr(evidence)))
}

fn replacement_of(t: &DocTree) -> Result<Json, String> {
    for k in ["product", "vendor", "url", "by", "readme"] {
        text(t, k)?;
    }
    let as_of_ok = matches!(t.attr_of("asOf"), Some(Json::Str(s)) if {
        let b = s.as_bytes();
        b.len() == 10 && b[4] == b'-' && b[7] == b'-' && b.iter().enumerate().all(|(i, c)| i == 4 || i == 7 || c.is_ascii_digit())
    });
    if !as_of_ok {
        return Err(format!("<replacement> asOf is a day (2026-10-14), not {}", said(t.attr_of("asOf"))));
    }
    let layers = t
        .children
        .iter()
        .filter(|l| l.tag == "layer")
        .map(|l| {
            one_of(l, "kind", LAYER_KINDS)?;
            text(l, "title")?;
            let capabilities = l.children.iter().filter(|c| c.tag == "capability").map(capability_of).collect::<Result<Vec<_>, _>>()?;
            Ok(fields(l, LAYER_FIELDS).with("capabilities", Json::Arr(capabilities)))
        })
        .collect::<Result<Vec<_>, String>>()?;
    let mut doc = fields(t, REPLACEMENT_FIELDS).with("layers", Json::Arr(layers));
    if let Some(d) = t.single("decision") {
        one_of(d, "choice", DECISION_CHOICES)?;
        text(d, "because")?;
        doc.set("decision", fields(d, DECISION_FIELDS));
    }
    if let Some(d) = t.single("differentiation") {
        text(d, "text")?;
        if let Some(v) = d.attr_of("text") {
            doc.set("differentiation", v.clone());
        }
    }
    if let Some(e) = t.single("economics") {
        for k in ECONOMICS_FIELDS {
            text(e, k)?;
        }
        doc.set("economics", fields(e, ECONOMICS_FIELDS));
    }
    Ok(doc)
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
