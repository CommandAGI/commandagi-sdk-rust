//! A PROCESS FLOW IN RUST — `<name>.flow.rs`: the ordered steps that make one thing, each with the tool it visits, its
//! recipe parameters with their units, its time and who does it, and the measurements taken in the process. The same
//! elements, and the same document, as the TypeScript SDK's `flows.ts` and the Python SDK's `flows.py`:
//!
//! ```
//! use commandagi::design::flows::*;
//!
//! fn document() -> El {
//!     flow([
//!         tool().id("coater").ref_("stage1.bom.3d.rs#coater").rate_per_hour(40).currency("USD"),
//!         step([]).id("why").kind("note").instruction("Marks first: every later layer aligns to them."),
//!         step([
//!             param().name("speed").value(3000).unit("rpm"),
//!             measure().id("resist").name("resist thickness").nominal(1.2).plus(0.1).minus(0.1).unit("um"),
//!         ])
//!         .id("coat").order(1).tool("coater").by("in-house").minutes(6),
//!     ])
//!     .name("Alignment marks")
//!     .of("stage1.bom.3d.rs#wafer")
//! }
//! # commandagi::design::declare(document()).unwrap();
//! ```
//!
//! A record is an element and its fields are the element's attributes (`rate_per_hour` is `ratePerHour`; `ref_` is
//! `ref`). Nothing adds a default. The check is the host's (`commandagi check <file>.flow.rs`).

use super::documents::{declare_document, DocTree, TagRule, Vocabulary};
use super::element::{attributes, elements};
use super::{Declared, Family};
pub use super::{fragment, json, El, Json, Value};

elements! {
    /// A process flow: its tools and its steps.
    flow: children;
    /// A tool a step visits: `.id("coater").ref_("Kit.3d.rs#coater").rate_per_hour(40).currency("USD")`.
    tool: leaf;
    /// One step, with its params and measures: `step([…]).id("coat").order(1).tool("coater").by("in-house").minutes(6)`.
    step: children;
    /// A recipe parameter with its unit: `.name("speed").value(3000).unit("rpm")`.
    param: leaf;
    /// A measurement taken in the process and its limits: `.id("cd").nominal(2).plus(0.2).minus(0.2).unit("um")`.
    measure: leaf;
}

attributes! {
    /// The fields of a flow's records, chained on their elements.
    pub trait FlowsAttrs {
        name; description; of; id; ref_; rate_per_hour; currency; order; kind; tool; fixture; by; minutes;
        setup_minutes; instruction; note; value; unit; nominal; min; max; plus; minus; method; sampling; acceptance;
    }
}

pub(crate) const FAMILY: Family = Family { module: "flows", roots: &["flow"], declare };

pub const FLOW_STEP_KINDS: &[&str] = &["work", "note"];
pub const FLOW_FIELDS: &[&str] = &["name", "description", "of"];
pub const FLOW_TOOL_FIELDS: &[&str] = &["id", "ref", "name", "ratePerHour", "currency", "description"];
pub const FLOW_STEP_FIELDS: &[&str] = &["id", "order", "kind", "tool", "fixture", "by", "minutes", "setupMinutes", "ratePerHour", "currency", "instruction", "note"];
pub const FLOW_PARAM_FIELDS: &[&str] = &["name", "value", "unit", "description"];
pub const FLOW_MEASURE_FIELDS: &[&str] = &["id", "name", "nominal", "min", "max", "plus", "minus", "unit", "method", "sampling", "acceptance"];

const FLOW: Vocabulary = Vocabulary {
    format: "flow",
    noun: "a process flow",
    root: "flow",
    tags: &[
        TagRule::new("flow", &[]).attrs(FLOW_FIELDS),
        TagRule::new("tool", &["flow"]).key("id").required(&["id"]).attrs(FLOW_TOOL_FIELDS),
        TagRule::new("step", &["flow"]).key("id").required(&["id"]).attrs(FLOW_STEP_FIELDS),
        TagRule::new("param", &["step"]).required(&["name", "value"]).attrs(FLOW_PARAM_FIELDS),
        TagRule::new("measure", &["step"]).key("id").required(&["id"]).attrs(FLOW_MEASURE_FIELDS),
    ],
    from_tree: flow_of,
};

