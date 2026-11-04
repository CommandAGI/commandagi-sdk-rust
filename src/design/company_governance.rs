//! A COMPANY'S GOVERNANCE, COMPLIANCE, IP AND INSURANCE IN RUST — the children of a `company()` that hold them, and
//! the `vote()` children of a `paper()` (CommandAGI's docs/company-substrate.md § governance, compliance, IP and
//! insurance). The same elements, and the same documents, as the TypeScript SDK's `company-governance.ts` and the
//! Python SDK's `company_governance.py`. `business` re-exports them, so `use commandagi::design::business::*;` stays
//! the file's one import:
//!
//! ```
//! use commandagi::design::business::*;
//!
//! fn document() -> El {
//!     company([
//!         seat().body("board").title("director").holder("Northwind/people/Lin Chen.vcf").start("2024-01-31"),
//!         compliance().industries(["naics:541370"]),
//!         evidence().obligation("US-DE:de-llc-lp-annual-tax").due("2026-06-01").file("Northwind/filings/2026.pdf"),
//!         ip_asset().kind("trademark").stage("registered").title("NORTHWIND").jurisdiction("US").registered("2025-02-11"),
//!         policy().kind("aviation").carrier("Skyward Mutual").start("2025-07-01").end("2026-07-01"),
//!         paper([vote().holder("Northwind/people/Lin Chen.vcf").choice("for")]).kind("board-consent").file("c.pdf").body("board"),
//!     ])
//!     .name("Northwind")
//! }
//! # commandagi::design::declare(document()).unwrap();
//! ```
//!
//! The company document gains `governance`, `compliance`, `ip` and `insurance` only when the file writes one of their
//! elements. Every document stays its own file, named by a ref; tallies and statuses are computed by the app.

use super::business::{BusinessNode, Kind, Kind::*, TagSpec};
use super::element::{attributes, elements};
use super::Json;

elements! {
    /// A seat of a body (the board, the officers, a committee): `.body("board").title("director").holder("…vcf")`.
    seat = "Seat": leaf;
    /// The compliance register's settings: the company's industry codes.
    compliance = "Compliance": leaf;
    /// An obligation of the company's own (a certification, a contract's report): `.id("…").name("…").due("…")`.
    obligation = "Obligation": leaf;
    /// The evidence that one occurrence of an obligation was done: `.obligation("<key>").due("…").file("…")`.
    evidence = "Evidence": leaf;
    /// A patent, trademark, copyright, design or trade secret the company holds.
    ip_asset = "IpAsset": leaf;
    /// A person's signed inventions assignment.
    assignment = "Assignment": leaf;
    /// An insurance policy.
    policy = "Policy": leaf;
    /// One vote on a paper (a consent, a resolution, minutes): `.holder("…vcf").choice("for").evidence("…pdf")`.
    vote = "Vote": leaf;
}

attributes! {
    /// The fields of seats, obligations, evidence, IP assets, policies and votes that a company's other elements lack
    /// (`.due()` is `FormationAttrs`'s, `.currency()` `FinanceAttrs`'s).
    pub trait GovernanceAttrs {
        holder; start; end; appointed_by; industries; every; url; authority; obligation; done; stage; number; filed;
        registered; owner; classes; person; signed; carrier; limit; broker; threshold; quorum; choice;
    }
}

const fn leaf(tag: &'static str, props: &'static [(&'static str, Kind)], required: &'static [&'static str]) -> TagSpec {
    TagSpec { tag, props, required, one: &[], many: &[] }
}

/// The tags' attributes.
pub(crate) const GOVERNANCE_TAGS: &[TagSpec] = &[
    leaf("Seat", &[("body", Text), ("title", Text), ("holder", Text), ("start", Text), ("end", Text), ("appointedBy", Text)], &["body", "title", "holder"]),
    leaf("Compliance", &[("industries", Texts)], &[]),
    leaf("Obligation", &[("id", Text), ("name", Text), ("authority", Text), ("url", Text), ("due", Text), ("every", Text), ("note", Text)], &["id", "name", "due"]),
    leaf("Evidence", &[("obligation", Text), ("due", Text), ("done", Text), ("file", Text), ("note", Text)], &["obligation", "due"]),
    leaf(
        "IpAsset",
        &[
            ("kind", Text),
            ("stage", Text),
            ("title", Text),
            ("jurisdiction", Text),
            ("number", Text),
            ("filed", Text),
            ("registered", Text),
            ("expires", Text),
            ("owner", Text),
            ("classes", Texts),
            ("file", Text),
        ],
        &["kind", "title"],
    ),
    leaf("Assignment", &[("person", Text), ("signed", Text), ("file", Text)], &["person"]),
    leaf(
        "Policy",
        &[
            ("kind", Text),
            ("carrier", Text),
            ("number", Text),
            ("start", Text),
            ("end", Text),
            ("limit", Amount),
            ("currency", Text),
            ("broker", Text),
            ("file", Text),
        ],
        &["kind", "carrier", "start", "end"],
    ),
    leaf("Vote", &[("holder", Text), ("choice", Text), ("signed", Text), ("evidence", Text)], &["holder", "choice"]),
];

fn props_of(node: &BusinessNode, field: &str) -> Vec<Json> {
    node.many(field).iter().map(|n| Json::Obj(n.props.clone())).collect()
}

/// What a `company()`'s governance children add to the company document: only the parts the file writes.
pub(crate) fn governance_of(node: &BusinessNode, mut company: Json) -> Json {
    if !node.many("seats").is_empty() {
        company.set("governance", Json::obj().with("seats", props_of(node, "seats")));
    }
    if node.one("compliance").is_some() || !node.many("obligations").is_empty() || !node.many("evidence").is_empty() {
        let industries = node.one("compliance").and_then(|n| n.prop("industries")).cloned().unwrap_or(Json::Arr(Vec::new()));
        company.set(
            "compliance",
            Json::obj().with("industries", industries).with("obligations", props_of(node, "obligations")).with("evidence", props_of(node, "evidence")),
        );
    }
    if !node.many("ipAssets").is_empty() || !node.many("assignments").is_empty() {
        company.set("ip", Json::obj().with("assets", props_of(node, "ipAssets")).with("assignments", props_of(node, "assignments")));
    }
    if !node.many("policies").is_empty() {
        company.set("insurance", Json::obj().with("policies", props_of(node, "policies")));
    }
    company
}
