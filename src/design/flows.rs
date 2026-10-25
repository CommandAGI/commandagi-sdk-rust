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
//!             visit().tool("coater").minutes(2).instruction("second coat"),
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
//! `ref`). A step that goes to more tools than its own names each in a `visit().tool(…).minutes(…)`. Nothing adds a
//! default. The check is the host's (`commandagi check <file>.flow.rs`).

use super::documents::{declare_document, DocTree, TagRule, Vocabulary};
use super::element::{attributes, elements};
use super::{Declared, Family};
pub use super::{fragment, json, El, Json, Value};

elements! {
    /// A process flow or an agent workflow: its inputs, tools and steps.
    flow: children;
    /// A typed input a run binds: `input().name("recipient").type_("text")`.
    input: leaf;
    /// A tool a step visits: `.id("coater").ref_("Kit.3d.rs#coater").rate_per_hour(40).currency("USD")`.
    tool: leaf;
    /// One step, with its params, measures and expects: `step([…]).id("coat").order(1).tool("coater").by("in-house").minutes(6)`.
    step: children;
    /// A recipe parameter with its unit: `.name("speed").value(3000).unit("rpm")`.
    param: leaf;
    /// A measurement taken in the process and its limits: `.id("cd").nominal(2).plus(0.2).minus(0.2).unit("um")`.
    measure: leaf;
    /// One more tool a step goes to, and its time there: `.tool("oven").minutes(2)`.
    visit: leaf;
    /// What must be observed after a step: `.name("screen").equals("home").within(5)`.
    expect: leaf;
}

attributes! {
    /// The fields of a flow's records, chained on their elements.
    pub trait FlowsAttrs {
        name; description; of; id; ref_; rate_per_hour; currency; order; kind; tool; fixture; by; minutes;
        setup_minutes; instruction; note; value; unit; nominal; min; max; plus; minus; method; sampling; acceptance;
        channel; command; after; surprise; surprise_to; type_; private; default_; required; equals; tolerance; within;
    }
}

pub(crate) const FAMILY: Family = Family { module: "flows", roots: &["flow"], declare };

pub const FLOW_STEP_KINDS: &[&str] = &["work", "note", "message", "command", "computer", "robot", "chain", "workflow", "decision", "agent"];
pub const FLOW_ACTION_KINDS: &[&str] = &["message", "command", "computer", "robot", "chain", "workflow", "decision", "agent"];
pub const FLOW_INPUT_TYPES: &[&str] = &["text", "number", "boolean"];
pub const FLOW_SURPRISE: &[&str] = &["parent", "agent", "person"];
pub const FLOW_FIELDS: &[&str] = &["name", "description", "of"];
pub const FLOW_INPUT_FIELDS: &[&str] = &["name", "type", "private", "default", "required", "description"];
pub const FLOW_TOOL_FIELDS: &[&str] = &["id", "ref", "name", "ratePerHour", "currency", "description"];
pub const FLOW_STEP_FIELDS: &[&str] = &["id", "order", "kind", "tool", "fixture", "by", "minutes", "setupMinutes", "ratePerHour", "currency", "instruction", "note", "channel", "ref", "command", "after", "surprise", "surpriseTo"];
pub const FLOW_PARAM_FIELDS: &[&str] = &["name", "value", "unit", "description"];
pub const FLOW_MEASURE_FIELDS: &[&str] = &["id", "name", "nominal", "min", "max", "plus", "minus", "unit", "method", "sampling", "acceptance"];
pub const FLOW_VISIT_FIELDS: &[&str] = &["tool", "minutes", "setupMinutes", "ratePerHour", "currency", "instruction"];
pub const FLOW_EXPECT_FIELDS: &[&str] = &["name", "equals", "tolerance", "within"];

