//! A COMPANY'S FORMATION PARTS, in Rust (docs/company-substrate.md § formation and filings): its registered agent in each
//! place (`registered_agent()`, a child of `company()`), what a `paper().kind("filing")` says about the filing it is,
//! and the day its dissolution was authorized (`entity().dissolved(…)`); the same elements as the TypeScript SDK's
//! `company-formation.ts` and the Python SDK's `company_formation.py`:
//!
//! ```
//! use commandagi::design::business::*;
//!
//! fn document() -> El {
//!     company([
//!         entity().jurisdiction("US-DE").form("corporation").formed("2026-01-05"),
//!         registered_agent().jurisdiction("US-DE").name("Agent Co.").address("1 Main St, Dover, DE 19901"),
//!         paper([])
//!             .kind("filing")
//!             .title("Certificate of Incorporation")
//!             .file("Acme/legal/filings/incorporation.md")
//!             .jurisdiction("US-DE")
//!             .rule("de-certificate-of-incorporation")
//!             .status("accepted")
//!             .submitted("2026-01-04")
//!             .accepted("2026-01-05"),
//!     ])
//!     .name("Acme")
//! }
//! # commandagi::design::declare(document()).unwrap();
//! ```
//!
//! A filing's `rule` names the jurisdiction catalog's filing it follows, `period` the year an annual filing reports,
//! `due` its day. Its status goes `draft`, then `submitted` (a person submitted it), then `accepted` or `rejected`. An
//! agent drafts and never submits. The company document holds the agents as `registeredAgents`, absent when the file
//! writes none.

use super::business::{spec, Kind::Text, TagSpec};
use super::element::{attributes, elements};

elements! {
    /// A registered agent in one place: `.jurisdiction("US-DE").name("Agent Co.").address("…")`.
    registered_agent = "RegisteredAgent": leaf;
}

attributes! {
    /// The fields of a registered agent and of a filing paper, and `dissolved` of the entity, chained on their elements.
    pub trait FormationAttrs {
        rule; period; due; submitted; accepted; confirmation; address; dissolved;
    }
}

/// The tags this part adds: their attributes (all text) and what each needs.
pub(crate) const FORMATION_TAGS: &[TagSpec] =
    &[spec("RegisteredAgent", &[("jurisdiction", Text), ("name", Text), ("address", Text), ("since", Text)], &["jurisdiction", "name"])];