fn declare(root: El) -> Result<Declared, String> {
    declare_document(&root, &FLOW)
}

/// How a message names a record: its call with the attribute that names it (`step().id("x")`).
fn at(t: &DocTree, key: &str) -> String {
    let call = super::element::fn_name(t.tag);
    match t.attr(key) {
        Some(v) => format!("{call}().{}({})", super::element::rust_name(key), v.text()),
        None => format!("{call}()"),
    }
}
fn num(v: Option<&Json>) -> Option<f64> {
    match v {
        Some(Json::Num(n)) if n.is_finite() => Some(*n),
        _ => None,
    }
}
fn need(cond: bool, message: impl FnOnce() -> String) -> Result<(), String> {
    if cond {
        Ok(())
    } else {
        Err(message())
    }
}
fn is_id(v: Option<&Json>) -> bool {
    matches!(v, Some(Json::Str(s)) if !s.is_empty() && s.chars().next().unwrap().is_ascii_alphanumeric() && s.chars().all(|ch| ch.is_ascii_alphanumeric() || "._-".contains(ch)))
}
fn said(v: Option<&Json>) -> String {
    v.map(Json::text).unwrap_or_else(|| "undefined".into())
}

fn money(t: &DocTree, w: &str) -> Result<(), String> {
    need(t.attr("ratePerHour").is_none() || num(t.attr("ratePerHour")).is_some_and(|r| r >= 0.0), || format!("{w}: ratePerHour is a number of at least 0 (money an hour)"))?;
    need(t.attr("currency").is_none() || matches!(t.attr("currency"), Some(Json::Str(s)) if s.len() == 3 && s.chars().all(|c| c.is_ascii_uppercase())), || format!("{w}: currency is an ISO 4217 code (USD)"))
}

