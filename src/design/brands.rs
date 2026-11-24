//! A BRAND IN RUST — a `<Name>.brand.rs`, one brand of a company: the names it goes by, its tagline and voice, its
//! logos, the design system it is drawn with, its domains, its sites, its social handles and the marks that protect
//! it. The same elements, and the same document, as the TypeScript SDK's `brands.ts` and the Python SDK's `brands.py`:
//!
//! ```
//! use commandagi::design::brands::*;
//!
//! fn document() -> El {
//!     brand_surface([
//!         alias().name("Northwind Survey").kind("legal"),
//!         logo().file("../design/logo.svg").kind("mark").background("light"),
//!         domain().name("northwind.example").registrar("Porkbun LLC").expires("2027-05-18").auto_renew(true)
//!             .transfer_lock(commandagi::design::facts::about("2026-12-30")),
//!         site().folder("../site/"),
//!         site().host("cloudflare-pages").project("northwind-www").url("https://www.northwind.example"),
//!         mailbox().address("support@northwind.example").kind("support"),
//!         handle().platform("x").handle("northwindsurvey"),
//!         mark().title("NORTHWIND").jurisdiction("US").number("99123456"),
//!     ])
//!     .name("Northwind")
//!     .voice(["plain", "warm"])
//!     .design_system("../design/brand-book.json")
//! }
//! # commandagi::design::declare(document()).unwrap();
//! ```
//!
//! The rule of the ontology's files: a record is an element and its fields are the element's attributes. Every path
//! is a ref from the brand file's folder: the brand holds no logo, design system or site of its own, and a `mark()`
//! refs a trademark the company records as an IP asset. Nothing adds a default. A company names each brand file with
//! `business::brand().file(…)`. A run gives back `{format, document, sources}` beside an empty graph; `sources` names
//! each element's site by its path (`""`, `Domain#northwind.example`, `Handle@0`).

use super::documents::{declare_document, DocTree, TagRule, Vocabulary};
use super::element::{attributes, elements, rust_name};
use super::facts::fact_marker;
use super::{Declared, Family};
pub use super::{fragment, json, El, Json, Value};

elements! {
    /// A brand: its names, logos, domains, sites, handles and marks.
    brand_surface = "BrandSurface": children;
    /// Another name of the brand: `.name("Northwind Survey").kind("legal")`.
    alias = "Alias": leaf;
    /// A logo: `.file("logo.svg").kind("mark").background("light")`, a ref from the brand file's folder.
    logo = "Logo": leaf;
    /// A domain: `.name("northwind.example").registrar(…).expires("2027-05-18").site(…).transfer_lock(…)`.
    domain = "Domain": leaf;
    /// A site: `.folder("../site/")`, the folder with its `site.json`; or one hosted elsewhere, `.url(…).host(…).project(…)`.
    site = "Site": leaf;
    /// An address the brand receives mail at: `.address("support@northwind.example").kind("support")`.
    mailbox = "Mailbox": leaf;
    /// A social handle: `.platform("x").handle("northwindsurvey").connection("x")`.
    handle = "Handle": leaf;
    /// A mark the company records as an IP asset: `.title("NORTHWIND").jurisdiction("US").number(…)`.
    mark = "Mark": leaf;
}

attributes! {
    /// The fields of a brand and its records, chained on their elements.
    pub trait BrandAttrs {
        name; tagline; about; voice; design_system;
        kind; locale; file; background;
        registrar; expires; auto_renew; site; transfer_lock;
        folder; host; project; address; platform; handle; url; connection;
        title; jurisdiction; number;
    }
}

pub(crate) const FAMILY: Family = Family { module: "brands", roots: &["BrandSurface"], declare };

pub const BRAND_FIELDS: &[&str] = &["name", "tagline", "about", "voice", "designSystem"];
pub const ALIAS_FIELDS: &[&str] = &["name", "kind", "locale"];
pub const LOGO_FIELDS: &[&str] = &["file", "kind", "background"];
pub const DOMAIN_FIELDS: &[&str] = &["name", "registrar", "expires", "autoRenew", "site", "transferLock"];
pub const SITE_FIELDS: &[&str] = &["folder", "url", "host", "project"];
pub const MAILBOX_FIELDS: &[&str] = &["address", "kind"];
pub const HANDLE_FIELDS: &[&str] = &["platform", "handle", "url", "connection"];
pub const MARK_FIELDS: &[&str] = &["title", "jurisdiction", "number"];

