//! AN OPPORTUNITY AND AN EXPERIMENT IN RUST — a `<name>.opportunity.rs` declares one opportunity of the market manager
//! (a collaborator, a partnership or a listing; an application is its folder of `applications/` and a client is a
//! company's `Deal`), and a `<name>.experiment.rs` one experiment. The same elements, and the same documents, as the
//! TypeScript SDK's `opportunities.ts` (`.opportunity.tsx`) and the Python SDK's `opportunities.py`:
//!
//! ```
//! use commandagi::design::opportunities::*;
//!
//! fn document() -> El {
//!     opportunity([
//!         contact().ref_("../relationships/cards/fetch-ai.vcf").role("marketplace"),
//!         draft().id("listing").channel("agentverse").body("Plans, simulates and runs robots."),
//!     ])
//!     .title("CommandAGI's agent on Agentverse")
//!     .kind("listing")
//!     .stage("draft")
//! }
//! # commandagi::design::declare(document()).unwrap();
//! ```
//!
//! A stage is one of its kind's (`OPPORTUNITY_STAGES`); the last `move_()` names the stage it is in. A `draft()` is all
//! that exists until a principal approves it (`approved_by`, `approved_at`): a listing is never `submitted` or `live`
//! without an approved draft. An experiment's objectives are kept apart by kind (quality, distribution, impact); reach
//! alone is never an objective.

use super::documents::{declare_document, DocTree, TagRule, Vocabulary};
use super::element::{attributes, elements};
use super::{Declared, Family};
pub use super::{fragment, json, El, Json, Value};

elements! {
    /// An opportunity: its contacts, actions, evidence, moves and drafts as children.
    opportunity: children;
    /// A person or an organisation of it, by ref: `.ref_("people/ada.vcf").role("judge")`.
    contact: leaf;
    /// A next action: `.id("approve").title("Approve the text").owner("person:ada").decision(true)`.
    action: leaf;
    /// Where a fact was read: `.id("api").fact("…").source("https://…").way("read the page").by("agent:x").at("…")`.
    evidence: leaf;
    /// One recorded stage change: `.from("found").to("contacted").at("2026-11-08T10:00:00Z").by("person:ada")`.
    move_ = "move": leaf;
    /// A message, a post or a listing as drafted: `.id("listing").channel("agentverse").body("…")`.
    draft: leaf;
    /// An experiment: its arms, objectives and decision as children.
    experiment: children;
    /// One arm of an experiment: `.id("number").file("hooks/number.txt")`.
    arm: leaf;
    /// One objective: `.id("read").kind("quality").metric("read-through").direction("up")`.
    objective: leaf;
    /// The decision: `.arm("number").at("…").by("agent:x").reason("…")`.
    decision: leaf;
}

attributes! {
    /// The fields of an opportunity, an experiment and their children, chained on their elements.
    pub trait OpportunitiesAttrs {
        title; kind; stage; status; source; deadline; value; currency; eligibility; folder; owner; counterparty; channel; handle; about;
        ref_ = "ref"; role; note; id; due; decision; done; fact; way; by; at; from; to; subject; body; approved_by; approved_at;
        hypothesis; opportunity; started; ended; file; metric; direction; arm; reason;
    }
}

pub(crate) const FAMILY: Family = Family { module: "opportunities", roots: &["opportunity", "experiment"], declare };

