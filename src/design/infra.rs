//! INFRASTRUCTURE IN RUST — an account's own machines declared as code. The same elements, and the same documents, as
//! the TypeScript SDK's `infra.ts` and the Python SDK's `infra.py`. One folder is one thing: `infra/vpn/<name>/exit.rs`
//! one VPN exit, `infra/sessions/<name>/session.rs` one logged-in state, `infra/vantages/<name>/vantage.rs` one observer:
//!
//! ```
//! use commandagi::design::infra::*;
//!
//! fn document() -> El {
//!     exit().name("fsn-1").provider("hetzner").region("fsn1").country("DE").continent("EU").size("cax11")
//!         .image("debian-12").max_monthly_usd(6.99)
//! }
//! # commandagi::design::declare(document()).unwrap();
//! ```
//!
//! ```
//! use commandagi::design::infra::*;
//!
//! fn document() -> El {
//!     session([uses().monitor("../../../social/x/nvidia")])
//!         .name("x-web").site("https://x.com").kind("browser-profile").profile("x-web").refreshed("2026-10-12")
//! }
//! # commandagi::design::declare(document()).unwrap();
//! ```
//!
//! ```
//! use commandagi::design::infra::*;
//!
//! fn document() -> El {
//!     vantage().name("fsn-1").path("worker").exit("fsn-1")
//!         .location(json(r#"{"city": "Falkenstein", "country": "DE"}"#)).locale("de-DE").timezone("Europe/Berlin")
//! }
//! # commandagi::design::declare(document()).unwrap();
//! ```
//!
//! Nothing adds a default: what the file does not say, the document does not have. [`exit_with_defaults`],
//! [`session_with_defaults`] and [`vantage_with_defaults`] fill it as its reader does. A session's file holds no
//! credential. A vantage is a device, a place and a session, never a fake person, so the vocabulary has no words for
//! one: a field it does not list is refused by name.
//!
//! The CommandAGI platform runs only a TSX file: its monitors read `vantage.tsx` and `session.tsx`, and `make
//! infra-plan` reads `exit.tsx`. A `.rs` file runs in the local sandbox.

use super::documents::{declare_document, js_string, DocTree, TagRule, Vocabulary};
use super::element::{attributes, elements};
use super::{Declared, Family};
pub use super::{fragment, json, El, Json, Value};

elements! {
    /// One VPN exit: a machine at a provider that carries the overlay's traffic out.
    exit: leaf;
    /// One observer of the account: a device, a place and a session.
    vantage: leaf;
    /// One logged-in state: `uses()` for each monitor that reads through it.
    session: children;
    /// A monitor that reads through a session: `.monitor("../../../social/x/nvidia")`.
    uses: leaf;
}

attributes! {
    /// The fields of an exit, a vantage and a session, chained on the element.
    pub trait InfraAttrs {
        name; provider; region; country; continent; size; image; protocol; enabled; price; max_monthly_usd;
        path; host; unit; snapshot; exit; location; locale; timezone; session; device; clock;
        site; kind; profile; refreshed; note; monitor;
    }
}

pub(crate) const FAMILY: Family = Family { module: "infra", roots: &["exit", "session", "vantage"], declare };

pub const EXIT_FIELDS: &[&str] = &["name", "provider", "region", "country", "continent", "size", "image", "protocol", "enabled", "price", "maxMonthlyUsd"];
/// The providers the reconciler can drive.
pub const INFRA_PROVIDERS: &[&str] = &["hetzner", "vultr"];
pub const EXIT_PROTOCOLS: &[&str] = &["wireguard"];
pub const CONTINENTS: &[&str] = &["AF", "AN", "AS", "EU", "NA", "OC", "SA"];

