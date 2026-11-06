//! A COMPANY'S DEALS IN RUST — its sales pipeline, one `deal()` child of a `company()` for each deal (CommandAGI's
//! docs/formats.md § companies, "deals"). The same element, and the same document, as the TypeScript SDK's
//! `company-deals.ts` and the Python SDK's `company_deals.py`. `business` re-exports it, so
//! `use commandagi::design::business::*;` stays the file's one import:
//!
//! ```
//! use commandagi::design::business::*;
//!
//! fn document() -> El {
//!     company([
//!         deal().id("acme-pilot").title("Pilot of ten units").counterparty("relationships/cards/acme.vcf").stage("proposal")
//!             .value("120000").currency("USD").expected("2027-03-31").owner("ceo").since("2026-11-01"),
//!     ])
//!     .name("Northwind")
//! }
//! # commandagi::design::declare(document()).unwrap();
//! ```
//!
//! `counterparty` names who the deal is with: a ref to a card (a `.vcf`) or the id of a row of
//! `relationships/relationships.csv`. `stage` is `lead`, `qualified`, `proposal`, `negotiation`, `won` or `lost`.
//! `value` is the deal's worth as the file states it, in `currency`; a deal with no value is one whose worth is unknown.
//! The company document gains `deals` only when the file writes one; every total is computed by the app and never written.

use super::business::{BusinessNode, Kind::*, TagSpec};
use super::element::{attributes, elements};
use super::Json;

elements! {
    /// One deal of the pipeline: `.id("…").title("…").counterparty("…vcf").stage("proposal").value("120000")`.
    deal = "Deal": leaf;
}

attributes! {
    /// The fields of a deal that a company's other elements lack (`.value()`, `.since()`, `.note()` and `.source()`
    /// are `BusinessAttrs`'s, `.currency()` `FinanceAttrs`'s, `.owner()` and `.stage()` `GovernanceAttrs`'s).
    pub trait DealAttrs {
        counterparty; expected; closed;
    }
}

/// The tag's attributes.
pub(crate) const DEALS_TAGS: &[TagSpec] = &[TagSpec {
    tag: "Deal",
    props: &[
        ("id", Text),
        ("title", Text),
        ("counterparty", Text),
        ("stage", Text),
        ("value", Amount),
        ("currency", Text),
        ("expected", Text),
        ("closed", Text),
        ("owner", Text),
        ("since", Text),
        ("source", Text),
        ("note", Text),
    ],
    required: &["id", "title", "stage"],
    one: &[],
    many: &[],
}];

/// What a `company()`'s deals add to the company document: `deals`, only when the file writes one.
pub(crate) fn deals_of(node: &BusinessNode, company: Json) -> Json {
    let deals: Vec<Json> = node.many("deals").iter().map(|n| Json::Obj(n.props.clone())).collect();
    if deals.is_empty() {
        company
    } else {
        company.with("deals", deals)
    }
}
