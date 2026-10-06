//! SIGNED RECORDS IN RUST — a market contract (`.contract.rs`) and a product instance (`.instance.rs`), written as
//! elements. The same elements, and the same documents, as the TypeScript SDK's `records.ts` and the Python SDK's
//! `records.py`:
//!
//! ```
//! use commandagi::design::records::*;
//!
//! fn document() -> El {
//!     instance([
//!         event().seq(0).kind("manufacture").at(1772409600000_i64).prev(json("null")).by("org-acme").sig("ed25519:…"),
//!     ])
//!     .serial("ACME-FU-0031")
//!     .product_id("fleet-unit")
//! }
//! # commandagi::design::declare(document()).unwrap();
//! ```
//!
//! The rule of the ontology's files: a record is an element and its fields are the element's attributes. These
//! records are SIGNED, so nothing is rewritten on the way: a contract's `terms` are ONE attribute (the exact object
//! both parties sign, `json(r#"{…}"#)`, nulls and empty strings kept), times stay epoch milliseconds (an integer past
//! `i32` is written with its type: `1780765664000_i64`), signatures are written as they are. A contract's parties are
//! `party()` children in order; an instance's provenance chain is `event()` children, oldest first, each named by its
//! `seq`. Nothing adds a default. A run gives back `{format, document, sources}` beside an empty graph; `sources`
//! names each element's site by its path (`""`, `party@0`, `event#0`).

use super::documents::{declare_document, js_string, DocTree, TagRule, Vocabulary};
use super::element::{attributes, elements};
use super::{Declared, Family};
pub use super::{fragment, json, El, Json, Value};

elements! {
    /// A contract record: its parties.
    contract: children;
    /// A party of a contract, its fields as stored: `.principal("u-1").role("offeror")`.
    party: leaf;
    /// A product instance: its provenance chain.
    instance: children;
    /// An event of an instance's chain: `.seq(0).kind("manufacture").at(…).by("org-acme")`.
    event: leaf;
}

attributes! {
    /// The fields of a contract record, a party, a product instance and an event, chained on their elements.
    pub trait RecordsAttrs {
        id; terms; created_at;
        principal; role; acted_by; agent; sig; signed_at;
        serial; product_id; model;
        seq; kind; at; prev; by; to; contract_ref; attestation_ref; data;
    }
}

pub(crate) const FAMILY: Family = Family { module: "records", roots: &["contract", "instance"], declare };

/// The fields of a contract record (packages/domain/core/src/contract-document.ts `ContractDoc`), but its parties.
pub const CONTRACT_FIELDS: &[&str] = &["id", "terms", "createdAt"];
/// The fields of a product instance (`ProductInstance`), but its events.
pub const INSTANCE_FIELDS: &[&str] = &["serial", "productId", "model"];
/// The fields of a provenance event (`InstanceEvent`).
pub const INSTANCE_EVENT_FIELDS: &[&str] = &["seq", "kind", "at", "prev", "by", "to", "contractRef", "attestationRef", "data", "sig"];

const CONTRACT: Vocabulary = Vocabulary {
    format: "contract",
    noun: "a contract record",
    root: "contract",
    tags: &[
        TagRule::new("contract", &[]).required(&["id", "terms", "createdAt"]).attrs(CONTRACT_FIELDS),
        // A party's fields are the record's as stored (principal, role, actedBy, agent, sig, signedAt, …): any.
        TagRule::new("party", &["contract"]).required(&["principal", "role"]),
    ],
    from_tree: contract_of,
};

const INSTANCE: Vocabulary = Vocabulary {
    format: "instance",
    noun: "a product instance",
    root: "instance",
    tags: &[
        TagRule::new("instance", &[]).required(&["serial", "productId"]).attrs(INSTANCE_FIELDS),
        TagRule::new("event", &["instance"]).key("seq").required(&["seq", "kind", "at", "by"]).attrs(INSTANCE_EVENT_FIELDS),
    ],
    from_tree: instance_of,
};

fn declare(root: El) -> Result<Declared, String> {
    declare_document(&root, if root.tag == "contract" { &CONTRACT } else { &INSTANCE })
}

fn field(t: &DocTree, name: &str) -> Json {
    t.attr(name).cloned().unwrap_or(Json::Null)
}

fn contract_of(t: &DocTree) -> Result<Json, String> {
    let at = format!("contract().id({})", Json::from(js_string(t.attr("id"))).text());
    if !matches!(t.attr("terms"), Some(Json::Obj(_))) {
        return Err(format!("{at}: terms is the object the parties sign"));
    }
    match t.attr("createdAt") {
        Some(Json::Num(n)) if n.is_finite() && n.fract() == 0.0 => {}
        other => {
            let said = other.map(Json::text).unwrap_or_else(|| "undefined".into());
            return Err(format!("{at}: created_at is epoch milliseconds, not {said}"));
        }
    }
    let parties: Vec<Json> = t.children.iter().map(DocTree::record).collect();
    Ok(Json::obj()
        .with("id", field(t, "id"))
        .with("terms", field(t, "terms"))
        .with("parties", parties)
        .with("createdAt", field(t, "createdAt")))
}

fn instance_of(t: &DocTree) -> Result<Json, String> {
    let events: Vec<Json> = t.children.iter().map(DocTree::record).collect();
    for (i, e) in events.iter().enumerate() {
        if e.get("seq") != Some(&Json::Num(i as f64)) {
            let said = e.get("seq").map(Json::text).unwrap_or_else(|| "undefined".into());
            return Err(format!("event().seq({said}) is event {i} of the chain: events are written oldest first, seq 0, 1, 2 …"));
        }
    }
    let mut out = Json::obj();
    for k in INSTANCE_FIELDS {
        if let Some(v) = t.attr(k) {
            out.set(k, v.clone());
        }
    }
    Ok(out.with("events", events))
}
