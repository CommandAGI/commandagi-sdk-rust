//! A CONNECTOR IN RUST — a `<name>.connector.rs`: a remote API declared as a folder tree, so a mount can list it and
//! read its files. The same elements, and the same document, as the TypeScript SDK's `connectors.ts`
//! (`<name>.connector.tsx`) and the Python SDK's `connectors.py`:
//!
//! ```
//! use commandagi::design::connectors::*;
//!
//! fn document() -> El {
//!     connector(
//!         [
//!             folder([dir().name("models")]).path(""),
//!             folder([each([dir().name("{a}").title("{a/name}")]).in_("$").as_("a")]).path("models").get("/assets?type=models"),
//!         ],
//!     )
//!     .name("Poly Haven")
//!     .base("https://api.polyhaven.com")
//!     .rate(2)
//!     .cache(86400)
//!     .licence("CC0-1.0")
//! }
//! # commandagi::design::declare(document()).unwrap();
//! ```
//!
//! Elements: `connector` (`name`, `base`, `rate`, `cache`, `home`, `terms`, `robots`, `hosts`, and every entry's `title`,
//! `author`, `licence`, `source`, `credit`), `credential`, `licence()` (`value` to `spdx`), `folder` (`path`, `get`,
//! `decode`, `page`, `page_size`, `total`, `next`), `request`, `here`, `each` (`in_`, `as_`, `where_`, `is`, `not_`: the JSX
//! attributes `in`, `as`, `where`, `is` and `not`), `dir` and `file`. A text attribute is a template (`{a}`, `{a/name}`, `{slug}`,
//! `|keys`, `|join:<sep>`, `|base`, `|stem`, `|ext`, `|text`, `|name`, `|lower`, `|first`, `|unique`; `{@}` is the folder's own
//! entry). A connector that would make an impolite or unreadable request is refused with why.

use super::documents::{declare_document, js_string, DocTree, TagRule, Vocabulary};
use super::element::{attributes, elements, fn_name};
use super::{Declared, Family};
pub use super::{fragment, json, El, Json, Value};

elements! {
    /// A connector: its `folder()`s, an optional `credential()` and its `licence()`s.
    connector: children;
    /// The service account's field sent with each request.
    credential: leaf;
    /// How an entry's licence is named: `.value("CC0").spdx("CC0-1.0")`.
    licence: leaf;
    /// One level of the tree.
    folder: children;
    /// Another request a folder's entries read, bound to `as_`.
    request: leaf;
    /// The fields of the folder itself.
    here: leaf;
    /// For each value at `in_`, bound to `as_`.
    each: children;
    /// A folder of the level.
    dir: leaf;
    /// A file of the level.
    file: leaf;
}

attributes! {
    /// The fields of a connector's elements, chained on them.
    pub trait ConnectorsAttrs {
        name; id; base; rate; cache; home; terms; robots; hosts;
        title; author; licence; source; credit;
        service; field; query; header; prefix; optional;
        value; spdx;
        path; get; decode; page; page_size; total; next;
        as_; in_; where_; is; not_;
        zip; url; member; size;
    }
}

pub(crate) const FAMILY: Family = Family { module: "connectors", roots: &["connector"], declare };

pub const PROVENANCE_FIELDS: &[&str] = &["title", "author", "licence", "source", "credit"];
pub const CONNECTOR_DECODES: &[&str] = &["json", "gzip-json"];

const CONNECTOR_ATTRS: &[&str] = &["name", "base", "rate", "cache", "home", "terms", "robots", "hosts", "title", "author", "licence", "source", "credit"];
const CREDENTIAL_ATTRS: &[&str] = &["service", "field", "query", "header", "prefix", "hosts", "optional"];
const FOLDER_ATTRS: &[&str] = &["path", "get", "decode", "page", "pageSize", "total", "next"];
const EACH_ATTRS: &[&str] = &["in", "as", "where", "is", "not"];
const DIR_ATTRS: &[&str] = &["name", "id", "zip", "title", "author", "licence", "source", "credit"];
const FILE_ATTRS: &[&str] = &["path", "url", "zip", "member", "size", "title", "author", "licence", "source", "credit"];