pub const OPPORTUNITY_KINDS: &[&str] = &["collaborator", "partnership", "listing"];
/// Each kind's workflow: (kind, its open stages in order, the stages that close it).
pub const OPPORTUNITY_STAGES: &[(&str, &[&str], &[&str])] = &[
    ("collaborator", &["found", "contacted", "talking", "working"], &["ended", "dropped"]),
    ("partnership", &["found", "contacted", "negotiating", "active"], &["ended", "declined", "dropped"]),
    ("listing", &["draft", "approved", "submitted", "live"], &["withdrawn", "refused"]),
];
pub const PUBLIC_STAGES: &[&str] = &["submitted", "live"];
pub const OPPORTUNITY_STATUSES: &[&str] = &["open", "waiting", "blocked", "closed"];
pub const ELIGIBILITIES: &[&str] = &["eligible", "eligible-with-partner", "not-eligible", "unknown"];
pub const OPPORTUNITY_FIELDS: &[&str] = &["title", "kind", "stage", "status", "source", "deadline", "value", "currency", "eligibility", "folder", "owner", "counterparty", "channel", "handle", "about"];
pub const CONTACT_FIELDS: &[&str] = &["ref", "role", "note"];
pub const ACTION_FIELDS: &[&str] = &["id", "title", "due", "owner", "decision", "done"];
pub const EVIDENCE_FIELDS: &[&str] = &["id", "fact", "ref", "source", "way", "by", "at"];
pub const MOVE_FIELDS: &[&str] = &["from", "to", "at", "by", "note"];
pub const DRAFT_FIELDS: &[&str] = &["id", "channel", "to", "subject", "body", "approvedBy", "approvedAt"];
pub const EXPERIMENT_STATUSES: &[&str] = &["draft", "running", "decided", "stopped"];
pub const OBJECTIVE_KINDS: &[&str] = &["quality", "distribution", "impact"];
pub const EXPERIMENT_FIELDS: &[&str] = &["title", "hypothesis", "status", "opportunity", "channel", "started", "ended"];
pub const ARM_FIELDS: &[&str] = &["id", "file", "note"];
pub const OBJECTIVE_FIELDS: &[&str] = &["id", "kind", "metric", "direction", "note"];
pub const DECISION_FIELDS: &[&str] = &["arm", "at", "by", "reason"];

const OPPORTUNITY: Vocabulary = Vocabulary {
    format: "opportunity",
    noun: "an opportunity",
    root: "opportunity",
    tags: &[
        TagRule::new("opportunity", &[]).required(&["title", "kind", "stage"]).attrs(OPPORTUNITY_FIELDS),
        TagRule::new("contact", &["opportunity"]).key("ref").required(&["ref"]).attrs(CONTACT_FIELDS),
        TagRule::new("action", &["opportunity"]).key("id").required(&["id", "title"]).attrs(ACTION_FIELDS),
        TagRule::new("evidence", &["opportunity"]).key("id").required(&["id", "fact"]).attrs(EVIDENCE_FIELDS),
        TagRule::new("move", &["opportunity"]).key("at").required(&["to", "at", "by"]).attrs(MOVE_FIELDS),
        TagRule::new("draft", &["opportunity"]).key("id").required(&["id", "body"]).attrs(DRAFT_FIELDS),
    ],
    from_tree: opportunity_of,
};

const EXPERIMENT: Vocabulary = Vocabulary {
    format: "experiment",
    noun: "an experiment",
    root: "experiment",
    tags: &[
        TagRule::new("experiment", &[]).required(&["title"]).attrs(EXPERIMENT_FIELDS),
        TagRule::new("arm", &["experiment"]).key("id").required(&["id"]).attrs(ARM_FIELDS),
        TagRule::new("objective", &["experiment"]).key("id").required(&["id", "kind", "metric", "direction"]).attrs(OBJECTIVE_FIELDS),
        TagRule::new("decision", &["experiment"]).single().required(&["arm", "at", "by", "reason"]).attrs(DECISION_FIELDS),
    ],
    from_tree: experiment_of,
};

fn declare(root: El) -> Result<Declared, String> {
    declare_document(&root, if root.tag == "experiment" { &EXPERIMENT } else { &OPPORTUNITY })
}

/// JavaScript's `length` of a string: its UTF-16 code units.
fn js_len(s: &str) -> usize {
    s.encode_utf16().count()
}

fn is_text(v: Option<&Json>, max: usize) -> bool {
    matches!(v, Some(Json::Str(s)) if !s.trim().is_empty() && js_len(s) <= max)
}

fn said(v: Option<&Json>) -> String {
    v.map(Json::text).unwrap_or_else(|| "undefined".into())
}

fn str_of(v: Option<&Json>) -> Option<&str> {
    v.and_then(Json::as_str)
}

