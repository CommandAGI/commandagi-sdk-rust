//! A COMPANY, AN RFC OR A CASE IN RUST — the documents the CommandAGI company app and contract editor open, declared
//! as code (`<Name>.company.rs`, `<name>.rfc.rs`, `<name>.case.rs`). The same elements, and the same documents, as the
//! TypeScript SDK's `business.ts` and the Python SDK's `business.py`:
//!
//! ```
//! use commandagi::design::business::*;
//!
//! fn document() -> El {
//!     company([
//!         entity().jurisdiction("US-DE").form("llc").formed("2024-01-31").fiscal_year_end("12-31"),
//!         books().journal("Northwind/books/main.journal"),
//!         cap_table().ocf("Northwind/captable/"),
//!         registration().kind("tax-id").jurisdiction("US").id("12-3456789"),
//!     ])
//!     .name("Northwind")
//!     .files(["Northwind/"])
//! }
//! # commandagi::design::declare(document()).unwrap();
//! ```
//!
//! The reader gives back the document the apps use: the company itself, or an RFC's or a case's `{"id"?, "draft"}`.
//! The standard's parts REF their files (the journal stays hledger, the cap table Open Cap Table Format); the company
//! is on the standard when it names any of them. A change's `parameter` is the draft's `key`. Beside the document,
//! `sources` names the call each part was written in, by path (`""` the root, `entity`, `options/0/changes/1`). An
//! unknown attribute or child is refused by name.
//!
//! ```text
//! rfc([option([change().op …, …]).title.summary, …]).title.target.procedure.body.engine_json.id?
//! case([harm().id.interest.description.amount, relief().kind.description.harm_ids.amount.days, …])
//!     .respondent.filed_for.criminal.source.duty_id.article … .id?
//! ```

use super::element::{attributes, elements, fn_name, rust_name};
use super::{Declared, Family};
pub use super::{fragment, json, El, Json, Value};

elements! {
    /// A company: its entity, the standard's parts and its registrations.
    company = "Company": children;
    /// The legal entity: `.jurisdiction("US-DE").form("llc")`.
    entity = "Entity": leaf;
    /// A registration (a tax id, a licence): `.kind("tax-id").jurisdiction("US").id("…")`.
    registration = "Registration": leaf;
    /// The books: an hledger journal, by its path.
    books = "Books": leaf;
    /// The cap table: an Open Cap Table Format folder, by its path.
    cap_table = "CapTable": leaf;
    /// The people folder.
    people = "People": leaf;
    /// The calendar folder.
    calendar = "Calendar": leaf;
    /// The matters folder.
    matters = "Matters": leaf;
    /// An RFC draft: its options.
    rfc = "Rfc": children;
    /// One option of an RFC: its changes.
    option = "Option": children;
    /// One change of the law: `.op("add_duty").duty_id("…")`; `.parameter(…)` is the draft's `key`.
    change = "Change": leaf;
    /// A case draft: its harms and relief.
    case = "Case": children;
    /// A harm of a case: `.id("h1").interest("property").amount("4200")`.
    harm = "Harm": leaf;
    /// A relief a case asks: `.kind("restitution").harm_ids(["h1"])`.
    relief = "Relief": leaf;
}

attributes! {
    /// The fields of a company, an RFC and a case, chained on their elements.
    pub trait BusinessAttrs {
        name; about; files; dashboard;
        jurisdiction; form; tax_classification; formed; fiscal_year_end; ids; operates_in; conditions;
        kind; id; issued; expires; file; journal; ocf; folder;
        procedure; title; body; target; engine_json; summary;
        op; parameter; value; duty_id; text; protects; harm; elements; criminal; uniqueness; issuers; modalities;
        respondent; filed_for; source; article; agreement_id; term; act; act_date; evidence; media; reopens;
        interest; description; amount; harm_ids; days;
    }
}

pub(crate) const FAMILY: Family = Family { module: "business", roots: &["Company", "Rfc", "Case"], declare };

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    Text,
    Texts,
    Bool,
    Record,
    Data,
    Amount,
}
use Kind::*;

/// One tag: its attributes and their kinds, those it needs, and its children by the field they fill.
struct TagSpec {
    tag: &'static str,
    props: &'static [(&'static str, Kind)],
    required: &'static [&'static str],
    /// A child that occurs at most once, by the field it fills.
    one: &'static [(&'static str, &'static str)],
    /// Children that repeat, by the list they fill.
    many: &'static [(&'static str, &'static str)],
}

