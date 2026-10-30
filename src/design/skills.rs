//! A SYMBOLIC AGENT'S SKILL IN RUST — a `skill.rs` in the skill's own folder (`skills/<name>/skill.rs`). The same
//! elements, and the same document, as the TypeScript SDK's `skills.ts` (`skill.tsx`) and the Python SDK's
//! `skills.py`. A skill's name is its folder's name, so the file never names itself:
//!
//! ```
//! use commandagi::design::skills::*;
//!
//! fn document() -> El {
//!     skill([
//!         phrase().match_("count [the] errors in {file:path}"),
//!         step().tool("read_file").args(json(r#"{"path": "{file}"}"#)).as_("read"),
//!         step().expr(r#"count(filter(lines(read.text), contains(it, "error")))"#).as_("n"),
//!         reply().text("{file} has {n} lines that say error."),
//!     ])
//!     .about("How many lines of a file say error.")
//! }
//! # commandagi::design::declare(document()).unwrap();
//! ```
//!
//! What a skill holds, each a child in the order written: `phrase().match_(…)`, `step()` (exactly one of `expr`,
//! `tool`, `script`, `skill` or `see`; `args`; `as_` binds its value; `always`), `reply().text(…)`,
//! `rule().if_(…).then(…)`, `fact().is_(…)` and `example().say(…).reply(…)`; on `skill` itself `about`, `needs`,
//! `gives`, `removes` and `origin`. A literal script beside it declares `result().value(…)`. Structure only: the
//! phrases, expressions and facts are read, and a skill is admitted or rejected, by the agent's runtime
//! (`packages/domain/world/symbolic` in the CommandAGI repository).

use super::documents::{declare_document, DocTree, TagRule, Vocabulary};
use super::element::{attributes, elements};
use super::{Declared, Family};
pub use super::{fragment, json, El, Json, Value};

elements! {
    /// One skill: its phrases, steps, reply, rules, facts and examples, in the order written.
    skill: children;
    /// How it is asked for: `.match_("count [the] errors in {file:path}")`.
    phrase: leaf;
    /// What it does: exactly one of `.expr`, `.tool`, `.script`, `.skill` or `.see`.
    step: leaf;
    /// What it says, with `{bindings}` filled in: `.text("{n}")`.
    reply: leaf;
    /// A rule it knows: `.if_(["unit(?u, ?d)"]).then("known(?u)")`.
    rule: leaf;
    /// A fact it knows: `.is_("ready(cam)")`.
    fact: leaf;
    /// An utterance and the reply it must give: the skill's own check.
    example: leaf;
    /// What a literal script declares: `.value(…)`, the value its step binds.
    result: leaf;
}

attributes! {
    /// The fields of a skill and of its children, chained on their elements.
    pub trait SkillsAttrs {
        about; needs; gives; removes; origin;
        match_; expr; tool; script; skill_ = "skill"; see; args; as_; always; image; template; against;
        text; if_; then; name; is_; say; reply_ = "reply"; value;
    }
}

pub(crate) const FAMILY: Family = Family { module: "skills", roots: &["skill", "result"], declare };

pub const SKILL_FIELDS: &[&str] = &["about", "needs", "gives", "removes", "origin"];
pub const STEP_KINDS: &[&str] = &["expr", "tool", "script", "skill", "see"];
pub const STEP_FIELDS: &[&str] = &["expr", "tool", "script", "skill", "see", "args", "as", "always", "image", "template", "against"];
pub const RULE_FIELDS: &[&str] = &["if", "then", "name"];

const SKILL: Vocabulary = Vocabulary {
    format: "skill",
    noun: "a skill",
    root: "skill",
    tags: &[
        TagRule::new("skill", &[]).attrs(SKILL_FIELDS),
        TagRule::new("phrase", &["skill"]).key("match").required(&["match"]).attrs(&["match"]),
        TagRule::new("step", &["skill"]).attrs(STEP_FIELDS),
        TagRule::new("reply", &["skill"]).single().required(&["text"]).attrs(&["text"]),
        TagRule::new("rule", &["skill"]).required(&["if", "then"]).attrs(RULE_FIELDS),
        TagRule::new("fact", &["skill"]).key("is").required(&["is"]).attrs(&["is"]),
        TagRule::new("example", &["skill"]).key("say").required(&["say", "reply"]).attrs(&["say", "reply"]),
    ],
    from_tree: skill_of,
};

const RESULT: Vocabulary = Vocabulary {
    format: "result",
    noun: "a script's result",
    root: "result",
    tags: &[TagRule::new("result", &[]).required(&["value"]).attrs(&["value"])],
    from_tree: result_of,
};

fn declare(root: El) -> Result<Declared, String> {
    declare_document(&root, if root.tag == "skill" { &SKILL } else { &RESULT })
}