fn digits(s: &str) -> Option<i64> {
    if s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    s.parse().ok()
}

/// Days since 1970-01-01 of a civil date (Howard Hinnant's `days_from_civil`).
fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// The day `YYYY-MM-DD` as days since the epoch, or none.
fn civil_day(s: &str) -> Option<i64> {
    let b = s.as_bytes();
    if b.len() != 10 || b[4] != b'-' || b[7] != b'-' {
        return None;
    }
    let (y, m, d) = (digits(&s[0..4])?, digits(&s[5..7])?, digits(&s[8..10])?);
    ((1..=12).contains(&m) && (1..=31).contains(&d)).then(|| days_from_civil(y, m, d))
}

/// An ISO 8601 instant (`YYYY-MM-DDTHH:MM[:SS[.f]](Z|±HH:MM)`) as milliseconds since the epoch, or none.
fn instant_ms(s: &str) -> Option<f64> {
    if s.len() < 17 || s.as_bytes()[10] != b'T' {
        return None;
    }
    let day = civil_day(&s[..10])?;
    let rest = &s[11..];
    let (clock, zone) = match rest.find(|c| c == 'Z' || c == '+' || c == '-') {
        Some(i) => (&rest[..i], &rest[i..]),
        None => return None,
    };
    let parts: Vec<&str> = clock.split(':').collect();
    if parts.len() < 2 || parts.len() > 3 || parts[0].len() != 2 || parts[1].len() != 2 {
        return None;
    }
    let (h, mi) = (digits(parts[0])?, digits(parts[1])?);
    let mut sec = 0.0;
    if parts.len() == 3 {
        let (whole, frac) = parts[2].split_once('.').map_or((parts[2], None), |(w, f)| (w, Some(f)));
        if whole.len() != 2 {
            return None;
        }
        sec = digits(whole)? as f64;
        if let Some(f) = frac {
            digits(f)?;
            sec += format!("0.{f}").parse::<f64>().ok()?;
        }
    }
    if h > 23 || mi > 59 || sec >= 60.0 {
        return None;
    }
    let offset = if zone == "Z" {
        0
    } else {
        let z = zone.as_bytes();
        if z.len() != 6 || z[3] != b':' {
            return None;
        }
        let m = digits(&zone[1..3])? * 60 + digits(&zone[4..6])?;
        if z[0] == b'+' { m } else { -m }
    };
    Some(((day * 1440 + h * 60 + mi - offset) as f64) * 60_000.0 + sec * 1000.0)
}

fn is_day(v: Option<&Json>) -> bool {
    str_of(v).is_some_and(|s| civil_day(s).is_some() || instant_ms(s).is_some())
}

fn is_instant(v: Option<&Json>) -> bool {
    str_of(v).is_some_and(|s| instant_ms(s).is_some())
}

/// A principal as the authority records name one: `person:`, `agent:`, `org:` or `role:<id>`.
fn is_principal(v: Option<&Json>) -> bool {
    let Some(s) = str_of(v) else { return false };
    let Some(rest) = ["person:", "agent:", "org:", "role:"].iter().find_map(|p| s.strip_prefix(p)) else { return false };
    let b = rest.as_bytes();
    !b.is_empty() && b.len() <= 128 && b[0].is_ascii_alphanumeric() && b.iter().all(|c| c.is_ascii_alphanumeric() || b"._@-".contains(c))
}

fn is_id(v: Option<&Json>) -> bool {
    let Some(s) = str_of(v) else { return false };
    let b = s.as_bytes();
    !b.is_empty() && b.len() <= 64 && (b[0].is_ascii_lowercase() || b[0].is_ascii_digit()) && b.iter().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || *c == b'-')
}

fn is_decimal(s: &str) -> bool {
    let all_digits = |x: &str| !x.is_empty() && x.bytes().all(|b| b.is_ascii_digit());
    let (whole, frac) = s.split_once('.').map_or((s, None), |(w, f)| (w, Some(f)));
    all_digits(whole) && frac.map_or(true, all_digits)
}

