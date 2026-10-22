//! A THREAD'S MOUNTED TASKS IN RUST — a `mounts.rs` in the thread's run folder. The same elements, and the same
//! document, as the TypeScript SDK's `mounts.ts` (`mounts.tsx`) and the Python SDK's `mounts.py`. The file declares one
//! `mounts`, and each `mount` is one record of it, its fields the element's attributes:
//!
//! ```
//! use commandagi::design::mounts::*;
//!
//! fn document() -> El {
//!     mounts([
//!         mount().label("launch").folder("../../Tasks/launch"),
//!         mount().label("ship it").task("../../Tasks/Ship it.task.tsx"),
//!         mount().label("owner inbox").assignee("user:u_1").status(["todo", "doing", "blocked"]),
//!     ])
//! }
//! # commandagi::design::declare(document()).unwrap();
//! ```
//!
//! `label` names the mount in its thread (unique there). What it mounts is one task (`task`, a ref to a `.task.tsx`),
//! or a list: the tasks at or under a folder (`folder`), the tasks of a project (`project`, a ref to a `.project.tsx`),
//! or a query (`assignee`, `labels`, `status`), each narrowing the others. A ref is a path relative to the run folder,
//! or an absolute address. A run gives back `{format: "mounts", document: {mounts: […]}, sources}` beside an empty
//! graph; `sources` names each mount's site by its label (`mount#launch`).

use super::documents::{declare_document, DocTree, TagRule, Vocabulary};
use super::element::{attributes, elements};
use super::{Declared, Family};
pub use super::{fragment, json, El, Json, Value};

elements! {
    /// A thread's mounts: one `mount()` each.
    mounts: children;
    /// One mount: `.label("launch").folder("../../Tasks/launch")`.
    mount: leaf;
}

attributes! {
    /// The fields of a mount, chained on its element.
    pub trait MountsAttrs {
        label; task; folder; project; assignee; labels; status;
    }
}

pub(crate) const FAMILY: Family = Family { module: "mounts", roots: &["mounts"], declare };

pub const MOUNT_FIELDS: &[&str] = &["label", "task", "folder", "project", "assignee", "labels", "status"];
/// The task statuses a mount's `status` may name.
pub const MOUNT_STATUSES: &[&str] = &["proposed", "todo", "doing", "blocked", "done", "cancelled", "declined"];

const MOUNTS: Vocabulary = Vocabulary {
    format: "mounts",
    noun: "a thread's mounted tasks",
    root: "mounts",
    tags: &[
        TagRule::new("mounts", &[]).attrs(&[]),
        TagRule::new("mount", &["mounts"]).key("label").required(&["label"]).attrs(MOUNT_FIELDS),
    ],
    from_tree: mounts_of,
};

fn declare(root: El) -> Result<Declared, String> {
    declare_document(&root, &MOUNTS)
}

/// JavaScript's `length` of a string: its UTF-16 code units.
fn js_len(s: &str) -> usize {
    s.encode_utf16().count()
}

fn is_text(v: &Json, max: usize) -> bool {
    matches!(v, Json::Str(s) if !s.trim().is_empty() && js_len(s) <= max)
}

fn is_texts(v: &Json) -> bool {
    matches!(v, Json::Arr(items) if items.iter().all(|x| is_text(x, 200)))
}

/// `user:<id>` or `agent:<id>`: an id with no space and no colon.
fn assignee_ok(v: &Json) -> bool {
    let Json::Str(s) = v else { return false };
    let id = s.strip_prefix("user:").or_else(|| s.strip_prefix("agent:"));
    matches!(id, Some(id) if !id.is_empty() && !id.contains(|c: char| c == ':' || c.is_whitespace()))
}

/// One mount as its element declares it, or why it does not read (the TypeScript SDK's `mountOf`). A field written
/// `json("null")` is written, as a JSX attribute written `{null}` is: only an absent one is not.
pub fn mount_of(t: &DocTree) -> Result<Json, String> {
    let label = t.attr("label");
    let Some(Json::Str(name)) = label.filter(|v| is_text(v, 200)) else {
        let said = label.map(Json::text).unwrap_or_else(|| "undefined".into());
        return Err(format!("mount(): label is a short text that names it in its thread, not {said}"));
    };
    let at = format!("mount().label({})", Json::Str(name.clone()).text());
    for k in ["task", "folder", "project", "assignee"] {
        if let Some(v) = t.attr(k).filter(|v| !is_text(v, 1000)) {
            return Err(format!("{at} {k} is a text (a ref or an address), not {}", v.text()));
        }
    }
    if t.attr("labels").is_some_and(|v| !is_texts(v)) {
        return Err(format!("{at} labels is a list of texts"));
    }
    if let Some(v) = t.attr("status") {
        let known = matches!(v, Json::Arr(items) if items.iter().all(|s| matches!(s, Json::Str(s) if MOUNT_STATUSES.contains(&s.as_str()))));
        if !is_texts(v) || !known {
            return Err(format!("{at} status is a list of {}", MOUNT_STATUSES.join(", ")));
        }
    }
    if let Some(v) = t.attr("assignee").filter(|v| !assignee_ok(v)) {
        return Err(format!("{at} assignee is user:<id> or agent:<id>, not {}", v.text()));
    }
    let query = ["folder", "project", "assignee", "labels", "status"].iter().any(|k| t.attr(k).is_some());
    match (t.attr("task").is_some(), query) {
        (true, true) => return Err(format!("{at} mounts one task, or a list (folder, project, assignee, labels, status): not both")),
        (false, false) => return Err(format!("{at} mounts nothing: give task, folder, project, or a query (assignee, labels, status)")),
        _ => {}
    }
    let mut out = Json::obj();
    for k in MOUNT_FIELDS {
        match t.attr(k) {
            Some(Json::Arr(items)) if items.is_empty() => {}
            Some(v) => {
                out.set(k, v.clone());
            }
            None => {}
        }
    }
    Ok(out)
}

fn mounts_of(t: &DocTree) -> Result<Json, String> {
    let mut mounts = Vec::new();
    let mut seen: Vec<String> = Vec::new();
    for c in t.children.iter().filter(|c| c.tag == "mount") {
        let m = mount_of(c)?;
        let label = m.get("label").and_then(Json::as_str).unwrap_or_default().to_string();
        if seen.contains(&label) {
            return Err(format!("two mounts are labelled {}: a label names one mount in its thread", Json::Str(label).text()));
        }
        seen.push(label);
        mounts.push(m);
    }
    Ok(Json::obj().with("mounts", Json::Arr(mounts)))
}
