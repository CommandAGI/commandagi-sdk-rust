//! TASKS AND PROJECTS IN RUST — a `.task.rs` and a `.project.rs`, written as the elements the tasks and projects apps
//! edit. The same elements, and the same documents, as the TypeScript SDK's `tasks.ts` and the Python SDK's `tasks.py`:
//!
//! ```
//! use commandagi::design::tasks::*;
//!
//! fn document() -> El {
//!     task([subtask().ref_("Ship it/Draft.task.rs")])
//!         .id("ship")
//!         .title("Ship it")
//!         .status("doing")
//!         .due_at("2026-10-20")
//! }
//! # commandagi::design::declare(document()).unwrap();
//! ```
//!
//! The rule of the ontology's files: a record is an element and its fields are the element's attributes. A task's
//! subtasks are `subtask().ref_(…)` children (each its own file, by path from this one), in order; a project's views
//! are `view()` children. Times (`start_at`, `due_at`, `closed_at`, `expires`, `approved_at`, `created_at`,
//! `updated_at`) are epoch milliseconds in the document and may be written as a date (`"2026-10-20"`, UTC midnight) or a
//! UTC time (`"2026-10-20T14:30:00Z"`). Nothing adds a default. A run gives back `{format, document, sources}` beside an
//! empty graph; `sources` names each element's site by its path (`""`, `subtask#Ship it/Draft.task.rs`, `view#board`).
//!
//! A proposal is a task with status `proposed`: `sketch().ref_(…)` names the file that shows what it will do, and each
//! `asks()` has the shape of one standing grant (`world`, `units`, `channels`, `verbs`; or `folder` and `files`; or
//! `folder`, `venues`, `symbols`, `max_qty`, `max_notional`, `per_day`; each with `hours` and `note`):
//!
//! ```
//! use commandagi::design::tasks::*;
//!
//! fn document() -> El {
//!     task([
//!         sketch().ref_("../worlds/shop/runs/cut/run.json"),
//!         asks().world("../worlds/shop/world.tsx").units(["cnc-1"]).channels(["gcode"]).verbs(["open", "arm", "run"]).hours(4),
//!     ])
//!     .id("cut")
//!     .title("Cut the bracket")
//!     .status("proposed")
//!     .expires("2026-10-11T18:00:00Z")
//! }
//! # commandagi::design::declare(document()).unwrap();
//! ```

use super::documents::{declare_document, DocTree, TagRule, Vocabulary};
use super::element::{attributes, elements, fn_name, rust_name};
use super::{Declared, Family};
pub use super::{fragment, json, El, Json, Value};

elements! {
    /// A task: its sketch, its asks and its subtasks.
    task: children;
    /// A proposal's sketch: the file that shows what it will do, by its path from this file.
    sketch: leaf;
    /// One thing a proposal asks for, the shape of one standing grant.
    asks: leaf;
    /// One file a change proposal changes: `.file("../cad/Plate.3d.rs").base("<sha256>").proposed("Thinner/Plate.3d.rs")`.
    change: leaf;
    /// A subtask: the file of a task, by its path from this file: `subtask().ref_("Ship it/Draft.task.rs")`.
    subtask: leaf;
    /// A project: its views.
    project: children;
    /// A view of a project: `.id("board").mode("board").group_by("status")`.
    view: leaf;
}

attributes! {
    /// The fields of a task, a project and a view, chained on their elements.
    pub trait TasksAttrs {
        id; title; status; priority; glyph; assignees; projects; blocked_by; labels; start_at; due_at; closed_at; thread_id;
        created_by; readme; created_at; updated_at; expires; approved_by; approved_at; grants;
        /// A subtask's or a sketch's file, by its path from this file.
        ref_;
        world; units; channels; verbs; folder; files; venues; symbols; max_qty; max_notional; per_day; hours; note;
        file; base; proposed;
        name; default_view_id; subprojects; archived;
        mode; group_by; filter; lane_order;
    }
}

pub(crate) const FAMILY: Family = Family { module: "tasks", roots: &["task", "project"], declare };