fn in_list(v: Option<&Json>, list: &[&str]) -> bool {
    str_of(v).is_some_and(|s| list.contains(&s))
}

/// The stages of a kind, open then closed.
pub fn stages_of(kind: &str) -> Vec<&'static str> {
    OPPORTUNITY_STAGES.iter().find(|(k, _, _)| *k == kind).map(|(_, o, c)| [*o, *c].concat()).unwrap_or_default()
}

fn closing(kind: &str) -> &'static [&'static str] {
    OPPORTUNITY_STAGES.iter().find(|(k, _, _)| *k == kind).map(|(_, _, c)| *c).unwrap_or(&[])
}

/// Whether a stage closes its kind's workflow.
pub fn closes_kind(kind: &str, stage: &str) -> bool {
    closing(kind).contains(&stage)
}

/// The attributes of `fields` the tree states, in the fields' order, beside what `out` already holds.
fn keep(mut out: Json, t: &DocTree, fields: &[&str]) -> Json {
    for k in fields {
        if out.get(k).is_none() {
            if let Some(v) = t.attr_of(k) {
                out.set(k, v.clone());
            }
        }
    }
    out
}

/// Each child of `tag`, read by `of`, refused when one key is used twice.
fn many(t: &DocTree, tag: &str, key: &str, of: &dyn Fn(&DocTree) -> Result<Json, String>) -> Result<Json, String> {
    let mut seen: Vec<String> = Vec::new();
    let mut out = Vec::new();
    for c in t.children.iter().filter(|c| c.tag == tag) {
        let k = said(c.attr_of(key));
        if seen.contains(&k) {
            return Err(format!("{tag}().{key}({k}) is declared twice"));
        }
        seen.push(k);
        out.push(of(c)?);
    }
    Ok(Json::Arr(out))
}

fn check_texts(t: &DocTree, at: &str, keys: &[(&str, usize)]) -> Result<(), String> {
    for (k, max) in keys {
        if let Some(v) = t.attr_of(k) {
            if !is_text(Some(v), *max) {
                return Err(format!("{at} {k} is a short text, not {}", v.text()));
            }
        }
    }
    Ok(())
}

fn contact_of(t: &DocTree) -> Result<Json, String> {
    if !is_text(t.attr_of("ref"), 1000) {
        return Err(format!("contact(): ref names a card (a .vcf, by ref) or a relationship's id, not {}", said(t.attr_of("ref"))));
    }
    let at = format!("contact().ref_({})", said(t.attr_of("ref")));
    check_texts(t, &at, &[("role", 80), ("note", 2000)])?;
    Ok(keep(Json::obj(), t, CONTACT_FIELDS))
}

fn action_of(t: &DocTree) -> Result<Json, String> {
    if !is_id(t.attr_of("id")) {
        return Err(format!("action(): id is lower-case letters, digits and dashes (at most 64), not {}", said(t.attr_of("id"))));
    }
    let at = format!("action().id({})", said(t.attr_of("id")));
    if !is_text(t.attr_of("title"), 200) {
        return Err(format!("{at} title is a short text (at most 200), not {}", said(t.attr_of("title"))));
    }
    if t.attr_of("due").is_some() && !is_day(t.attr_of("due")) {
        return Err(format!("{at} due is an ISO 8601 day or instant, not {}", said(t.attr_of("due"))));
    }
    if t.attr_of("done").is_some() && !is_day(t.attr_of("done")) {
        return Err(format!("{at} done is the ISO 8601 day or instant it was done, not {}", said(t.attr_of("done"))));
    }
    if t.attr_of("owner").is_some() && !is_principal(t.attr_of("owner")) {
        return Err(format!("{at} owner is a principal (person:, agent:, org: or role:<id>), not {}", said(t.attr_of("owner"))));
    }
    if let Some(d) = t.attr_of("decision") {
        if !matches!(d, Json::Bool(_)) {
            return Err(format!("{at} decision is true when only its owner can decide it, not {}", d.text()));
        }
    }
    Ok(keep(Json::obj(), t, ACTION_FIELDS))
}

