//! A COMPANY'S TEAM IN RUST — the people operations a `company()` declares in one `team()`, the same elements and the
//! same `team` as the TypeScript SDK's `people-ops.ts` and the Python SDK's `people_ops.py`:
//!
//! ```
//! use commandagi::design::business::*;
//!
//! fn document() -> El {
//!     company([
//!         people().folder("Northwind/people/"),
//!         team([
//!             role().id("ceo").title("Managing member"),
//!             role().id("pilot").title("Drone pilot").reports_to("ceo").band("ic-2").headcount(2).kind("employee"),
//!             band().id("ic-2").currency("USD").per("year").min("80000").max("110000"),
//!             benefit().id("health").kind("health").waiting_days(30),
//!             checklist().id("onboarding").when("start").steps(["Sign the offer"]),
//!         ])
//!         .hiring("Northwind/hiring/"),
//!     ])
//!     .name("Northwind")
//! }
//! # commandagi::design::declare(document()).unwrap();
//! ```
//!
//! The team holds what the company decides (roles and reporting lines, bands, benefit plans, checklists, the hiring
//! folder). The payroll provider is the company's finance (`payroll()`, `business_finance.rs`). People, payroll runs and grants stay their own files; no element takes a tax id, a
//! bank number or a date of birth. `business.rs` reads these tags with its own.

use super::business::{spec, BusinessNode, Kind::*, TagSpec};
use super::element::{attributes, elements};
use super::Json;

elements! {
    /// A company's team: its roles, bands, benefits and checklists.
    team = "Team": children;
    /// A role of the org chart: `.id("pilot").title("Drone pilot").reports_to("ceo").band("ic-2")`.
    role = "Role": leaf;
    /// A compensation band: `.id("ic-2").currency("USD").per("year").min("80000").max("110000")`.
    band = "Band": leaf;
    /// A benefit plan: `.id("health").kind("health").provider("…").renews("2027-01-01").waiting_days(30)`.
    benefit = "Benefit": leaf;
    /// An onboarding or offboarding checklist: `.id("onboarding").when("start").steps([…])`.
    checklist = "Checklist": leaf;
}

attributes! {
    /// The fields of a team's elements that the company's own and its finance's do not have, chained on them
    /// (`provider` and `currency` are `FinanceAttrs`': one method of a name on an element).
    pub trait PeopleOpsAttrs {
        hiring; offer_letter; department; reports_to; band; headcount; level; per; min; mid; max;
        policy; starts; renews; eligible; waiting_days; employer_share; when; kinds; steps;
    }
}

/// The team's tags, beside the company's (`business.rs` `BUSINESS_TAGS`).
pub(super) const PEOPLE_OPS_TAGS: &[TagSpec] = &[
    TagSpec {
        tag: "Team",
        props: &[("hiring", Text), ("offerLetter", Text)],
        required: &[],
        one: &[],
        many: &[("roles", "Role"), ("bands", "Band"), ("benefits", "Benefit"), ("checklists", "Checklist")],
    },
    spec("Role", &[("id", Text), ("title", Text), ("department", Text), ("reportsTo", Text), ("band", Text), ("headcount", Amount), ("kind", Text)], &["id", "title"]),
    spec(
        "Band",
        &[("id", Text), ("title", Text), ("level", Text), ("currency", Text), ("per", Text), ("min", Amount), ("mid", Amount), ("max", Amount)],
        &["id", "currency", "per", "min", "max"],
    ),
    spec(
        "Benefit",
        &[
            ("id", Text),
            ("kind", Text),
            ("name", Text),
            ("provider", Text),
            ("policy", Text),
            ("starts", Text),
            ("renews", Text),
            ("file", Text),
            ("eligible", Texts),
            ("waitingDays", Amount),
            ("employerShare", Amount),
        ],
        &["kind"],
    ),
    spec("Checklist", &[("id", Text), ("title", Text), ("when", Text), ("kinds", Texts), ("steps", Texts)], &["id", "when", "steps"]),
];

/// A company's `team` from its `team()`: each list in the file's order.
pub(super) fn team_of(node: &BusinessNode) -> Json {
    let list = |field: &str| node.many(field).iter().map(|n| Json::Obj(n.props.clone())).collect::<Vec<_>>();
    let out = Json::obj().with("hiring", node.prop("hiring").cloned().unwrap_or(Json::Null));
    let out = match node.prop("offerLetter") {
        Some(t) => out.with("offerLetter", t.clone()),
        None => out,
    };
    out.with("roles", list("roles"))
        .with("bands", list("bands"))
        .with("benefits", list("benefits"))
        .with("checklists", list("checklists"))
}