fn tool_ok(t: &DocTree) -> Result<(), String> {
    let w = at(t, "id");
    need(is_id(t.attr("id")), || format!(r#"{w}: id is letters, digits, ".", "_" and "-""#))?;
    need(t.attr("ref").is_none() || matches!(t.attr("ref"), Some(Json::Str(s)) if !s.trim().is_empty()), || format!("{w}: ref is a device definition, or an item or body of a 3D document (<file>#<id>)"))?;
    money(t, &w)
}

fn step_ok(t: &DocTree) -> Result<(), String> {
    let w = at(t, "id");
    need(is_id(t.attr("id")), || format!(r#"{w}: id is letters, digits, ".", "_" and "-""#))?;
    need(t.attr("order").is_none() || num(t.attr("order")).is_some_and(|o| o >= 1.0 && o.fract() == 0.0), || format!("{w}: order is a whole number from 1"))?;
    need(t.attr("kind").is_none() || matches!(t.attr("kind"), Some(Json::Str(s)) if FLOW_STEP_KINDS.contains(&s.as_str())), || format!("{w}: kind must be one of {} (got {})", FLOW_STEP_KINDS.join(", "), said(t.attr("kind"))))?;
    for k in ["minutes", "setupMinutes"] {
        need(t.attr(k).is_none() || num(t.attr(k)).is_some_and(|m| m >= 0.0), || format!("{w}: {k} is a number of at least 0"))?;
    }
    money(t, &w)?;
    if t.attr("kind").and_then(Json::as_str) == Some("note") {
        let no_time = t.attr("minutes").is_none() || num(t.attr("minutes")) == Some(0.0);
        need(t.attr("tool").is_none() && no_time && t.children.is_empty(), || format!("{w}: a note step names no tool, takes no time (minutes 0 or absent) and holds no param or measure"))?;
    }
    let names: Vec<&Json> = t.children.iter().filter(|c| c.tag == "param").filter_map(|c| c.attr("name")).collect();
    if let Some(twice) = names.iter().enumerate().find(|(i, n)| names[..*i].contains(n)).map(|(_, n)| *n) {
        return Err(format!("{w}: two params are named {}", twice.text()));
    }
    for c in &t.children {
        if c.tag == "param" {
            let pw = at(c, "name");
            need(matches!(c.attr("name"), Some(Json::Str(s)) if !s.trim().is_empty()), || format!("{pw}: name says what the parameter is"))?;
            need(matches!(c.attr("value"), Some(Json::Num(_) | Json::Str(_) | Json::Bool(_))), || format!("{pw}: value is a number, a text or true/false"))?;
            need(c.attr("unit").is_none() || matches!(c.attr("unit"), Some(Json::Str(s)) if !s.trim().is_empty()), || format!("{pw}: unit is a unit's symbol (rpm, sccm, mTorr, °C, s)"))?;
        } else {
            let mw = at(c, "id");
            need(is_id(c.attr("id")), || format!(r#"{mw}: id is letters, digits, ".", "_" and "-""#))?;
            for k in ["nominal", "min", "max", "plus", "minus"] {
                need(c.attr(k).is_none() || num(c.attr(k)).is_some(), || format!("{mw}: {k} is a number"))?;
            }
            for k in ["plus", "minus"] {
                need(c.attr(k).is_none() || num(c.attr(k)).is_some_and(|v| v >= 0.0), || format!("{mw}: {k} is a number of at least 0"))?;
            }
            need((c.attr("plus").is_none() && c.attr("minus").is_none()) || c.attr("nominal").is_some(), || format!("{mw}: plus and minus are tolerances of the nominal: give nominal"))?;
            if let (Some(lo), Some(hi)) = (num(c.attr("min")), num(c.attr("max"))) {
                need(lo <= hi, || format!("{mw}: min is at most max"))?;
            }
        }
    }
    Ok(())
}

fn flow_of(t: &DocTree) -> Result<Json, String> {
    need(t.attr("of").is_none() || matches!(t.attr("of"), Some(Json::Str(_))), || "flow(): of is a ref to what the flow makes (a 3D document, or <file>#<item or body>)".to_string())?;
    for c in &t.children {
        if c.tag == "tool" {
            tool_ok(c)?;
        } else {
            step_ok(c)?;
        }
    }
    let measures: Vec<&Json> = t.children.iter().flat_map(|c| c.children.iter()).filter(|m| m.tag == "measure").filter_map(|m| m.attr("id")).collect();
    if let Some(again) = measures.iter().enumerate().find(|(i, m)| measures[..*i].contains(m)).map(|(_, m)| *m) {
        return Err(format!("measure().id({}): two measures have this id (a ref names it as <file>#<id>)", again.text()));
    }
    let mut out = Json::obj();
    for k in FLOW_FIELDS {
        if let Some(v) = t.attr(k) {
            out.set(k, v.clone());
        }
    }
    let tools: Vec<Json> = t.children.iter().filter(|c| c.tag == "tool").map(DocTree::record).collect();
    if !tools.is_empty() {
        out.set("tools", tools);
    }
    let steps: Vec<Json> = t
        .children
        .iter()
        .filter(|c| c.tag == "step")
        .map(|s| {
            let mut rec = s.record();
            let params: Vec<Json> = s.children.iter().filter(|c| c.tag == "param").map(DocTree::record).collect();
            let measures: Vec<Json> = s.children.iter().filter(|c| c.tag == "measure").map(DocTree::record).collect();
            if !params.is_empty() {
                rec.set("params", params);
            }
            if !measures.is_empty() {
                rec.set("measures", measures);
            }
            rec
        })
        .collect();
    if !steps.is_empty() {
        out.set("steps", steps);
    }
    Ok(out)
}