fn evidence_of(t: &DocTree) -> Result<Json, String> {
    if !is_id(t.attr_of("id")) {
        return Err(format!("evidence(): id is lower-case letters, digits and dashes (at most 64), not {}", said(t.attr_of("id"))));
    }
    let at = format!("evidence().id({})", said(t.attr_of("id")));
    if !is_text(t.attr_of("fact"), 2000) {
        return Err(format!("{at} fact says what it is evidence of, not {}", said(t.attr_of("fact"))));
    }
    if t.attr_of("ref").is_none() && t.attr_of("source").is_none() {
        return Err(format!("{at} names where the fact was read: a ref (a file) or a source (a URL)"));
    }
    check_texts(t, &at, &[("ref", 2000), ("source", 2000), ("way", 2000)])?;
    if t.attr_of("by").is_some() && !is_principal(t.attr_of("by")) {
        return Err(format!("{at} by is the principal that read it, not {}", said(t.attr_of("by"))));
    }
    if t.attr_of("at").is_some() && !is_instant(t.attr_of("at")) {
        return Err(format!("{at} at is the ISO 8601 instant it was read, not {}", said(t.attr_of("at"))));
    }
    Ok(keep(Json::obj(), t, EVIDENCE_FIELDS))
}

fn move_of(t: &DocTree, kind: &str) -> Result<Json, String> {
    let stages = stages_of(kind);
    if !is_instant(t.attr_of("at")) {
        return Err(format!("move_(): at is an ISO 8601 instant, not {}", said(t.attr_of("at"))));
    }
    let at = format!("move_().at({})", said(t.attr_of("at")));
    if !in_list(t.attr_of("to"), &stages) {
        return Err(format!("{at} to is a stage of a {kind} ({}), not {}", stages.join(", "), said(t.attr_of("to"))));
    }
    if t.attr_of("from").is_some() && !in_list(t.attr_of("from"), &stages) {
        return Err(format!("{at} from is a stage of a {kind}, not {}", said(t.attr_of("from"))));
    }
    if t.attr_of("from").is_some() && t.attr_of("from") == t.attr_of("to") {
        return Err(format!("{at} moves to the stage it is in ({})", str_of(t.attr_of("to")).unwrap_or("")));
    }
    if !is_principal(t.attr_of("by")) {
        return Err(format!("{at} by is the principal who moved it, not {}", said(t.attr_of("by"))));
    }
    check_texts(t, &at, &[("note", 2000)])?;
    Ok(keep(Json::obj(), t, MOVE_FIELDS))
}

fn draft_of(t: &DocTree) -> Result<Json, String> {
    if !is_id(t.attr_of("id")) {
        return Err(format!("draft(): id is lower-case letters, digits and dashes (at most 64), not {}", said(t.attr_of("id"))));
    }
    let at = format!("draft().id({})", said(t.attr_of("id")));
    if !is_text(t.attr_of("body"), 20000) {
        return Err(format!("{at} body is the text that would be sent or published (at most 20000), not {}", said(t.attr_of("body"))));
    }
    if t.attr_of("channel").is_some() && !is_id(t.attr_of("channel")) {
        return Err(format!("{at} channel is a channel's id, not {}", said(t.attr_of("channel"))));
    }
    check_texts(t, &at, &[("to", 1000), ("subject", 1000)])?;
    if t.attr_of("approvedBy").is_some() != t.attr_of("approvedAt").is_some() {
        return Err(format!("{at} approvedBy and approvedAt come together: who approved it, and when"));
    }
    if t.attr_of("approvedBy").is_some() && !is_principal(t.attr_of("approvedBy")) {
        return Err(format!("{at} approvedBy is the principal that approved it, not {}", said(t.attr_of("approvedBy"))));
    }
    if t.attr_of("approvedAt").is_some() && !is_instant(t.attr_of("approvedAt")) {
        return Err(format!("{at} approvedAt is an ISO 8601 instant, not {}", said(t.attr_of("approvedAt"))));
    }
    Ok(keep(Json::obj(), t, DRAFT_FIELDS))
}