pub const VANTAGE_FIELDS: &[&str] = &["name", "path", "host", "unit", "snapshot", "exit", "location", "locale", "timezone", "session", "device", "clock", "enabled"];
/// Where a vantage observes from: a browser, a VM, an Android VM, a phone, the api worker, or a host.
pub const VANTAGE_PATHS: &[&str] = &["browser", "vm", "android-vm", "phone", "worker", "host"];
pub const LOCATION_FIELDS: &[&str] = &["city", "region", "country", "zip", "lat", "lon", "precision"];
/// How exactly a vantage's location is known: from a point to a whole country.
pub const LOCATION_PRECISIONS: &[&str] = &["point", "zip", "city", "region", "country"];
pub const DEVICE_FIELDS: &[&str] = &["kind", "os", "browser", "model", "carrier"];

pub const SESSION_FIELDS: &[&str] = &["name", "site", "kind", "host", "profile", "snapshot", "refreshed", "note"];
pub const SESSION_KINDS: &[&str] = &["browser-profile", "vm-snapshot", "android-snapshot"];
/// The source path (`via`) that reads through each kind of state.
pub const SESSION_VIA: &[(&str, &str)] = &[("browser-profile", "browser"), ("vm-snapshot", "vm"), ("android-snapshot", "android-vm")];

const EXIT: Vocabulary = Vocabulary {
    format: "exit",
    noun: "a VPN exit",
    root: "exit",
    tags: &[TagRule::new("exit", &[]).required(&["name", "provider", "region", "country", "size", "image"]).attrs(EXIT_FIELDS)],
    from_tree: exit_of,
};
const VANTAGE: Vocabulary = Vocabulary {
    format: "vantage",
    noun: "a vantage",
    root: "vantage",
    tags: &[TagRule::new("vantage", &[]).required(&["name", "path"]).attrs(VANTAGE_FIELDS)],
    from_tree: vantage_of,
};
const SESSION: Vocabulary = Vocabulary {
    format: "session",
    noun: "a logged-in state",
    root: "session",
    tags: &[
        TagRule::new("session", &[]).required(&["name", "site", "kind"]).attrs(SESSION_FIELDS),
        TagRule::new("uses", &["session"]).key("monitor").required(&["monitor"]).attrs(&["monitor"]),
    ],
    from_tree: session_of,
};

fn declare(root: El) -> Result<Declared, String> {
    match root.tag {
        "exit" => declare_document(&root, &EXIT),
        "vantage" => declare_document(&root, &VANTAGE),
        _ => declare_document(&root, &SESSION),
    }
}

// Each check below is one of the TypeScript SDK's patterns, written out: this crate has no dependencies.

/// `^[a-z0-9][a-z0-9-]{0,47}$`.
fn is_name(s: &str) -> bool {
    let bytes = s.as_bytes();
    (1..=48).contains(&bytes.len()) && bytes[0] != b'-' && bytes.iter().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || *b == b'-')
}

/// `^[A-Z]{2}$`.
fn is_country(s: &str) -> bool {
    s.len() == 2 && s.bytes().all(|b| b.is_ascii_uppercase())
}

/// `^([A-Z]{2}|[a-z0-9][a-z0-9-]{2,47})$`.
fn is_exit_ref(s: &str) -> bool {
    is_country(s) || (s.len() >= 3 && is_name(s))
}

/// `^[a-z]{2,3}(-[A-Za-z0-9]{2,8})*$`.
fn is_locale(s: &str) -> bool {
    let mut parts = s.split('-');
    let language = parts.next().unwrap_or_default();
    (2..=3).contains(&language.len()) && language.bytes().all(|b| b.is_ascii_lowercase()) && parts.all(|p| (2..=8).contains(&p.len()) && p.bytes().all(|b| b.is_ascii_alphanumeric()))
}

/// `^[A-Za-z]+(\/[A-Za-z0-9_+-]+)*$`.
fn is_timezone(s: &str) -> bool {
    let mut parts = s.split('/');
    let area = parts.next().unwrap_or_default();
    !area.is_empty() && area.bytes().all(|b| b.is_ascii_alphabetic()) && parts.all(|p| !p.is_empty() && p.bytes().all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'+' | b'-')))
}