/// Each list of the document, the tag of its elements and the fields they take.
const LISTS: &[(&str, &str, &[&str])] = &[
    ("aliases", "Alias", ALIAS_FIELDS),
    ("logos", "Logo", LOGO_FIELDS),
    ("domains", "Domain", DOMAIN_FIELDS),
    ("sites", "Site", SITE_FIELDS),
    ("mailboxes", "Mailbox", MAILBOX_FIELDS),
    ("handles", "Handle", HANDLE_FIELDS),
    ("marks", "Mark", MARK_FIELDS),
];

const ROOT: &[&str] = &["BrandSurface"];
const BRAND: Vocabulary = Vocabulary {
    format: "brand",
    noun: "a brand",
    root: "BrandSurface",
    tags: &[
        TagRule::new("BrandSurface", &[]).required(&["name"]).attrs(BRAND_FIELDS),
        TagRule::new("Alias", ROOT).required(&["name"]).attrs(ALIAS_FIELDS),
        TagRule::new("Logo", ROOT).key("file").required(&["file"]).attrs(LOGO_FIELDS),
        TagRule::new("Domain", ROOT).key("name").required(&["name"]).attrs(DOMAIN_FIELDS),
        TagRule::new("Site", ROOT).attrs(SITE_FIELDS),
        TagRule::new("Mailbox", ROOT).key("address").required(&["address"]).attrs(MAILBOX_FIELDS),
        TagRule::new("Handle", ROOT).required(&["platform", "handle"]).attrs(HANDLE_FIELDS),
        TagRule::new("Mark", ROOT).required(&["title"]).attrs(MARK_FIELDS),
    ],
    from_tree: brand_of,
};

fn declare(root: El) -> Result<Declared, String> {
    declare_document(&root, &BRAND)
}

/// One attribute's type: text, a flag (`auto_renew`), or the voice's words; or an unknown or approximate fact of one.
fn check_one(tag: &str, k: &str, v: &Json) -> Result<Json, String> {
    let f = super::element::fn_name(tag);
    if let Some(fact) = fact_marker(v, &format!("{f}(): {}", rust_name(k)), |x| check_one(tag, k, x))? {
        return Ok(fact);
    }
    let (ok, what) = match k {
        "autoRenew" => (matches!(v, Json::Bool(_)), "true or false"),
        "voice" => (matches!(v, Json::Arr(xs) if xs.iter().all(|x| matches!(x, Json::Str(_)))), "a list of words"),
        _ => (matches!(v, Json::Str(_)), "text"),
    };
    if ok {
        Ok(v.clone())
    } else {
        Err(format!("{f}(): {} is {what}", rust_name(k)))
    }
}

fn check(t: &DocTree) -> Result<(), String> {
    for node in std::iter::once(t).chain(t.children.iter()) {
        for (k, v) in &node.attrs {
            check_one(node.tag, k, v)?;
        }
        if node.tag == "Site" && ["folder", "url", "project"].iter().all(|k| node.attr_of(k).is_none()) {
            return Err("site() names its folder, or the url or project of a site hosted elsewhere".into());
        }
    }
    Ok(())
}

/// The fields a record says: an absent field and an empty text are not said.
fn fields(t: &DocTree, keys: &[&str]) -> Json {
    let mut out = Json::obj();
    for k in keys {
        if let Some(v) = t.attr_of(k).filter(|v| **v != Json::from("")) {
            out.set(k, v.clone());
        }
    }
    out
}

fn brand_of(t: &DocTree) -> Result<Json, String> {
    check(t)?;
    let mut doc = fields(t, BRAND_FIELDS);
    for (list, tag, keys) in LISTS {
        let items: Vec<Json> = t.children.iter().filter(|c| c.tag == *tag).map(|c| fields(c, keys)).collect();
        doc.set(list, items);
    }
    Ok(doc)
}