/// An opportunity as its tree declares it (the TypeScript SDK's `opportunityOf`).
pub fn opportunity_of(t: &DocTree) -> Result<Json, String> {
    if !is_text(t.attr_of("title"), 160) {
        return Err(format!("opportunity(): title is a short text (at most 160), not {}", said(t.attr_of("title"))));
    }
    let at = format!("opportunity().title({})", said(t.attr_of("title")));
    let kind = match str_of(t.attr_of("kind")) {
        Some(k) if OPPORTUNITY_KINDS.contains(&k) => k.to_string(),
        other => {
            let home = match other {
                Some("application") => ": an application is its folder of applications/",
                Some("client") => ": a client is a <Deal> of its company",
                _ => "",
            };
            return Err(format!("{at} kind is {}, not {}{home}", OPPORTUNITY_KINDS.join(", "), said(t.attr_of("kind"))));
        }
    };
    let stages = stages_of(&kind);
    let Some(stage) = str_of(t.attr_of("stage")).filter(|s| stages.contains(s)).map(str::to_string) else {
        return Err(format!("{at} stage is a stage of a {kind} ({}), not {}", stages.join(", "), said(t.attr_of("stage"))));
    };
    let status = str_of(t.attr_of("status"));
    if t.attr_of("status").is_some() && !in_list(t.attr_of("status"), OPPORTUNITY_STATUSES) {
        return Err(format!("{at} status is {}, not {}", OPPORTUNITY_STATUSES.join(", "), said(t.attr_of("status"))));
    }
    if closes_kind(&kind, &stage) && status.is_some_and(|s| s != "closed") {
        return Err(format!("{at} is {stage}, which closes it: its status is closed, not {}", status.unwrap_or("")));
    }
    if status == Some("closed") && !closes_kind(&kind, &stage) {
        return Err(format!("{at} is closed, so its stage is one that closes a {kind} ({}), not {stage}", closing(&kind).join(", ")));
    }
    if t.attr_of("source").is_some() && !is_text(t.attr_of("source"), 2000) {
        return Err(format!("{at} source is where it was found (a URL or a ref), not {}", said(t.attr_of("source"))));
    }
    if t.attr_of("deadline").is_some() && !is_day(t.attr_of("deadline")) {
        return Err(format!("{at} deadline is an ISO 8601 day or instant, not {}", said(t.attr_of("deadline"))));
    }
    let mut out = Json::obj().with("title", t.attr_of("title").cloned().unwrap_or(Json::Null)).with("kind", Json::Str(kind.clone())).with("stage", Json::Str(stage.clone()));
    if let Some(v) = t.attr_of("value") {
        let text = match v {
            Json::Num(n) if n.is_finite() && *n >= 0.0 => Some(if n.fract() == 0.0 && n.abs() < 1e21 { format!("{}", *n as i64) } else { format!("{n}") }),
            Json::Str(s) => Some(s.clone()),
            _ => None,
        };
        let Some(text) = text.filter(|s| is_decimal(s)) else {
            return Err(format!("{at} value is a decimal amount (text or a number, not negative), not {}", v.text()));
        };
        let currency_ok = str_of(t.attr_of("currency")).is_some_and(|c| c.len() == 3 && c.bytes().all(|b| b.is_ascii_uppercase()));
        if !currency_ok {
            return Err(format!("{at} value needs its currency (three capitals, ISO 4217), not {}", said(t.attr_of("currency"))));
        }
        out.set("value", Json::Str(text));
    } else if t.attr_of("currency").is_some() {
        return Err(format!("{at} names a currency and no value"));
    }
    if t.attr_of("eligibility").is_some() && !in_list(t.attr_of("eligibility"), ELIGIBILITIES) {
        return Err(format!("{at} eligibility is {}, not {}", ELIGIBILITIES.join(", "), said(t.attr_of("eligibility"))));
    }
    for k in ["folder", "counterparty"] {
        if t.attr_of(k).is_some() && !is_text(t.attr_of(k), 1000) {
            return Err(format!("{at} {k} is a ref, not {}", said(t.attr_of(k))));
        }
    }
    if t.attr_of("owner").is_some() && !is_principal(t.attr_of("owner")) {
        return Err(format!("{at} owner is a principal (person:, agent:, org: or role:<id>), not {}", said(t.attr_of("owner"))));
    }
    if t.attr_of("channel").is_some() && !is_id(t.attr_of("channel")) {
        return Err(format!("{at} channel is a channel's id, not {}", said(t.attr_of("channel"))));
    }
    if t.attr_of("handle").is_some() && !is_text(t.attr_of("handle"), 300) {
        return Err(format!("{at} handle is its name on its channel (a GitHub org or owner/repo, an agent's address, a domain), not {}", said(t.attr_of("handle"))));
    }
    if t.attr_of("handle").is_some() && t.attr_of("channel").is_none() {
        return Err(format!("{at} names a handle and no channel"));
    }
    if t.attr_of("about").is_some() && !is_text(t.attr_of("about"), 8000) {
        return Err(format!("{at} about is a text (at most 8000)"));
    }
    let mut out = keep(std::mem::replace(&mut out, Json::Null), t, OPPORTUNITY_FIELDS);
    out.set("contacts", many(t, "contact", "ref", &contact_of)?);
    out.set("actions", many(t, "action", "id", &action_of)?);
    out.set("evidence", many(t, "evidence", "id", &evidence_of)?);
    let moves = many(t, "move", "at", &|m| move_of(m, &kind))?;
    let last = match &moves {
        Json::Arr(list) => list.iter().filter_map(|m| Some((instant_ms(m.get("at")?.as_str()?)?, m))).max_by(|a, b| a.0.total_cmp(&b.0)).map(|(_, m)| m.clone()),
        _ => None,
    };
    out.set("moves", moves);
    let drafts = many(t, "draft", "id", &draft_of)?;
    let approved = matches!(&drafts, Json::Arr(list) if list.iter().any(|d| d.get("approvedBy").is_some()));
    out.set("drafts", drafts);
    if let Some(m) = last {
        let to = m.get("to").and_then(Json::as_str).unwrap_or("");
        if to != stage {
            return Err(format!("{at} is {stage}, but its last move_() ({}) went to {to}", m.get("at").and_then(Json::as_str).unwrap_or("")));
        }
    }
    if kind == "listing" && PUBLIC_STAGES.contains(&stage.as_str()) && !approved {
        return Err(format!("{at} is {stage}: a listing is public only after a principal approved its draft() (approvedBy, approvedAt)"));
    }
    Ok(out)
}