/// `^\d{4}-(0[1-9]|1[0-2])-(0[1-9]|[12]\d|3[01])$`.
fn is_day(s: &str) -> bool {
    let b = s.as_bytes();
    let digits = |r: std::ops::Range<usize>| b[r].iter().all(u8::is_ascii_digit);
    if b.len() != 10 || b[4] != b'-' || b[7] != b'-' || !digits(0..4) || !digits(5..7) || !digits(8..10) {
        return false;
    }
    let month = (b[5] - b'0') * 10 + (b[6] - b'0');
    let day = (b[8] - b'0') * 10 + (b[9] - b'0');
    (1..=12).contains(&month) && (1..=31).contains(&day)
}

/// What JavaScript's `\s` matches: not Rust's `char::is_whitespace`, which differs on U+0085 and U+FEFF.
fn is_js_space(c: char) -> bool {
    matches!(c, '\t' | '\n' | '\u{b}' | '\u{c}' | '\r' | ' ' | '\u{a0}' | '\u{1680}' | '\u{2000}'..='\u{200a}' | '\u{2028}' | '\u{2029}' | '\u{202f}' | '\u{205f}' | '\u{3000}' | '\u{feff}')
}

/// `^https:\/\/[^\s/]+(\/\S*)?$`.
fn is_site(s: &str) -> bool {
    let Some(rest) = s.strip_prefix("https://") else { return false };
    let host = rest.split('/').next().unwrap_or_default();
    !host.is_empty() && !rest.contains(is_js_space)
}

fn at(t: &DocTree) -> String {
    let name = t.attr_of("name").and_then(Json::as_str).map(|n| format!(" name=\"{n}\"")).unwrap_or_default();
    format!("<{}{name}>", t.tag)
}

fn is_word(v: &Json) -> bool {
    matches!(v, Json::Str(s) if !s.is_empty())
}

fn word(t: &DocTree, k: &str) -> Result<(), String> {
    match t.attr_of(k).filter(|v| !is_word(v)) {
        Some(v) => Err(format!("{} {k} is a word, not {}", at(t), v.text())),
        None => Ok(()),
    }
}

fn one_of(t: &DocTree, k: &str, values: &[&str]) -> Result<(), String> {
    match t.attr_of(k).filter(|v| !v.as_str().is_some_and(|s| values.contains(&s))) {
        Some(v) => {
            let said: Vec<String> = values.iter().map(|x| format!("\"{x}\"")).collect();
            Err(format!("{} {k} is {}, not {}", at(t), said.join(", "), v.text()))
        }
        None => Ok(()),
    }
}

fn enabled_of(t: &DocTree) -> Result<(), String> {
    match t.attr_of("enabled").filter(|v| !matches!(v, Json::Bool(_))) {
        Some(v) => Err(format!("{} enabled is true or false, not {}", at(t), v.text())),
        None => Ok(()),
    }
}

/// Written as an attribute of a word: a field that is not a string fell to [`word`] first.
fn text<'a>(t: &'a DocTree, k: &str) -> &'a str {
    t.attr_of(k).and_then(Json::as_str).unwrap_or_default()
}

/// The fields the file said, in the order of `fields`.
fn said(t: &DocTree, fields: &[&str]) -> Json {
    let mut out = Json::obj();
    for k in fields {
        if let Some(v) = t.attr_of(k) {
            out.set(k, v.clone());
        }
    }
    out
}

fn exit_of(t: &DocTree) -> Result<Json, String> {
    for k in ["name", "region", "size", "image", "price"] {
        word(t, k)?;
    }
    if !is_name(text(t, "name")) {
        return Err(format!("{} name is lower-case letters, digits and -, at most 48", at(t)));
    }
    one_of(t, "provider", INFRA_PROVIDERS)?;
    one_of(t, "protocol", EXIT_PROTOCOLS)?;
    one_of(t, "continent", CONTINENTS)?;
    let country = t.attr_of("country");
    if !is_country(&js_string(country)) {
        return Err(format!("{} country is an ISO 3166 code in capitals (\"DE\"), not {}", at(t), country.map(Json::text).unwrap_or_default()));
    }
    enabled_of(t)?;
    if let Some(v) = t.attr_of("maxMonthlyUsd").filter(|v| !matches!(v, Json::Num(n) if *n > 0.0)) {
        return Err(format!("{} maxMonthlyUsd is a price in USD (6.99), not {}", at(t), v.text()));
    }
    Ok(said(t, EXIT_FIELDS))
}

