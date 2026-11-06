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

use super::company_formation::FORMATION_TAGS;
pub use super::company_formation::{registered_agent, FormationAttrs};
use super::company_governance::{governance_of, GOVERNANCE_TAGS};
use super::company_deals::{deals_of, DEALS_TAGS};
use super::element::{attributes, elements, fn_name, rust_name};
use super::business_finance::{finance_of, COMPANY_FINANCE_MANY, COMPANY_FINANCE_ONE, FINANCE_TAGS};
use super::people_ops::{team_of, PEOPLE_OPS_TAGS};
use super::facts::fact_marker;
use super::{Declared, Family};
pub use super::business_finance::{bank_account, budget, data_room, include, payroll, FinanceAttrs};
pub use super::people_ops::{band, benefit, checklist, role, team, PeopleOpsAttrs};
pub use super::company_governance::{assignment, compliance, evidence, ip_asset, obligation, policy, seat, vote, GovernanceAttrs};
pub use super::company_deals::{deal, DealAttrs};
pub use super::{fragment, json, El, Json, Value};

/// A priced round, pro forma, and its investments (`round()`, `investment()`).
mod equity;
use equity::{rounds_of, EQUITY_TAGS};
pub use equity::{investment, round, EquityAttrs};

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
    /// A group's subsidiary: its own company file, `.file("Acme UK.company.rs").ownership("100")`.
    subsidiary = "Subsidiary": leaf;
    /// A typed document of the company: `.kind("nda").file("Acme/Documents/NDA.md")`, its signatures and versions.
    paper = "Paper": children;
    /// A signature on a paper: `.party("Acme Inc.").signer("Ada Park").evidence("…signed.pdf")`.
    signature = "Signature": leaf;
    /// An earlier file of a paper: `.file("…v1.md").date("2024-01-08")`.
    version = "Version": leaf;
    /// A brand of the company: its own `.brand.rs`, `.file("brands/Northwind.brand.rs")` (`super::brands`).
    brand = "Brand": leaf;
    /// What the company sells or makes: `.name("…").maker("Exocortex Labs").seller("CommandAGI Inc.")` (the merchant of record).
    product = "Product": leaf;
    /// The company's relationships (suppliers, manufacturers, funders, channels): `.file("relationships/relationships.csv")`.
    relationships = "Relationships": leaf;
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
        name; about; postal_address; files; dashboard;
        jurisdiction; form; tax_classification; formed; fiscal_year_end; ids; operates_in; conditions;
        kind; id; issued; expires; file; journal; ocf; folder;
        procedure; title; body; target; engine_json; summary;
        op; parameter; value; duty_id; text; protects; harm; elements; criminal; uniqueness; issuers; modalities;
        respondent; filed_for; source; article; agreement_id; term; act; act_date; evidence; media; reopens;
        interest; description; amount; harm_ids; days;
        ownership; since; method; status; parties; effective; renewal; notice_days;
        party; signer; capacity; date; note;
        maker; seller;
    }
}

pub(crate) const FAMILY: Family = Family { module: "business", roots: &["Company", "Rfc", "Case"], declare };

#[derive(Clone, Copy, PartialEq)]
pub(super) enum Kind {
    Text,
    Texts,
    Bool,
    Record,
    Data,
    Amount,
}
pub(crate) use Kind::*;

/// One tag: its attributes and their kinds, those it needs, and its children by the field they fill.
pub(super) struct TagSpec {
    pub(super) tag: &'static str,
    pub(super) props: &'static [(&'static str, Kind)],
    pub(super) required: &'static [&'static str],
    /// A child that occurs at most once, by the field it fills.
    pub(super) one: &'static [(&'static str, &'static str)],
    /// Children that repeat, by the list they fill.
    pub(super) many: &'static [(&'static str, &'static str)],
}