fn is_text(v: &Json) -> bool {
    matches!(v, Json::Str(s) if !s.is_empty())
}

fn is_facts(v: &Json) -> bool {
    matches!(v, Json::Arr(items) if items.iter().all(is_text))
}

fn text(t: &DocTree, k: &str) -> Result<(), String> {
    match t.attr_of(k) {
        Some(v) if !is_text(v) => Err(format!("<{}> {k} is text, not {}", t.tag, v.text())),
        _ => Ok(()),
    }
}

/// The attributes `keys` names that are written, in that order (a field written `json("null")` is left out, as TypeScript
/// leaves out `undefined`).
fn said(t: &DocTree, keys: &[&str]) -> Json {
    let mut out = Json::obj();
    for k in keys {
        match t.attr_of(k) {
            None | Some(Json::Null) => {}
            Some(v) => {
                out.set(k, v.clone());
            }
        }
    }
    out
}

/// A skill as its elements declare it, or why it does not read (the TypeScript SDK's `skillOf`).
fn skill_of(t: &DocTree) -> Result<Json, String> {
    for k in ["about", "origin"] {
        text(t, k)?;
    }
    for k in ["needs", "gives", "removes"] {
        if let Some(v) = t.attr_of(k).filter(|v| !is_facts(v)) {
            return Err(format!("<skill> {k} is a list of facts ([\"at(?unit, home)\"]), not {}", v.text()));
        }
    }
    let parts = |tag: &'static str| t.children.iter().filter(move |c| c.tag == tag);
    let mut steps = Vec::new();
    for (i, s) in parts("step").enumerate() {
        let kinds: Vec<&str> = STEP_KINDS.iter().copied().filter(|k| s.attr_of(k).is_some()).collect();
        if kinds.len() != 1 {
            let has = if kinds.is_empty() { String::new() } else { format!(" (it has {})", kinds.join(" and ")) };
            return Err(format!("<step> {} is exactly one of {}{has}", i + 1, STEP_KINDS.join(", ")));
        }
        for k in ["expr", "tool", "script", "skill", "see", "as", "image", "template", "against"] {
            text(s, k)?;
        }
        if let Some(v) = s.attr_of("always").filter(|v| !matches!(v, Json::Bool(_))) {
            return Err(format!("<step> {}: always is true or false, not {}", i + 1, v.text()));
        }
        if let Some(v) = s.attr_of("args").filter(|v| !matches!(v, Json::Obj(_))) {
            return Err(format!("<step> {}: args is an object ({{ path: \"{{file}}\" }}), not {}", i + 1, v.text()));
        }
        steps.push(said(s, STEP_FIELDS));
    }
    let mut rules = Vec::new();
    for r in parts("rule") {
        let cond = r.attr_of("if");
        let reads = match cond {
            Some(v @ Json::Str(_)) => is_text(v),
            Some(Json::Arr(items)) => !items.is_empty() && items.iter().all(is_text),
            _ => false,
        };
        if !reads {
            return Err(format!("<rule> if is a fact or a list of facts, not {}", cond.map(Json::text).unwrap_or_else(|| "undefined".into())));
        }
        text(r, "then")?;
        text(r, "name")?;
        rules.push(said(r, RULE_FIELDS));
    }
    let reply = parts("reply").next();
    if let Some(r) = reply {
        text(r, "text")?;
    }
    let mut phrases = Vec::new();
    for p in parts("phrase") {
        text(p, "match")?;
        phrases.push(Json::obj().with("match", p.attr_of("match").cloned().unwrap_or(Json::Null)));
    }
    let mut facts = Vec::new();
    for f in parts("fact") {
        text(f, "is")?;
        facts.push(f.attr_of("is").cloned().unwrap_or(Json::Null));
    }
    let mut examples = Vec::new();
    for e in parts("example") {
        match (e.attr_of("say"), e.attr_of("reply")) {
            (Some(say @ Json::Str(s)), Some(rep @ Json::Str(_))) if !s.is_empty() => {
                examples.push(Json::obj().with("say", say.clone()).with("reply", rep.clone()));
            }
            _ => return Err("<example> is say=\"…\" reply=\"…\"".into()),
        }
    }
    let mut out = said(t, SKILL_FIELDS).with("phrases", Json::Arr(phrases)).with("steps", Json::Arr(steps));
    if let Some(r) = reply {
        out.set("reply", r.attr_of("text").cloned().unwrap_or(Json::Null));
    }
    Ok(out.with("rules", Json::Arr(rules)).with("facts", Json::Arr(facts)).with("examples", Json::Arr(examples)))
}

fn result_of(t: &DocTree) -> Result<Json, String> {
    if !t.children.is_empty() {
        return Err("<result> has no children; its value is its .value(…)".into());
    }
    Ok(Json::obj().with("value", t.attr_of("value").cloned().unwrap_or(Json::Null)))
}