/// An object attribute's fields, each one the vocabulary lists.
fn only_keys<'a>(t: &'a DocTree, k: &str, allowed: &[&str]) -> Result<Option<&'a [(String, Json)]>, String> {
    let Some(v) = t.attr_of(k) else { return Ok(None) };
    let Json::Obj(entries) = v else {
        return Err(format!("{} {k} is an object, not {}", at(t), v.text()));
    };
    if let Some((key, _)) = entries.iter().find(|(key, _)| !allowed.contains(&key.as_str())) {
        return Err(format!("{} {k}.{key} is not read ({k} has {}; a vantage has no fields for a person)", at(t), allowed.join(", ")));
    }
    Ok(Some(entries))
}

fn field<'a>(entries: &'a [(String, Json)], k: &str) -> Option<&'a Json> {
    entries.iter().find(|(key, _)| key == k).map(|(_, v)| v)
}

fn vantage_of(t: &DocTree) -> Result<Json, String> {
    for k in ["name", "host", "unit", "snapshot", "exit", "locale", "timezone", "session"] {
        word(t, k)?;
    }
    if !is_name(text(t, "name")) {
        return Err(format!("{} name is the folder's word: lower-case letters, digits and -, at most 48", at(t)));
    }
    one_of(t, "path", VANTAGE_PATHS)?;
    let path = text(t, "path");
    let written = |k: &str| t.attr_of(k).is_some();
    if path != "worker" && path != "host" && !written("host") {
        return Err(format!("{} a {path} is driven by a host: name it (host=\"laptop-linux\")", at(t)));
    }
    if path == "worker" && written("host") {
        return Err(format!("{} the worker's path has no host", at(t)));
    }
    if written("unit") && path != "phone" {
        return Err(format!("{} unit names a phone's unit in a world, for path=\"phone\" only", at(t)));
    }
    if written("snapshot") && path != "vm" && path != "android-vm" {
        return Err(format!("{} snapshot is for a VM", at(t)));
    }
    if written("exit") && !is_exit_ref(text(t, "exit")) {
        return Err(format!("{} exit is an exit's name (fsn-1) or a country (DE), not {}", at(t), Json::from(text(t, "exit")).text()));
    }
    if written("locale") && !is_locale(text(t, "locale")) {
        return Err(format!("{} locale is a BCP 47 tag (en-US), not {}", at(t), Json::from(text(t, "locale")).text()));
    }
    if written("timezone") && !is_timezone(text(t, "timezone")) {
        return Err(format!("{} timezone is an IANA zone (America/Chicago), not {}", at(t), Json::from(text(t, "timezone")).text()));
    }
    if let Some(loc) = only_keys(t, "location", LOCATION_FIELDS)? {
        if field(loc, "country").is_some_and(|v| !is_country(&js_string(Some(v)))) {
            return Err(format!("{} location.country is an ISO 3166 code in capitals (US)", at(t)));
        }
        for (k, lo, hi) in [("lat", -90.0, 90.0), ("lon", -180.0, 180.0)] {
            if field(loc, k).is_some_and(|v| !matches!(v, Json::Num(n) if *n >= lo && *n <= hi)) {
                return Err(format!("{} location.{k} is a number from {lo} to {hi}", at(t)));
            }
        }
        if field(loc, "precision").is_some_and(|v| !v.as_str().is_some_and(|s| LOCATION_PRECISIONS.contains(&s))) {
            return Err(format!("{} location.precision is {}", at(t), LOCATION_PRECISIONS.join(", ")));
        }
        for k in ["city", "region", "zip"] {
            if field(loc, k).is_some_and(|v| !is_word(v)) {
                return Err(format!("{} location.{k} is a word", at(t)));
            }
        }
    }
    if let Some(device) = only_keys(t, "device", DEVICE_FIELDS)? {
        if let Some((k, _)) = device.iter().find(|(_, v)| !is_word(v)) {
            return Err(format!("{} device.{k} is a word", at(t)));
        }
    }
    if t.attr_of("clock").is_some_and(|v| !matches!(v, Json::Obj(e) if matches!(e.as_slice(), [(k, Json::Num(sd))] if k == "sd" && *sd >= 0.0))) {
        return Err(format!("{} clock is {{ sd }} in ms (the observer's clock: NTP is tens of ms)", at(t)));
    }
    enabled_of(t)?;
    Ok(said(t, VANTAGE_FIELDS))
}