pub(crate) const fn spec(tag: &'static str, props: &'static [(&'static str, Kind)], required: &'static [&'static str]) -> TagSpec {
    TagSpec { tag, props, required, one: &[], many: &[] }
}

/// The vocabulary: each tag's attributes and children (what the document's fields are).
const BUSINESS_TAGS: &[TagSpec] = &[
    TagSpec {
        tag: "Company",
        props: &[("name", Text), ("about", Text), ("postalAddress", Text), ("files", Texts), ("dashboard", Text)],
        required: &["name"],
        one: &[("entity", "Entity"), ("books", "Books"), ("captable", "CapTable"), ("people", "People"), ("calendar", "Calendar"), ("matters", "Matters"), ("team", "Team"), ("compliance", "Compliance"), ("relationships", "Relationships")],
        many: &[("registrations", "Registration"), ("subsidiaries", "Subsidiary"), ("papers", "Paper"), ("rounds", "Round"), ("registeredAgents", "RegisteredAgent"), ("brands", "Brand"),
            ("seats", "Seat"), ("obligations", "Obligation"), ("evidence", "Evidence"), ("ipAssets", "IpAsset"), ("assignments", "Assignment"), ("policies", "Policy"), ("products", "Product"), ("deals", "Deal")],
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
            ("dissolved", Text),
        ],
        &["jurisdiction"],
    ),
    spec("Registration", &[("kind", Text), ("jurisdiction", Text), ("id", Text), ("issued", Text), ("expires", Text), ("file", Text)], &["kind"]),
    spec("Books", &[("journal", Text)], &["journal"]),
    spec("CapTable", &[("ocf", Text)], &["ocf"]),
    spec("People", &[("folder", Text)], &["folder"]),
    spec("Calendar", &[("folder", Text)], &["folder"]),
    spec("Matters", &[("folder", Text)], &["folder"]),
    spec("Subsidiary", &[("file", Text), ("ownership", Amount), ("since", Text), ("method", Text)], &["file", "ownership"]),
    TagSpec {
        tag: "Paper",
        props: &[
            ("kind", Text),
            ("title", Text),
            ("file", Text),
            ("status", Text),
            ("parties", Texts),
            ("jurisdiction", Text),
            ("effective", Text),
            ("expires", Text),
            ("renewal", Text),
            ("noticeDays", Amount),
            // A filing's (company_formation.rs).
            ("rule", Text),
            ("period", Text),
            ("due", Text),
            ("submitted", Text),
            ("accepted", Text),
            ("confirmation", Text),
            // a vote on the paper (company_governance.rs)
            ("body", Text),
            ("threshold", Text),
            ("quorum", Text),
        ],
        required: &["kind", "file"],
        one: &[],
        many: &[("signatures", "Signature"), ("versions", "Version"), ("votes", "Vote")],
    },
    spec("Signature", &[("party", Text), ("signer", Text), ("capacity", Text), ("date", Text), ("method", Text), ("evidence", Text)], &["party"]),
    spec("Version", &[("file", Text), ("date", Text), ("note", Text)], &["file"]),
    spec("Brand", &[("file", Text)], &["file"]),
    spec("Product", &[("id", Text), ("name", Text), ("about", Text), ("maker", Text), ("seller", Text), ("status", Text), ("file", Text)], &["name"]),
    spec("Relationships", &[("file", Text)], &["file"]),
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
    pub(super) fn prop(&self, name: &str) -> Option<&Json> {
        self.props.iter().find(|(k, _)| k == name).map(|(_, v)| v)
    }
    pub(super) fn one(&self, field: &str) -> Option<&BusinessNode> {
        self.one.iter().find(|(f, _)| *f == field).and_then(|(_, n)| n.as_ref())
    }
    pub(super) fn many(&self, field: &str) -> &[BusinessNode] {
        self.many.iter().find(|(f, _)| *f == field).map(|(_, l)| l.as_slice()).unwrap_or(&[])
    }
}

/// An attribute's value, checked against its kind; or an unknown or approximate fact of one (`super::facts`).
fn value(tag: &str, prop: &str, kind: Kind, v: &Json) -> Result<Json, String> {
    if let Some(fact) = fact_marker(v, &format!("{}(): {}", fn_name(tag), rust_name(prop)), |x| value(tag, prop, kind, x))? {
        return Ok(fact);
    }
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
    let Some(spec) = BUSINESS_TAGS.iter().chain(FINANCE_TAGS).chain(FORMATION_TAGS).chain(EQUITY_TAGS).chain(PEOPLE_OPS_TAGS).chain(GOVERNANCE_TAGS).chain(DEALS_TAGS).find(|s| s.tag == el.tag) else {
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
    // A company also takes its finance's children (business_finance.rs).
    let (finance_one, finance_many): (&[(&str, &str)], &[(&str, &str)]) = if spec.tag == "Company" { (COMPANY_FINANCE_ONE, COMPANY_FINANCE_MANY) } else { (&[], &[]) };
    let spec_one: Vec<(&'static str, &'static str)> = spec.one.iter().chain(finance_one).copied().collect();
    let spec_many: Vec<(&'static str, &'static str)> = spec.many.iter().chain(finance_many).copied().collect();
    let mut one: Vec<(&'static str, Option<BusinessNode>)> = spec_one.iter().map(|(field, _)| (*field, None)).collect();
    let mut many: Vec<(&'static str, Vec<BusinessNode>)> = spec_many.iter().map(|(field, _)| (*field, Vec::new())).collect();
    for child in el.child_elements() {
        if let Some(i) = spec_one.iter().position(|(_, t)| *t == child.tag) {
            let node = read_business_node(child)?;
            if one[i].1.is_some() {
                return Err(format!("{f}() has two {}()", fn_name(child.tag)));
            }
            one[i].1 = Some(node);
        } else if let Some(i) = spec_many.iter().position(|(_, t)| *t == child.tag) {
            many[i].1.push(read_business_node(child)?);
        } else {
            let takes: Vec<String> = spec_one.iter().chain(&spec_many).map(|(_, t)| format!("{}()", fn_name(t))).collect();
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
            if let Some(d) = e.prop("dissolved") {
                out.set("dissolved", d.clone());
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
    let company = Json::obj()
        .with("format", "commandagi-company")
        .with("name", or_null(node.prop("name")))
        .with("about", node.prop("about").cloned().unwrap_or_else(|| "".into()));
    let company = match node.prop("postalAddress") {
        Some(p) => company.with("postalAddress", p.clone()),
        None => company,
    };
    let company = company
        .with("files", node.prop("files").cloned().unwrap_or(Json::Arr(Vec::new())))
        .with("dashboard", or_null(node.prop("dashboard")));
    let company = company.with("entity", entity).with("standard", standard);
    // Each area's field is absent when the file names none of it (docs/company-substrate.md § how an area joins).
    let subsidiaries: Vec<Json> = node.many("subsidiaries").iter().map(|x| Json::Obj(x.props.clone())).collect();
    let papers: Vec<Json> = node
        .many("papers")
        .iter()
        .map(|x| {
            let mut paper = Json::Obj(x.props.clone());
            for field in ["signatures", "versions", "votes"] {
                let list: Vec<Json> = x.many(field).iter().map(|n| Json::Obj(n.props.clone())).collect();
                if !list.is_empty() {
                    paper.set(field, list);
                }
            }
            paper
        })
        .collect();
    let company = if subsidiaries.is_empty() { company } else { company.with("subsidiaries", subsidiaries) };
    let company = if papers.is_empty() { company } else { company.with("papers", papers) };
    let rounds = rounds_of(node);
    let company = if rounds.is_empty() { company } else { company.with("rounds", rounds) };
    let company = match finance_of(node) {
        Some(finance) => company.with("finance", finance),
        None => company,
    };
    let agents: Vec<Json> = node.many("registeredAgents").iter().map(|x| Json::Obj(x.props.clone())).collect();
    let company = if agents.is_empty() { company } else { company.with("registeredAgents", agents) };
    let company = match node.one("team") {
        Some(t) => company.with("team", team_of(t)),
        None => company,
    };
    let brands: Vec<Json> = node.many("brands").iter().map(|x| Json::Obj(x.props.clone())).collect();
    let company = if brands.is_empty() { company } else { company.with("brands", brands) };
    let products: Vec<Json> = node.many("products").iter().map(|x| Json::Obj(x.props.clone())).collect();
    let company = if products.is_empty() { company } else { company.with("products", products) };
    let company = match node.one("relationships").and_then(|n| n.prop("file")) {
        Some(file) => company.with("relationships", file.clone()),
        None => company,
    };
    let company = governance_of(node, company);
    deals_of(node, company)
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
