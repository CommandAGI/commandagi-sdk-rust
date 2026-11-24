//! AN AGENT'S SKILL IN RUST — a `skill.rs` in the skill's own folder (`skills/<name>/skill.rs`). The same elements,
//! and the same document, as the TypeScript SDK's `skills.ts` (`skill.tsx`) and the Python SDK's `skills.py`. A
//! skill's name is its folder's name, so the file never names itself. A skill holds no phrases and no utterance
//! examples: free text is read only by a model agent with tools, which chooses a skill and gives its args as
//! structured values:
//!
//! ```
//! use commandagi::design::skills::*;
//!
//! fn document() -> El {
//!     skill([
//!         step().tool("read_file").args(json(r#"{"path": "{file}"}"#)).as_("read"),
//!         step().expr(r#"count(filter(lines(read.text), contains(it, "error")))"#).as_("n"),
//!         reply().text("{file} has {n} lines that say error."),
//!     ])
//!     .about("How many lines of a file say error.")
//! }
//! # commandagi::design::declare(document()).unwrap();
//! ```
//!
//! A skill's args are the `{name}` placeholders its steps read that no earlier step binds with `as_` (`file` above).
//! What a skill holds, each a child in the order written: `step()` (exactly one of `expr`, `tool`, `script`, `skill`
//! or `see`; `args`; `as_` binds its value; `always`) and `reply().text(…)`; on `skill` itself `about` and `origin`.
//! A literal script beside it declares `result().value(…)`. Structure only: the expressions are read, and a skill is
//! admitted or rejected, by the agent's runtime (`packages/domain/world/symbolic/skills.js` in the CommandAGI
//! repository).

use super::documents::{declare_document, DocTree, TagRule, Vocabulary};
use super::element::{attributes, elements};
use super::{Declared, Family};
pub use super::{fragment, json, El, Json, Value};

elements! {
    /// One skill: its steps and reply, in the order written.
    skill: children;
    /// What it does: exactly one of `.expr`, `.tool`, `.script`, `.skill` or `.see`.
    step: leaf;
    /// What it says, with `{bindings}` filled in: `.text("{n}")`.
    reply: leaf;
    /// What a literal script declares: `.value(…)`, the value its step binds.
    result: leaf;
}

attributes! {
    /// The fields of a skill and of its children, chained on their elements.
    pub trait SkillsAttrs {
        about; origin;
        expr; tool; script; skill_ = "skill"; see; args; as_; always; image; template; against;
        text; value;
    }
}

pub(crate) const FAMILY: Family = Family { module: "skills", roots: &["skill", "result"], declare };

pub const SKILL_FIELDS: &[&str] = &["about", "origin"];
pub const STEP_KINDS: &[&str] = &["expr", "tool", "script", "skill", "see"];
pub const STEP_FIELDS: &[&str] = &["expr", "tool", "script", "skill", "see", "args", "as", "always", "image", "template", "against"];

const SKILL: Vocabulary = Vocabulary {
    format: "skill",
    noun: "a skill",
    root: "skill",
    tags: &[
        TagRule::new("skill", &[]).attrs(SKILL_FIELDS),
        TagRule::new("step", &["skill"]).attrs(STEP_FIELDS),
        TagRule::new("reply", &["skill"]).single().required(&["text"]).attrs(&["text"]),
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
    let reply = parts("reply").next();
    if let Some(r) = reply {
        text(r, "text")?;
    }
    let mut out = said(t, SKILL_FIELDS).with("steps", Json::Arr(steps));
    if let Some(r) = reply {
        out.set("reply", r.attr_of("text").cloned().unwrap_or(Json::Null));
    }
    Ok(out)
}

fn result_of(t: &DocTree) -> Result<Json, String> {
    if !t.children.is_empty() {
        return Err("<result> has no children; its value is its .value(…)".into());
    }
    Ok(Json::obj().with("value", t.attr_of("value").cloned().unwrap_or(Json::Null)))
}