const fn spec(tag: &'static str, props: &'static [(&'static str, Kind)], required: &'static [&'static str]) -> TagSpec {
    TagSpec { tag, props, required, one: &[], many: &[] }
}

/// The vocabulary: each tag's attributes and children (what the document's fields are).
const BUSINESS_TAGS: &[TagSpec] = &[
    TagSpec {
        tag: "Company",
        props: &[("name", Text), ("about", Text), ("files", Texts), ("dashboard", Text)],
        required: &["name"],
        one: &[("entity", "Entity"), ("books", "Books"), ("captable", "CapTable"), ("people", "People"), ("calendar", "Calendar"), ("matters", "Matters")],
        many: &[("registrations", "Registration")],
    },
    spec(
        "Entity",
        &[
            ("jurisdiction", Text),
            ("form", Text),
            ("taxClassification", Text),
            ("name", Text),
            ("formed", Text),
            ("fiscalYearEnd", Text),
            ("ids", Record),
            ("operatesIn", Texts),
            ("conditions", Record),
        ],
        &["jurisdiction"],
    ),
    spec("Registration", &[("kind", Text), ("jurisdiction", Text), ("id", Text), ("issued", Text), ("expires", Text), ("file", Text)], &["kind"]),
    spec("Books", &[("journal", Text)], &["journal"]),
    spec("CapTable", &[("ocf", Text)], &["ocf"]),
    spec("People", &[("folder", Text)], &["folder"]),
    spec("Calendar", &[("folder", Text)], &["folder"]),
    spec("Matters", &[("folder", Text)], &["folder"]),
    TagSpec {
        tag: "Rfc",
        props: &[("id", Text), ("procedure", Text), ("title", Text), ("body", Text), ("target", Text), ("engineJson", Text)],
        required: &[],
        one: &[],
        many: &[("options", "Option")],
    },
    TagSpec { tag: "Option", props: &[("title", Text), ("summary", Text)], required: &[], one: &[], many: &[("changes", "Change")] },
    spec(
        "Change",
        &[
            ("op", Text),
            ("parameter", Text),
            ("value", Text),
            ("dutyId", Text),
            ("title", Text),
            ("text", Text),
            ("protects", Texts),
            ("harm", Text),
            ("elements", Text),
            ("criminal", Bool),
            ("kind", Text),
            ("uniqueness", Text),
            ("issuers", Text),
            ("modalities", Texts),
        ],
        &["op"],
    ),
    TagSpec {
        tag: "Case",
        props: &[
            ("id", Text),
            ("respondent", Text),
            ("filedFor", Text),
            ("criminal", Bool),
            ("source", Text),
            ("dutyId", Text),
            ("article", Text),
            ("agreementId", Text),
            ("term", Text),
            ("act", Text),
            ("actDate", Text),
            ("evidence", Text),
            ("media", Data),
            ("reopens", Text),
        ],
        required: &[],
        one: &[],
        many: &[("harms", "Harm"), ("relief", "Relief")],
    },
    spec("Harm", &[("id", Text), ("interest", Text), ("description", Text), ("amount", Amount)], &["id"]),
    spec("Relief", &[("kind", Text), ("description", Text), ("harmIds", Texts), ("amount", Amount), ("days", Amount)], &["kind"]),
];

/// One element as read: its attributes, its children by field, and its site.
pub struct BusinessNode {
    pub tag: &'static str,
    pub props: Vec<(String, Json)>,
    pub one: Vec<(&'static str, Option<BusinessNode>)>,
    pub many: Vec<(&'static str, Vec<BusinessNode>)>,
    pub source: Option<Json>,
}

impl BusinessNode {
    fn prop(&self, name: &str) -> Option<&Json> {
        self.props.iter().find(|(k, _)| k == name).map(|(_, v)| v)
    }
    fn one(&self, field: &str) -> Option<&BusinessNode> {
        self.one.iter().find(|(f, _)| *f == field).and_then(|(_, n)| n.as_ref())
    }
    fn many(&self, field: &str) -> &[BusinessNode] {
        self.many.iter().find(|(f, _)| *f == field).map(|(_, l)| l.as_slice()).unwrap_or(&[])
    }
}

/// An attribute's value, checked against its kind.
fn value(tag: &str, prop: &str, kind: Kind, v: &Json) -> Result<Json, String> {
    let bad = |what: &str| format!("{}(): {} is {what}, not {}", fn_name(tag), rust_name(prop), v.text());
    let ok = match (kind, v) {
        (Text, Json::Str(_)) | (Bool, Json::Bool(_)) | (Record, Json::Obj(_)) | (Data, _) => true,
        // An amount is as the form typed it (text) or as a file stored it (a number); each is kept as it is.
        (Amount, Json::Str(_)) => true,
        (Amount, Json::Num(n)) => n.is_finite(),
        (Texts, Json::Arr(items)) => items.iter().all(|x| matches!(x, Json::Str(_))),
        _ => false,
    };
    if ok {
        return Ok(v.clone());
    }
    Err(bad(match kind {
        Text => "text",
        Amount => "text or a number",
        Texts => "a list of text",
        Bool => "true or false",
        Record => "an object",
        Data => unreachable!("any JSON is data"),
    }))
}

/// Read one element of the vocabulary and its children, refusing what it does not say.
pub fn read_business_node(el: &El) -> Result<BusinessNode, String> {
    let Some(spec) = BUSINESS_TAGS.iter().find(|s| s.tag == el.tag) else {
        return Err(format!("{}() is not an element of a company, an RFC or a case", fn_name(el.tag)));
    };
    let f = fn_name(el.tag);
    let mut props = Vec::new();
    for (k, v) in &el.attrs {
        let Some((_, kind)) = spec.props.iter().find(|(p, _)| p == k) else {
            let takes: Vec<String> = spec.props.iter().map(|(p, _)| rust_name(p)).collect();
            return Err(format!("{f}() has no attribute {} (it takes {})", rust_name(k), takes.join(", ")));
        };
        props.push((k.clone(), value(el.tag, k, *kind, v)?));
    }
    for k in spec.required {
        let given = props.iter().find(|(p, _)| p == k).map(|(_, v)| v);
        if given.is_none() || given.and_then(Json::as_str) == Some("") {
            return Err(format!("{f}() needs {}", rust_name(k)));
        }
    }
    let mut one: Vec<(&'static str, Option<BusinessNode>)> = spec.one.iter().map(|(field, _)| (*field, None)).collect();
    let mut many: Vec<(&'static str, Vec<BusinessNode>)> = spec.many.iter().map(|(field, _)| (*field, Vec::new())).collect();
    for child in el.child_elements() {
        if let Some(i) = spec.one.iter().position(|(_, t)| *t == child.tag) {
            let node = read_business_node(child)?;
            if one[i].1.is_some() {
                return Err(format!("{f}() has two {}()", fn_name(child.tag)));
            }
            one[i].1 = Some(node);
        } else if let Some(i) = spec.many.iter().position(|(_, t)| *t == child.tag) {
            many[i].1.push(read_business_node(child)?);
        } else {
            let takes: Vec<String> = spec.one.iter().chain(spec.many).map(|(_, t)| format!("{}()", fn_name(t))).collect();
            let takes = if takes.is_empty() { "no children".to_string() } else { takes.join(", ") };
            return Err(format!("{f}() does not take {}() (it takes {takes})", fn_name(child.tag)));
        }
    }
    Ok(BusinessNode { tag: el.tag, props, one, many, source: el.source_json() })
}

/// Every element's site, by path (`""` the root, `entity`, `options/0/changes/1`).
fn sources_of(node: &BusinessNode, at: &str, out: &mut Vec<(String, Json)>) {
    if let Some(s) = &node.source {
        out.push((at.to_string(), s.clone()));
    }
    let join = |f: &str| if at.is_empty() { f.to_string() } else { format!("{at}/{f}") };
    for (field, n) in &node.one {
        if let Some(n) = n {
            sources_of(n, &join(field), out);
        }
    }
    for (field, list) in &node.many {
        for (i, n) in list.iter().enumerate() {
            sources_of(n, &join(&format!("{field}/{i}")), out);
        }
    }
}

fn or_null(v: Option<&Json>) -> Json {
    v.cloned().unwrap_or(Json::Null)
}

/// A company document from a `company()` (docs/formats.md § companies).
pub fn company_of(node: &BusinessNode) -> Json {
    let entity = match node.one("entity") {
        None => Json::Null,
        Some(e) => {
            let mut out = Json::obj().with("jurisdiction", or_null(e.prop("jurisdiction"))).with("form", or_null(e.prop("form")));
            if let Some(t) = e.prop("taxClassification") {
                out.set("taxClassification", t.clone());
            }
            out = out
                .with("name", or_null(e.prop("name")))
                .with("formed", or_null(e.prop("formed")))
                .with("fiscalYearEnd", or_null(e.prop("fiscalYearEnd")))
                .with("ids", e.prop("ids").cloned().unwrap_or_else(Json::obj))
                .with("operatesIn", e.prop("operatesIn").cloned().unwrap_or(Json::Arr(Vec::new())));
            if let Some(c) = e.prop("conditions") {
                out.set("conditions", c.clone());
            }
            out
        }
    };
    let reference = |field: &str, prop: &str| or_null(node.one(field).and_then(|n| n.prop(prop)));
    let registrations: Vec<Json> = node.many("registrations").iter().map(|r| Json::Obj(r.props.clone())).collect();
    let on = ["books", "captable", "people", "calendar", "matters"].iter().any(|f| node.one(f).is_some()) || !registrations.is_empty();
    let standard = if on {
        Json::obj()
            .with("enabled", true)
            .with("books", reference("books", "journal"))
            .with("captable", reference("captable", "ocf"))
            .with("people", reference("people", "folder"))
            .with("calendar", reference("calendar", "folder"))
            .with("registrations", registrations)
            .with("matters", reference("matters", "folder"))
    } else {
        Json::obj().with("enabled", false)
    };
    Json::obj()
        .with("format", "commandagi-company")
        .with("name", or_null(node.prop("name")))
        .with("about", node.prop("about").cloned().unwrap_or_else(|| "".into()))
        .with("files", node.prop("files").cloned().unwrap_or(Json::Arr(Vec::new())))
        .with("dashboard", or_null(node.prop("dashboard")))
        .with("entity", entity)
        .with("standard", standard)
}

/// `{"id"?, "draft"}`: the id once the contract opened the draft (or it was filed).
fn with_id(node: &BusinessNode, draft: Json) -> Json {
    let out = match node.prop("id") {
        Some(id) => Json::obj().with("id", id.clone()),
        None => Json::obj(),
    };
    out.with("draft", draft)
}

/// The props but `id`, as a draft's fields.
fn draft_of(node: &BusinessNode) -> Json {
    Json::Obj(node.props.iter().filter(|(k, _)| k != "id").cloned().collect())
}

/// An RFC from an `rfc()`: `{"draft"}`, and `id` once the contract opened it. Fields not written take the blank's.
pub fn rfc_of(node: &BusinessNode) -> Json {
    // A change's `parameter` attribute is the draft's `key` (JSX keeps `key` for itself).
    let change = |c: &BusinessNode| {
        let mut out = Json::Obj(c.props.iter().filter(|(k, _)| k != "parameter").cloned().collect());
        if let Some(p) = c.prop("parameter") {
            out.set("key", p.clone());
        }
        out
    };
    let options: Vec<Json> = node
        .many("options")
        .iter()
        .map(|o| Json::Obj(o.props.clone()).with("changes", o.many("changes").iter().map(change).collect::<Vec<_>>()))
        .collect();
    with_id(node, draft_of(node).with("options", options))
}

/// A case from a `case()`: `{"draft"}`, and `id` once it was filed. Fields not written take the blank's.
pub fn case_of(node: &BusinessNode) -> Json {
    let records = |field: &str| node.many(field).iter().map(|n| Json::Obj(n.props.clone())).collect::<Vec<_>>();
    with_id(node, draft_of(node).with("harms", records("harms")).with("relief", records("relief")))
}

/// Declare a company, an RFC or a case: its document, and each element's site by path.
fn declare(root: El) -> Result<Declared, String> {
    let node = read_business_node(&root)?;
    let (format, document) = match root.tag {
        "Company" => ("company", company_of(&node)),
        "Rfc" => ("rfc", rfc_of(&node)),
        _ => ("case", case_of(&node)),
    };
    let mut sources = Vec::new();
    sources_of(&node, "", &mut sources);
    Ok(Declared::Document { format: format.into(), document, sources: Json::Obj(sources) })
}
