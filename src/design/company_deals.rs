//! A COMPANY'S DEALS IN RUST — its sales pipeline, one `deal()` child of a `company()` for each deal, and the
//! `pipeline()` that says what each stage is worth as a chance (CommandAGI's docs/formats.md § companies, "deals"). The
//! same elements, and the same document, as the TypeScript SDK's `company-deals.ts` and the Python SDK's
//! `company_deals.py`. `business` re-exports them, so `use commandagi::design::business::*;` stays the file's one import:
//!
//! ```
//! use commandagi::design::business::*;
//!
//! fn document() -> El {
//!     company([
//!         pipeline([
//!             deal_stage().id("lead").probability(0.1), deal_stage().id("qualified").probability(0.25),
//!             deal_stage().id("proposal").probability(0.5), deal_stage().id("negotiation").probability(0.75),
//!         ])
//!         .stale_days(30),
//!         deal([deal_move().from("lead").to("proposal").at("2026-11-01T17:02:00Z").by("person:ada")])
//!             .id("acme-pilot").title("Pilot of ten units").counterparty("relationships/cards/acme.vcf").stage("proposal")
//!             .value("120000").currency("USD").expected("2027-03-31").owner("ceo").since("2026-11-01")
//!             .probability(0.3).probability_reason("their budget is not approved").source("the call of 2026-10-30"),
//!     ])
//!     .name("Northwind")
//! }
//! # commandagi::design::declare(document()).unwrap();
//! ```
//!
//! `pipeline()` lists the open stages in order, each with its default win probability (0 to 1); `won` (1) and `lost`
//! (0) close every pipeline and are not listed. A company with no `pipeline()` has the stages `lead`, `qualified`,
//! `proposal` and `negotiation` at 0.1, 0.25, 0.5 and 0.75 (the app's defaults). `stale_days` is how many days a deal
//! may go with no change before the app calls it stale.
//!
//! `counterparty` names who the deal is with: a ref to a card (a `.vcf` of the company's people or relationships
//! folder) or the id of a row of `relationships/relationships.csv`. `stage` is one of the pipeline's stages, `won` or
//! `lost` (the app refuses another word). `value` is the deal's worth as the file states it (text or a number) in
//! `currency`; a deal with no value is one whose worth is unknown, never one worth nothing. `probability` overrides its
//! stage's, and `probability_reason` says why (the app refuses one without the other). `expected` is the day it is
//! expected to close, `closed` the day it was won or lost, `owner` a role id of the team or a card ref, and `source`
//! says where the facts came from. Each `deal_move()` is one recorded stage change: from, to, when (`at`, an ISO 8601
//! instant) and by whom (`by`, a principal). The company document gains `deals` and `pipeline` only when the file
//! writes them; every total, weight and rate is computed by the app and never written.

use super::business::{BusinessNode, Kind::*, TagSpec};
use super::element::{attributes, elements};
use super::Json;

elements! {
    /// One deal of the pipeline, its recorded moves as children:
    /// `deal([deal_move()…]).id("…").title("…").counterparty("…vcf").stage("proposal").value("120000")`.
    deal = "Deal": children;
    /// One recorded stage change of a deal: `.from("lead").to("proposal").at("2026-11-01T17:02:00Z").by("person:ada")`.
    deal_move = "DealMove": leaf;
    /// The company's pipeline, its open stages as children: `pipeline([deal_stage()…]).stale_days(30)`.
    pipeline = "Pipeline": children;
    /// One open stage of the pipeline: `.id("proposal").label("proposal sent").probability(0.5)`.
    deal_stage = "DealStage": leaf;
}

attributes! {
    /// The fields of a deal, its moves and its pipeline that a company's other elements lack (`.value()`, `.since()`,
    /// `.note()`, `.id()`, `.title()` and `.source()` are `BusinessAttrs`'s, `.currency()` and `.closed()`
    /// `FinanceAttrs`'s, `.owner()` and `.stage()` `GovernanceAttrs`'s).
    pub trait DealAttrs {
        counterparty; expected; probability; probability_reason;
        from; to; at; by;
        stale_days; label;
    }
}

/// The tags' attributes and children.
pub(crate) const DEALS_TAGS: &[TagSpec] = &[
    TagSpec {
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
            ("probability", Amount),
            ("probabilityReason", Text),
            ("source", Text),
            ("note", Text),
        ],
        required: &["id", "title", "stage"],
        one: &[],
        many: &[("moves", "DealMove")],
    },
    TagSpec {
        tag: "DealMove",
        props: &[("from", Text), ("to", Text), ("at", Text), ("by", Text), ("note", Text)],
        required: &["to", "at"],
        one: &[],
        many: &[],
    },
    TagSpec { tag: "Pipeline", props: &[("staleDays", Amount)], required: &[], one: &[], many: &[("stages", "DealStage")] },
    TagSpec {
        tag: "DealStage",
        props: &[("id", Text), ("label", Text), ("probability", Amount)],
        required: &["id", "probability"],
        one: &[],
        many: &[],
    },
];

/// What a `company()`'s deals add to the company document: `pipeline` and `deals`, only when the file writes them.
pub(crate) fn deals_of(node: &BusinessNode, company: Json) -> Json {
    let props = |n: &BusinessNode| Json::Obj(n.props.clone());
    let company = match node.one("pipeline") {
        Some(p) => company.with("pipeline", props(p).with("stages", p.many("stages").iter().map(props).collect::<Vec<_>>())),
        None => company,
    };
    let deals: Vec<Json> = node
        .many("deals")
        .iter()
        .map(|d| {
            let moves: Vec<Json> = d.many("moves").iter().map(props).collect();
            if moves.is_empty() {
                props(d)
            } else {
                props(d).with("moves", moves)
            }
        })
        .collect();
    if deals.is_empty() {
        company
    } else {
        company.with("deals", deals)
    }
}