/// The fields of a task's body (packages/domain/core/src/task.ts `TaskDoc`), but its sketch, its asks and its subtasks.
pub const TASK_FIELDS: &[&str] = &[
    "id", "title", "status", "priority", "glyph", "assignees", "projects", "blockedBy", "labels", "startAt", "dueAt", "closedAt", "threadId",
    "createdBy", "readme", "expires", "approvedBy", "approvedAt", "grants", "createdAt", "updatedAt",
];
/// The fields of one `asks()`: the shape of one standing grant (a world's units, a folder's files, a folder's venues).
pub const ASK_FIELDS: &[&str] = &[
    "world", "units", "channels", "verbs", "folder", "files", "venues", "symbols", "maxQty", "maxNotional", "perDay", "hours", "note",
];
/// The fields of one `change()` of a change proposal: the file it edits, the version it was made from, the edited copy.
pub const CHANGE_FIELDS: &[&str] = &["file", "base", "proposed"];
/// The fields of a project's body (`ProjectDoc`), but its views.
pub const PROJECT_FIELDS: &[&str] = &["id", "name", "glyph", "defaultViewId", "subprojects", "files", "archived", "readme", "createdAt", "updatedAt"];
/// The fields of a project's view (`TaskView`).
pub const VIEW_FIELDS: &[&str] = &["id", "name", "mode", "groupBy", "filter", "laneOrder"];
/// The fields that are times: epoch milliseconds in the document, a date or a UTC time in code.
pub const TIME_FIELDS: &[&str] = &["startAt", "dueAt", "closedAt", "expires", "approvedAt", "createdAt", "updatedAt"];

const TASK: Vocabulary = Vocabulary {
    format: "task",
    noun: "a task",
    root: "task",
    tags: &[
        TagRule::new("task", &[]).required(&["id"]).attrs(TASK_FIELDS),
        TagRule::new("sketch", &["task"]).single().required(&["ref"]).attrs(&["ref"]),
        TagRule::new("asks", &["task"]).attrs(ASK_FIELDS),
        TagRule::new("change", &["task"]).key("file").required(&["file", "base", "proposed"]).attrs(CHANGE_FIELDS),
        TagRule::new("subtask", &["task"]).key("ref").required(&["ref"]).attrs(&["ref"]),
    ],
    from_tree: task_of,
};

const PROJECT: Vocabulary = Vocabulary {
    format: "project",
    noun: "a project",
    root: "project",
    tags: &[
        TagRule::new("project", &[]).required(&["id"]).attrs(PROJECT_FIELDS),
        TagRule::new("view", &["project"]).key("id").required(&["id"]).attrs(VIEW_FIELDS),
    ],
    from_tree: project_of,
};

fn declare(root: El) -> Result<Declared, String> {
    declare_document(&root, if root.tag == "task" { &TASK } else { &PROJECT })
}

/// Days from 1970-01-01 to a proleptic Gregorian date (a day past its month's end runs into the next, as `Date.parse`).
fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// The epoch ms of `YYYY-MM-DD(THH:MM(:SS(.fff…)?)?(Z|±HH:MM))?`, as JavaScript's `Date.parse` reads it, or none.
fn parse_time(s: &str) -> Option<i64> {
    let b = s.as_bytes();
    let digits = |from: usize, n: usize| -> Option<i64> {
        let part = b.get(from..from + n)?;
        part.iter().all(u8::is_ascii_digit).then(|| part.iter().fold(0, |a, c| a * 10 + (c - b'0') as i64))
    };
    let (year, month, day) = (digits(0, 4)?, digits(5, 2)?, digits(8, 2)?);
    if b.get(4) != Some(&b'-') || b.get(7) != Some(&b'-') || !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }
    let date = days_from_civil(year, month, day) * 86_400_000;
    if b.len() == 10 {
        return Some(date);
    }
    if b.get(10) != Some(&b'T') || b.get(13) != Some(&b':') {
        return None;
    }
    let (hour, minute) = (digits(11, 2)?, digits(14, 2)?);
    let mut at = 16;
    let (mut second, mut ms) = (0, 0);
    if b.get(at) == Some(&b':') {
        second = digits(at + 1, 2)?;
        at += 3;
        if b.get(at) == Some(&b'.') {
            let start = at + 1;
            at = start;
            while b.get(at).is_some_and(u8::is_ascii_digit) {
                at += 1;
            }
            if at == start {
                return None;
            }
            // The first three digits are the milliseconds; the rest are dropped.
            let frac = &s[start..at];
            ms = format!("{:0<3}", &frac[..frac.len().min(3)]).parse::<i64>().ok()?;
        }
    }
    let offset = match b.get(at) {
        Some(b'Z') if at + 1 == b.len() => 0,
        Some(&sign @ (b'+' | b'-')) if at + 6 == b.len() && b[at + 3] == b':' => {
            let (oh, om) = (digits(at + 1, 2)?, digits(at + 4, 2)?);
            if oh > 23 || om > 59 {
                return None;
            }
            (oh * 60 + om) * 60_000 * if sign == b'+' { 1 } else { -1 }
        }
        _ => return None,
    };
    let at_24 = hour == 24 && minute == 0 && second == 0 && ms == 0;
    if (hour > 23 && !at_24) || minute > 59 || second > 59 {
        return None;
    }
    Some(date + ((hour * 60 + minute) * 60 + second) * 1000 + ms - offset)
}