fn arm_of(t: &DocTree) -> Result<Json, String> {
    if !is_id(t.attr_of("id")) {
        return Err(format!("arm(): id is lower-case letters, digits and dashes (at most 64), not {}", said(t.attr_of("id"))));
    }
    let at = format!("arm().id({})", said(t.attr_of("id")));
    check_texts(t, &at, &[("file", 2000), ("note", 2000)])?;
    Ok(keep(Json::obj(), t, ARM_FIELDS))
}

fn objective_of(t: &DocTree) -> Result<Json, String> {
    if !is_id(t.attr_of("id")) {
        return Err(format!("objective(): id is lower-case letters, digits and dashes (at most 64), not {}", said(t.attr_of("id"))));
    }
    let at = format!("objective().id({})", said(t.attr_of("id")));
    if !in_list(t.attr_of("kind"), OBJECTIVE_KINDS) {
        return Err(format!("{at} kind is {}, not {}", OBJECTIVE_KINDS.join(", "), said(t.attr_of("kind"))));
    }
    if !is_text(t.attr_of("metric"), 120) {
        return Err(format!("{at} metric names what is measured, not {}", said(t.attr_of("metric"))));
    }
    if !in_list(t.attr_of("direction"), &["up", "down"]) {
        return Err(format!("{at} direction is up or down, not {}", said(t.attr_of("direction"))));
    }
    check_texts(t, &at, &[("note", 2000)])?;
    Ok(keep(Json::obj(), t, OBJECTIVE_FIELDS))
}