const CONNECTOR: Vocabulary = Vocabulary {
    format: "connector",
    noun: "a connector",
    root: "connector",
    tags: &[
        TagRule::new("connector", &[]).attrs(CONNECTOR_ATTRS),
        TagRule::new("credential", &["connector"]).single().attrs(CREDENTIAL_ATTRS),
        TagRule::new("licence", &["connector"]).key("value").required(&["value", "spdx"]).attrs(&["value", "spdx"]),
        TagRule::new("folder", &["connector"]).key("path").required(&["path"]).attrs(FOLDER_ATTRS),
        TagRule::new("request", &["folder"]).key("as").required(&["as", "get"]).attrs(&["as", "get", "decode"]),
        TagRule::new("here", &["folder"]).single().attrs(PROVENANCE_FIELDS),
        TagRule::new("each", &["folder", "each"]).required(&["in", "as"]).attrs(EACH_ATTRS),
        TagRule::new("dir", &["folder", "each"]).required(&["name"]).attrs(DIR_ATTRS),
        TagRule::new("file", &["folder", "each"]).required(&["path"]).attrs(FILE_ATTRS),
    ],
    from_tree: connector_of,
};

fn declare(root: El) -> Result<Declared, String> {
    declare_document(&root, &CONNECTOR)
}

/// JavaScript's `length` of a string: its UTF-16 code units.
fn js_len(s: &str) -> usize {
    s.encode_utf16().count()
}

fn is_text(v: Option<&Json>, max: usize) -> bool {
    matches!(v, Some(Json::Str(s)) if js_len(s) > 0 && js_len(s) <= max)
}

fn text_of<'a>(t: &'a DocTree, k: &str) -> &'a str {
    t.attr_of(k).and_then(Json::as_str).unwrap_or_default()
}

fn is_https(v: Option<&Json>) -> bool {
    let Some(Json::Str(s)) = v.filter(|v| is_text(Some(v), 2000)) else { return false };
    let Some(rest) = s.get(..8).filter(|p| p.eq_ignore_ascii_case("https://")).map(|_| &s[8..]) else { return false };
    let host = rest.split(['/', '?', '#']).next().unwrap_or_default();
    !host.is_empty() && !s.contains(char::is_whitespace)
}

fn is_host(s: &str) -> bool {
    let labels: Vec<&str> = s.split('.').collect();
    labels.len() >= 2
        && labels.iter().all(|l| {
            let b = l.as_bytes();
            !b.is_empty()
                && b.iter().all(|c| c.is_ascii_alphanumeric() || *c == b'-')
                && b[0] != b'-'
                && b[b.len() - 1] != b'-'
        })
}

fn is_hosts(v: &Json) -> bool {
    matches!(v, Json::Arr(items) if items.iter().all(|h| matches!(h, Json::Str(s) if is_host(s))))
}