/// A time as the document holds it: epoch ms from an integer, a date or a UTC time.
pub fn time_of(value: &Json, what: &str) -> Result<Json, String> {
    match value {
        Json::Num(n) if n.is_finite() && n.fract() == 0.0 => return Ok(value.clone()),
        Json::Str(s) => {
            if let Some(ms) = parse_time(s) {
                return Ok(Json::Num(ms as f64));
            }
        }
        _ => {}
    }
    Err(format!("{what} is a date (\"2026-10-20\"), a UTC time (\"2026-10-20T14:30:00Z\") or epoch milliseconds, not {}", value.text()))
}

/// How a message names an element: `task().id("ship")`.
fn call(t: &DocTree) -> String {
    match t.attr_of("id") {
        Some(v @ Json::Str(_)) => format!("{}().id({})", fn_name(t.tag), v.text()),
        _ => format!("{}()", fn_name(t.tag)),
    }
}

/// The attributes of an element as a record's fields: times as epoch ms.
fn fields_of(t: &DocTree) -> Result<Json, String> {
    let at = call(t);
    let mut out = Json::obj();
    for (k, v) in &t.attrs {
        out.set(k, if TIME_FIELDS.contains(&k.as_str()) { time_of(v, &format!("{at} {}", rust_name(k)))? } else { v.clone() });
    }
    Ok(out)
}

fn task_of(t: &DocTree) -> Result<Json, String> {
    let mut subtasks = Vec::new();
    for c in t.children.iter().filter(|c| c.tag == "subtask") {
        match c.attr_of("ref") {
            Some(r @ Json::Str(s)) if !s.is_empty() => subtasks.push(r.clone()),
            other => {
                let said = other.map(Json::text).unwrap_or_else(|| "undefined".into());
                return Err(format!("subtask(): ref_ is a path to a task's file, not {said}"));
            }
        }
    }
    let mut out = fields_of(t)?;
    if let Some(s) = t.single("sketch") {
        match s.attr_of("ref") {
            Some(r @ Json::Str(text)) if !text.is_empty() => {
                out.set("sketch", r.clone());
            }
            other => {
                let said = other.map(Json::text).unwrap_or_else(|| "undefined".into());
                return Err(format!("sketch(): ref_ is a path to the file that shows the work, not {said}"));
            }
        }
    }
    let asks: Vec<Json> = t.children.iter().filter(|c| c.tag == "asks").map(DocTree::record).collect();
    if !asks.is_empty() {
        out.set("asks", Json::Arr(asks));
    }
    let changes: Vec<Json> = t.children.iter().filter(|c| c.tag == "change").map(DocTree::record).collect();
    for c in &changes {
        for k in ["file", "proposed"] {
            if !matches!(c.get(k), Some(Json::Str(s)) if !s.is_empty()) {
                let said = c.get(k).map(Json::text).unwrap_or_else(|| "undefined".into());
                return Err(format!("change(): {k} is a path from this file, not {said}"));
            }
        }
        if !matches!(c.get("base"), Some(Json::Str(d)) if d.len() == 64 && d.chars().all(|ch| ch.is_ascii_hexdigit() && !ch.is_ascii_uppercase())) {
            let said = c.get("base").map(Json::text).unwrap_or_else(|| "undefined".into());
            return Err(format!("change(): base is the content id (SHA-256, hex) of the version the edit was made from, not {said}"));
        }
    }
    if !changes.is_empty() {
        out.set("changes", Json::Arr(changes));
    }
    if !subtasks.is_empty() {
        out.set("subtasks", Json::Arr(subtasks));
    }
    Ok(out)
}

fn project_of(t: &DocTree) -> Result<Json, String> {
    let views: Vec<Json> = t.children.iter().filter(|c| c.tag == "view").map(DocTree::record).collect();
    let mut out = fields_of(t)?;
    if !views.is_empty() {
        out.set("views", Json::Arr(views));
    }
    Ok(out)
}