/// An experiment as its tree declares it (the TypeScript SDK's `experimentOf`).
pub fn experiment_of(t: &DocTree) -> Result<Json, String> {
    if !is_text(t.attr_of("title"), 160) {
        return Err(format!("experiment(): title is a short text (at most 160), not {}", said(t.attr_of("title"))));
    }
    let at = format!("experiment().title({})", said(t.attr_of("title")));
    if t.attr_of("hypothesis").is_some() && !is_text(t.attr_of("hypothesis"), 2000) {
        return Err(format!("{at} hypothesis is a text (at most 2000), not {}", said(t.attr_of("hypothesis"))));
    }
    if t.attr_of("status").is_some() && !in_list(t.attr_of("status"), EXPERIMENT_STATUSES) {
        return Err(format!("{at} status is {}, not {}", EXPERIMENT_STATUSES.join(", "), said(t.attr_of("status"))));
    }
    if t.attr_of("opportunity").is_some() && !is_text(t.attr_of("opportunity"), 1000) {
        return Err(format!("{at} opportunity is a ref to its .opportunity.tsx, not {}", said(t.attr_of("opportunity"))));
    }
    if t.attr_of("channel").is_some() && !is_id(t.attr_of("channel")) {
        return Err(format!("{at} channel is a channel's id, not {}", said(t.attr_of("channel"))));
    }
    for k in ["started", "ended"] {
        if t.attr_of(k).is_some() && !is_day(t.attr_of(k)) {
            return Err(format!("{at} {k} is an ISO 8601 day or instant, not {}", said(t.attr_of(k))));
        }
    }
    let mut out = keep(Json::obj().with("title", t.attr_of("title").cloned().unwrap_or(Json::Null)), t, EXPERIMENT_FIELDS);
    let arms = many(t, "arm", "id", &arm_of)?;
    let arm_ids: Vec<String> = match &arms {
        Json::Arr(list) => list.iter().filter_map(|a| a.get("id").and_then(Json::as_str).map(str::to_string)).collect(),
        _ => vec![],
    };
    out.set("arms", arms);
    let objectives = many(t, "objective", "id", &objective_of)?;
    // Reach alone rewards whatever spreads: an experiment that measures only distribution has no objective.
    if let Json::Arr(list) = &objectives {
        if !list.is_empty() && list.iter().all(|o| o.get("kind").and_then(Json::as_str) == Some("distribution")) {
            return Err(format!("{at} measures only distribution: name a quality or an impact objective too"));
        }
    }
    out.set("objectives", objectives);
    let decisions: Vec<&DocTree> = t.children.iter().filter(|c| c.tag == "decision").collect();
    if decisions.len() > 1 {
        return Err(format!("{at} has one decision(), not {}", decisions.len()));
    }
    let status = str_of(t.attr_of("status"));
    if let Some(d) = decisions.first() {
        if !str_of(d.attr_of("arm")).is_some_and(|a| arm_ids.iter().any(|x| x == a)) {
            return Err(format!("{at} decision() arm is one of its arms ({}), not {}", arm_ids.join(", "), said(d.attr_of("arm"))));
        }
        if !is_instant(d.attr_of("at")) {
            return Err(format!("{at} decision() at is an ISO 8601 instant, not {}", said(d.attr_of("at"))));
        }
        if !is_principal(d.attr_of("by")) {
            return Err(format!("{at} decision() by is the principal that decided, not {}", said(d.attr_of("by"))));
        }
        if !is_text(d.attr_of("reason"), 2000) {
            return Err(format!("{at} decision() reason says why, from the observations, not {}", said(d.attr_of("reason"))));
        }
        if status != Some("decided") {
            return Err(format!("{at} has a decision(), so its status is decided, not {}", said(t.attr_of("status"))));
        }
        out.set("decision", keep(Json::obj(), d, DECISION_FIELDS));
    } else if status == Some("decided") {
        return Err(format!("{at} is decided: its decision() says which arm, who, when and why"));
    }
    Ok(out)
}