fn session_of(t: &DocTree) -> Result<Json, String> {
    for k in SESSION_FIELDS {
        word(t, k)?;
    }
    if !is_name(text(t, "name")) {
        return Err(format!("{} name is lower-case letters, digits and -, at most 48", at(t)));
    }
    if !is_site(text(t, "site")) {
        return Err(format!("{} site is an https address (\"https://x.com\"), not {}", at(t), Json::from(text(t, "site")).text()));
    }
    one_of(t, "kind", SESSION_KINDS)?;
    let kind = text(t, "kind");
    let browser = kind == "browser-profile";
    let written = |k: &str| t.attr_of(k).is_some();
    if browser && !written("profile") {
        return Err(format!("{} a browser-profile names its profile", at(t)));
    }
    if !browser && !written("snapshot") {
        return Err(format!("{} a {kind} names its snapshot", at(t)));
    }
    if browser && written("snapshot") {
        return Err(format!("{} a browser-profile has a profile, not a snapshot", at(t)));
    }
    if !browser && written("profile") {
        return Err(format!("{} a {kind} has a snapshot, not a profile", at(t)));
    }
    if written("profile") && !is_name(text(t, "profile")) {
        return Err(format!("{} profile is lower-case letters, digits and -, at most 48", at(t)));
    }
    if written("refreshed") && !is_day(text(t, "refreshed")) {
        return Err(format!("{} refreshed is a day (\"2026-10-12\"), not {}", at(t), Json::from(text(t, "refreshed")).text()));
    }
    let uses: Vec<Json> = t.children.iter().map(|c| Json::obj().with("monitor", c.attr_of("monitor").cloned().unwrap_or(Json::Null))).collect();
    let mut out = said(t, SESSION_FIELDS);
    out.set("uses", Json::Arr(uses));
    Ok(out)
}

/// Each field in `fields`, null when the document does not have it.
fn filled(d: &Json, fields: &[&str]) -> Json {
    let mut out = Json::obj();
    for k in fields {
        out.set(k, d.get(k).cloned().unwrap_or(Json::Null));
    }
    out
}

/// An exit as the reconciler reads it: what the file does not say, filled.
pub fn exit_with_defaults(d: &Json) -> Json {
    let mut out = filled(d, EXIT_FIELDS);
    out.set("protocol", d.get("protocol").cloned().unwrap_or_else(|| Json::from("wireguard")));
    out.set("enabled", d.get("enabled").cloned().unwrap_or(Json::Bool(true)));
    out
}

/// A vantage as the platform reads it: what the file does not say, filled (`enabled` true, the rest null).
pub fn vantage_with_defaults(d: &Json) -> Json {
    let mut out = filled(d, VANTAGE_FIELDS);
    out.set("enabled", d.get("enabled").cloned().unwrap_or(Json::Bool(true)));
    out
}

/// A session as a source reads it: what the file does not say, filled.
pub fn session_with_defaults(d: &Json) -> Json {
    let uses = match d.get("uses") {
        Some(Json::Arr(items)) => items.iter().filter_map(|u| u.get("monitor").cloned()).collect(),
        _ => Vec::new(),
    };
    let mut out = filled(d, SESSION_FIELDS);
    out.set("uses", Json::Arr(uses));
    out
}