const FLOW: Vocabulary = Vocabulary {
    format: "flow",
    noun: "a process flow",
    root: "flow",
    tags: &[
        TagRule::new("flow", &[]).attrs(FLOW_FIELDS),
        TagRule::new("input", &["flow"]).key("name").required(&["name"]).attrs(FLOW_INPUT_FIELDS),
        TagRule::new("tool", &["flow"]).key("id").required(&["id"]).attrs(FLOW_TOOL_FIELDS),
        TagRule::new("step", &["flow"]).key("id").required(&["id"]).attrs(FLOW_STEP_FIELDS),
        TagRule::new("param", &["step"]).required(&["name", "value"]).attrs(FLOW_PARAM_FIELDS),
        TagRule::new("measure", &["step"]).key("id").required(&["id"]).attrs(FLOW_MEASURE_FIELDS),
        TagRule::new("visit", &["step"]).required(&["tool"]).attrs(FLOW_VISIT_FIELDS),
        TagRule::new("expect", &["step"]).required(&["name", "equals"]).attrs(FLOW_EXPECT_FIELDS),
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

fn input_ok(t: &DocTree) -> Result<(), String> {
    let w = at(t, "name");
    need(is_id(t.attr("name")), || format!(r#"{w}: name is letters, digits, ".", "_" and "-""#))?;
    need(t.attr("type").is_none() || matches!(t.attr("type"), Some(Json::Str(s)) if FLOW_INPUT_TYPES.contains(&s.as_str())), || format!("{w}: type is one of {} (got {})", FLOW_INPUT_TYPES.join(", "), said(t.attr("type"))))?;
    for k in ["private", "required"] {
        need(t.attr(k).is_none() || matches!(t.attr(k), Some(Json::Bool(_))), || format!("{w}: {k} is true or false"))?;
    }
    need(!(matches!(t.attr("private"), Some(Json::Bool(true))) && t.attr("default").is_some()), || format!("{w}: a private input states no default: its value is bound by the run and never written in the document"))?;
    if let Some(d) = t.attr("default") {
        let ok = match t.attr("type").and_then(Json::as_str) {
            Some("number") => matches!(d, Json::Num(_)),
            Some("boolean") => matches!(d, Json::Bool(_)),
            _ => matches!(d, Json::Str(_)),
        };
        need(ok, || format!("{w}: default is a {} (the input's type)", t.attr("type").and_then(Json::as_str).unwrap_or("text")))?;
    }
    Ok(())
}

/// The placeholders in a text: `${input:name}` (a typed input) and `${env:NAME}` (the host's secret store).
fn placeholders(s: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut rest = s;
    while let Some(i) = rest.find("${") {
        let after = &rest[i + 2..];
        let Some(end) = after.find('}') else { break };
        let inner = &after[..end];
        out.push(match inner.split_once(':') {
            Some((k, n)) => (k.to_string(), n.to_string()),
            None => (String::new(), inner.to_string()),
        });
        rest = &after[end + 1..];
    }
    out
}

fn bound(value: Option<&Json>, w: &str, inputs: &[String]) -> Result<(), String> {
    let Some(Json::Str(s)) = value else { return Ok(()) };
    let declared = if inputs.is_empty() { "none declared".to_string() } else { inputs.join(", ") };
    for (kind, name) in placeholders(s) {
        let ok = (kind == "input" && inputs.contains(&name))
            || (kind == "env" && name.chars().next().is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
                && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_'));
        let shown = format!("${{{}{}}}", if kind.is_empty() { String::new() } else { format!("{kind}:") }, name);
        need(ok, || format!("{w}: {shown} is not a bound value: name a declared input ({declared}) as ${{input:name}}, or the secret store as ${{env:NAME}}"))?;
    }
    Ok(())
}

/// What an action step must state, per kind.
fn action_needs(kind: &str) -> &'static [&'static str] {
    match kind {
        "message" | "computer" => &["channel", "instruction"],
        "command" | "robot" => &["channel", "command"],
        "chain" | "workflow" => &["ref"],
        "decision" => &["instruction"],
        _ => &["ref", "instruction"],
    }
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
        need(t.attr("tool").is_none() && no_time && t.children.is_empty(), || format!("{w}: a note step names no tool, takes no time (minutes 0 or absent) and holds no param, measure, visit or expect"))?;
    }
    let kind = t.attr("kind").and_then(Json::as_str).unwrap_or("");
    let action = FLOW_ACTION_KINDS.contains(&kind);
    if action {
        let says = |n: &str| match n {
            "instruction" => "its instruction",
            "channel" => "its channel",
            "command" => "its command",
            _ => "what it calls (ref)",
        };
        let missing: Vec<&str> = action_needs(kind).iter().copied().filter(|n| t.attr(n).is_none()).map(says).collect();
        need(missing.is_empty(), || format!("{w}: a {kind} step states {}", missing.join(" and ")))?;
        need(t.attr("tool").is_none() && t.attr("by").is_none() && t.attr("fixture").is_none(), || format!("{w}: an action step names no tool, no fixture and no by: it acts on its channel (or calls what its ref names)"))?;
        need(!t.children.iter().any(|c| c.tag == "visit" || c.tag == "measure"), || format!("{w}: an action step takes no visit or measure: say what must be observed after it in an <expect>"))?;
        need(t.children.iter().any(|c| c.tag == "expect"), || format!("{w}: an action step declares what must be observed after it (<expect name equals>)"))?;
        need(t.attr("surprise").is_some(), || format!("{w}: an action step says where a surprise goes (surprise: parent, agent or person)"))?;
    }
    need(t.attr("surprise").is_none() || matches!(t.attr("surprise"), Some(Json::Str(s)) if FLOW_SURPRISE.contains(&s.as_str())), || format!("{w}: surprise is one of {} (got {})", FLOW_SURPRISE.join(", "), said(t.attr("surprise"))))?;
    let surprise = t.attr("surprise").and_then(Json::as_str);
    need(surprise.is_none() || surprise == Some("parent") || matches!(t.attr("surpriseTo"), Some(Json::Str(s)) if !s.trim().is_empty()), || format!("{w}: surprise {} names who handles it (surpriseTo: an agent or a person)", said(t.attr("surprise"))))?;
    need(surprise != Some("parent") || t.attr("surpriseTo").is_none(), || format!("{w}: a surprise that goes to the parent names no surpriseTo"))?;
    if t.attr("after").is_some() {
        let Some(Json::Str(raw)) = t.attr("after") else { return Err(format!("{w}: after names the steps this one follows (their ids, space separated)")) };
        need(!raw.trim().is_empty(), || format!("{w}: after names the steps this one follows (their ids, space separated)"))?;
        let ids: Vec<&str> = raw.split_whitespace().collect();
        for i in &ids {
            need(is_id(Some(&Json::Str(i.to_string()))), || format!("{w}: after names a step's id (got {i:?})"))?;
        }
        need(!ids.contains(&t.attr("id").and_then(Json::as_str).unwrap_or("")), || format!("{w}: after does not name the step itself"))?;
        let mut seen = std::collections::HashSet::new();
        need(ids.iter().all(|i| seen.insert(*i)), || format!("{w}: after names a step twice"))?;
    }
    let names: Vec<&Json> = t.children.iter().filter(|c| c.tag == "param").filter_map(|c| c.attr("name")).collect();
    if let Some(twice) = names.iter().enumerate().find(|(i, n)| names[..*i].contains(n)).map(|(_, n)| *n) {
        return Err(format!("{w}: two params are named {}", twice.text()));
    }
    let expects: Vec<&Json> = t.children.iter().filter(|c| c.tag == "expect").filter_map(|c| c.attr("name")).collect();
    if let Some(twice) = expects.iter().enumerate().find(|(i, n)| expects[..*i].contains(n)).map(|(_, n)| *n) {
        return Err(format!("{w}: two expects are named {}", twice.text()));
    }
    for c in &t.children {
        if c.tag == "param" {
            let pw = at(c, "name");
            need(matches!(c.attr("name"), Some(Json::Str(s)) if !s.trim().is_empty()), || format!("{pw}: name says what the parameter is"))?;
            need(matches!(c.attr("value"), Some(Json::Num(_) | Json::Str(_) | Json::Bool(_))), || format!("{pw}: value is a number, a text or true/false"))?;
            need(c.attr("unit").is_none() || matches!(c.attr("unit"), Some(Json::Str(s)) if !s.trim().is_empty()), || format!("{pw}: unit is a unit's symbol (rpm, sccm, mTorr, °C, s)"))?;
        } else if c.tag == "visit" {
            let vw = format!("{w} {}", at(c, "tool"));
            need(is_id(c.attr("tool")), || format!("{vw}: tool is the id of one of the flow's tools"))?;
            for k in ["minutes", "setupMinutes"] {
                need(c.attr(k).is_none() || num(c.attr(k)).is_some_and(|m| m >= 0.0), || format!("{vw}: {k} is a number of at least 0"))?;
            }
            money(c, &vw)?;
        } else if c.tag == "expect" {
            let ew = at(c, "name");
            need(matches!(c.attr("name"), Some(Json::Str(s)) if !s.trim().is_empty()), || format!("{ew}: name says what is observed (a screen, an exit code, a reply)"))?;
            need(matches!(c.attr("equals"), Some(Json::Num(_) | Json::Str(_) | Json::Bool(_))), || format!("{ew}: equals is the value the observation must have (a number, a text or true/false)"))?;
            need(c.attr("tolerance").is_none() || num(c.attr("tolerance")).is_some_and(|v| v >= 0.0), || format!("{ew}: tolerance is a number of at least 0"))?;
            need(c.attr("tolerance").is_none() || num(c.attr("equals")).is_some(), || format!("{ew}: a tolerance holds of a number: equals is not a number"))?;
            need(c.attr("within").is_none() || num(c.attr("within")).is_some_and(|v| v > 0.0), || format!("{ew}: within is a number of seconds, more than 0"))?;
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
    for c in t.children.iter().filter(|c| c.tag == "input") {
        input_ok(c)?;
    }
    let inputs: Vec<String> = t.children.iter().filter(|c| c.tag == "input").filter_map(|c| c.attr("name").and_then(Json::as_str).map(str::to_string)).collect();
    for c in &t.children {
        if c.tag == "tool" {
            tool_ok(c)?;
        } else if c.tag == "step" {
            step_ok(c)?;
            let sw = at(c, "id");
            for b in c.children.iter().filter(|b| b.tag == "param") {
                bound(b.attr("value"), &at(b, "name"), &inputs)?;
            }
            bound(c.attr("command"), &sw, &inputs)?;
            bound(c.attr("instruction"), &sw, &inputs)?;
        }
    }
    let step_ids: Vec<String> = t.children.iter().filter(|c| c.tag == "step").filter_map(|c| c.attr("id").and_then(Json::as_str).map(str::to_string)).collect();
    for c in t.children.iter().filter(|c| c.tag == "step") {
        if let Some(Json::Str(raw)) = c.attr("after") {
            for i in raw.split_whitespace() {
                need(step_ids.iter().any(|id| id == i), || format!("{}: after names the step {i:?}, which this flow does not declare", at(c, "id")))?;
            }
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
    let declared: Vec<Json> = t.children.iter().filter(|c| c.tag == "input").map(DocTree::record).collect();
    if !declared.is_empty() {
        out.set("inputs", declared);
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
            let visits: Vec<Json> = s.children.iter().filter(|c| c.tag == "visit").map(DocTree::record).collect();
            let expects: Vec<Json> = s.children.iter().filter(|c| c.tag == "expect").map(DocTree::record).collect();
            if !params.is_empty() {
                rec.set("params", params);
            }
            if !measures.is_empty() {
                rec.set("measures", measures);
            }
            if !visits.is_empty() {
                rec.set("visits", visits);
            }
            if !expects.is_empty() {
                rec.set("expects", expects);
            }
            rec
        })
        .collect();
    if !steps.is_empty() {
        out.set("steps", steps);
    }
    Ok(out)
}