fn is_name(s: &str) -> bool {
    let mut cs = s.chars();
    matches!(cs.next(), Some(c) if c.is_ascii_alphabetic() || c == '_') && cs.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// One path segment: a literal name (no `{`, `}` or `/`), or one `{var}`.
fn is_path_segment(s: &str) -> bool {
    match s.strip_prefix('{').and_then(|r| r.strip_suffix('}')) {
        Some(var) => is_name(var),
        None => !s.is_empty() && !s.contains(['{', '}', '/']),
    }
}

fn is_whole(v: Option<&Json>) -> Option<f64> {
    v.and_then(Json::as_f64).filter(|n| n.is_finite() && n.fract() == 0.0)
}

fn said(t: &DocTree, k: &str) -> String {
    t.attr_of(k).map(Json::text).unwrap_or_else(|| "undefined".into())
}

fn only(t: &DocTree, keys: &[&str]) -> Json {
    let mut out = Json::obj();
    for k in keys {
        if let Some(v) = t.attr_of(k) {
            out.set(k, v.clone());
        }
    }
    out
}

/// The provenance fields an element writes, each a text.
fn provenance(t: &DocTree, at: &str) -> Result<Vec<(String, Json)>, String> {
    let mut out = Vec::new();
    for k in PROVENANCE_FIELDS {
        let Some(v) = t.attr_of(k) else { continue };
        if !is_text(Some(v), 1000) {
            return Err(format!("{at} {k} is a text, not {}", v.text()));
        }
        out.push((k.to_string(), v.clone()));
    }
    Ok(out)
}

fn producer(t: &DocTree, at: &str) -> Result<Json, String> {
    match t.tag {
        "each" => {
            if !is_text(t.attr_of("in"), 500) {
                return Err(format!("{at} each() in is a pointer ($/x, a/y), not {}", said(t, "in")));
            }
            let var = t.attr_of("as");
            if !is_text(var, 64) || !is_name(text_of(t, "as")) {
                return Err(format!("{at} each() as is a name, not {}", said(t, "as")));
            }
            if t.attr_of("where").is_some() && !is_text(t.attr_of("where"), 500) {
                return Err(format!("{at} each() where is a pointer, not {}", said(t, "where")));
            }
            let (is, not) = (t.attr_of("is").is_some(), t.attr_of("not").is_some());
            if t.attr_of("where").is_some() != (is || not) || (is && not) {
                return Err(format!("{at} each() where goes with is or with not: one of them"));
            }
            for k in ["is", "not"] {
                if t.attr_of(k).is_some_and(|v| !matches!(v, Json::Str(_) | Json::Num(_) | Json::Bool(_))) {
                    return Err(format!("{at} each() {k} is a text, a number or true/false"));
                }
            }
            let inner = format!("{at} each().in({})", Json::Str(text_of(t, "in").to_string()).text());
            let produce = t.children.iter().map(|c| producer(c, &inner)).collect::<Result<Vec<_>, _>>()?;
            let mut out = Json::obj().with("kind", "each");
            for k in EACH_ATTRS {
                if let Some(v) = t.attr_of(k) {
                    out.set(k, v.clone());
                }
            }
            out.set("produce", Json::Arr(produce));
            Ok(out)
        }
        "dir" => {
            if !is_text(t.attr_of("name"), 500) {
                return Err(format!("{at} dir() name is a text (a template), not {}", said(t, "name")));
            }
            if t.attr_of("zip").is_some() && !is_text(t.attr_of("zip"), 2000) {
                return Err(format!("{at} dir() zip is a url (a template), not {}", said(t, "zip")));
            }
            if t.attr_of("id").is_some() && !is_text(t.attr_of("id"), 500) {
                return Err(format!("{at} dir() id is a text (a template), not {}", said(t, "id")));
            }
            let mut out = Json::obj().with("kind", "dir").with("name", t.attr_of("name").cloned().unwrap_or(Json::Null));
            if let Some(id) = t.attr_of("id") {
                out.set("id", id.clone());
            }
            if let Some(z) = t.attr_of("zip") {
                out.set("zip", z.clone());
            }
            let at_dir = format!("{at} dir().name({})", Json::Str(text_of(t, "name").to_string()).text());
            for (k, v) in provenance(t, &at_dir)? {
                out.set(&k, v);
            }
            Ok(out)
        }
        "file" => {
            let where_ = format!("{at} file().path({})", Json::Str(js_string(t.attr_of("path"))).text());
            if !is_text(t.attr_of("path"), 1000) {
                return Err(format!("{at} file() path is a text (a template), not {}", said(t, "path")));
            }
            for k in ["url", "zip", "member"] {
                if t.attr_of(k).is_some() && !is_text(t.attr_of(k), 2000) {
                    return Err(format!("{where_} {k} is a text (a template), not {}", said(t, k)));
                }
            }
            if t.attr_of("zip").is_some() != t.attr_of("member").is_some() {
                return Err(format!("{where_} zip and member go together"));
            }
            if t.attr_of("url").is_some() == t.attr_of("zip").is_some() {
                return Err(format!("{where_} reads its bytes from url, or from zip and member: one of them"));
            }
            if let Some(size) = t.attr_of("size") {
                let number = matches!(size, Json::Num(n) if *n >= 0.0);
                if !number && !is_text(Some(size), 500) {
                    return Err(format!("{where_} size is a number or a template"));
                }
            }
            let mut out = Json::obj().with("kind", "file");
            for k in ["path", "url", "zip", "member", "size"] {
                if let Some(v) = t.attr_of(k) {
                    out.set(k, v.clone());
                }
            }
            for (k, v) in provenance(t, &where_)? {
                out.set(&k, v);
            }
            Ok(out)
        }
        other => Err(format!("{at} {}() is not a level's entry (each(), dir(), file())", fn_name(other))),
    }
}

fn folder_of(t: &DocTree) -> Result<Json, String> {
    let path = match t.attr_of("path") {
        Some(Json::Str(p)) if js_len(p) <= 500 => p.as_str(),
        _ => return Err(format!(r#"folder() path is a text ("" is the root), not {}"#, said(t, "path"))),
    };
    let at = format!("folder().path({})", Json::Str(path.to_string()).text());
    if !path.is_empty() && !path.split('/').all(is_path_segment) {
        return Err(format!(r#"{at} path is names and {{var}} segments, separated by "/""#));
    }
    if t.attr_of("get").is_some() && !is_text(t.attr_of("get"), 2000) {
        return Err(format!("{at} get is a path or url (a template), not {}", said(t, "get")));
    }
    let decodes = CONNECTOR_DECODES.join(" or ");
    if t.attr_of("decode").is_some_and(|v| !matches!(v, Json::Str(s) if CONNECTOR_DECODES.contains(&s.as_str()))) {
        return Err(format!("{at} decode is {decodes}"));
    }
    if let Some(page) = t.attr_of("page") {
        let name = text_of(t, "page");
        if !is_text(Some(page), 64) || (name != "link" && !is_name(name)) {
            return Err(format!(r#"{at} page is "link" or a query parameter's name"#));
        }
        if name != "link" && t.attr_of("next").is_none() && !is_whole(t.attr_of("pageSize")).is_some_and(|n| n > 0.0) {
            return Err(format!("{at} pages by {name}: it needs pageSize (by offset) or next (by a token)"));
        }
    }
    if t.attr_of("total").is_some() && !is_text(t.attr_of("total"), 500) {
        return Err(format!("{at} total is a template, not {}", said(t, "total")));
    }
    if t.attr_of("next").is_some() && (!is_text(t.attr_of("next"), 500) || t.attr_of("page").is_none() || text_of(t, "page") == "link") {
        return Err(format!("{at} next is the template of the next page's token, with page naming its query parameter"));
    }
    if t.attr_of("get").is_none() && (t.attr_of("page").is_some() || t.attr_of("decode").is_some()) {
        return Err(format!("{at} pages and decodes a request: it needs get"));
    }
    let mut requests = Vec::new();
    let mut here = None;
    let mut produce = Vec::new();
    for c in &t.children {
        match c.tag {
            "request" => {
                if !is_text(c.attr_of("as"), 64) || !is_name(text_of(c, "as")) {
                    return Err(format!("{at} request() as is a name, not {}", said(c, "as")));
                }
                if !is_text(c.attr_of("get"), 2000) {
                    return Err(format!("{at} request().as({}) get is a path or url", Json::Str(text_of(c, "as").to_string()).text()));
                }
                if c.attr_of("decode").is_some_and(|v| !matches!(v, Json::Str(s) if CONNECTOR_DECODES.contains(&s.as_str()))) {
                    return Err(format!("{at} request() decode is {decodes}"));
                }
                requests.push(only(c, &["as", "get", "decode"]));
            }
            "here" => {
                let mut h = Json::obj();
                for (k, v) in provenance(c, &format!("{at} here()"))? {
                    h.set(&k, v);
                }
                here = Some(h);
            }
            _ => produce.push(producer(c, &at)?),
        }
    }
    let mut out = only(t, FOLDER_ATTRS);
    if !requests.is_empty() {
        out.set("requests", Json::Arr(requests));
    }
    if let Some(h) = here {
        out.set("here", h);
    }
    out.set("produce", Json::Arr(produce));
    Ok(out)
}

/// A connector as its elements declare it, or why it does not read (the TypeScript SDK's `connectorOf`).
pub fn connector_of(t: &DocTree) -> Result<Json, String> {
    if !is_text(t.attr_of("name"), 200) {
        return Err(format!("connector() name is the source's name, not {}", said(t, "name")));
    }
    let at = format!("connector().name({})", Json::Str(text_of(t, "name").to_string()).text());
    if !is_https(t.attr_of("base")) {
        return Err(format!("{at} base is the API's https url, not {}", said(t, "base")));
    }
    if !t.attr_of("rate").and_then(Json::as_f64).is_some_and(|n| n > 0.0 && n <= 10.0) {
        return Err(format!("{at} rate is the requests a second it makes at most (above 0, at most 10), not {}", said(t, "rate")));
    }
    if !is_whole(t.attr_of("cache")).is_some_and(|n| n >= 60.0) {
        return Err(format!("{at} cache is the seconds a listing is kept (a whole number, at least 60), not {}", said(t, "cache")));
    }
    for k in ["home", "terms", "robots"] {
        if t.attr_of(k).is_some() && !is_https(t.attr_of(k)) {
            return Err(format!("{at} {k} is an https url, not {}", said(t, k)));
        }
    }
    if t.attr_of("hosts").is_some_and(|v| !is_hosts(v)) {
        return Err(format!("{at} hosts is a list of host names"));
    }
    let mut doc = only(t, CONNECTOR_ATTRS);
    for (k, v) in provenance(t, &at)? {
        doc.set(&k, v);
    }
    let mut credential = None;
    let mut licences = Vec::new();
    let mut folders = Vec::new();
    let mut paths: Vec<String> = Vec::new();
    for c in &t.children {
        match c.tag {
            "credential" => {
                if credential.is_some() {
                    return Err(format!("{at} has one credential()"));
                }
                let service = text_of(c, "service");
                if !is_text(c.attr_of("service"), 64)
                    || !service.starts_with(|ch: char| ch.is_ascii_lowercase() || ch.is_ascii_digit())
                    || !service.chars().all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-')
                {
                    return Err(format!("{at} credential() service is a service's name (docs/services.md), not {}", said(c, "service")));
                }
                let field = text_of(c, "field");
                if !is_text(c.attr_of("field"), 64) || !field.chars().all(|ch| ch.is_ascii_uppercase() || ch.is_ascii_digit() || ch == '_') {
                    return Err(format!("{at} credential() field is a field's name (API_KEY), not {}", said(c, "field")));
                }
                if c.attr_of("query").is_some() == c.attr_of("header").is_some() {
                    return Err(format!("{at} credential() goes as a query parameter or a header: one of them"));
                }
                for k in ["query", "header", "prefix"] {
                    if c.attr_of(k).is_some() && !is_text(c.attr_of(k), 64) {
                        return Err(format!("{at} credential() {k} is a text"));
                    }
                }
                if c.attr_of("hosts").is_some_and(|v| !is_hosts(v)) {
                    return Err(format!("{at} credential() hosts is a list of host names"));
                }
                if c.attr_of("optional").is_some_and(|v| !matches!(v, Json::Bool(_))) {
                    return Err(format!("{at} credential() optional is true or false"));
                }
                credential = Some(only(c, CREDENTIAL_ATTRS));
            }
            "licence" => {
                if !is_text(c.attr_of("value"), 200) || !is_text(c.attr_of("spdx"), 200) {
                    return Err(format!("{at} licence() maps a value the API writes to an spdx id: both are texts"));
                }
                licences.push(only(c, &["value", "spdx"]));
            }
            "folder" => {
                let f = folder_of(c)?;
                let path = f.get("path").and_then(Json::as_str).unwrap_or_default().to_string();
                if paths.contains(&path) {
                    return Err(format!(r#"{at} has two folder().path("{path}")"#));
                }
                paths.push(path);
                folders.push(f);
            }
            other => return Err(format!("{at} {}() is not read (credential(), licence(), folder())", fn_name(other))),
        }
    }
    if let Some(c) = credential {
        doc.set("credential", c);
    }
    if !licences.is_empty() {
        doc.set("licences", Json::Arr(licences));
    }
    if folders.is_empty() {
        return Err(format!("{at} declares no folder(): a connector lists at least its root"));
    }
    doc.set("folders", Json::Arr(folders));
    Ok(doc)
}
