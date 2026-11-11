//! A COMPANY'S LEGAL PROFILE, in Rust (docs/company-substrate.md § the legal profile): the facts a form asks for, so
//! an agent fills one from the company file alone. It adds the company's `website`, its addresses by purpose
//! (`address()`), its classes of stock (`stock()`) and its beneficial owners (`owner()`); the same elements as the
//! TypeScript SDK's `company-profile.ts` and the Python SDK's `company_profile.py`. A company names no mailbox and no
//! phone line: a form's contact email and phone are its responsible person's card:
//!
//! ```
//! use commandagi::design::business::*;
//!
//! fn document() -> El {
//!     company([
//!         entity().jurisdiction("US-DE").form("corporation").name("Acme Robotics, Inc.").formed("2026-01-05"),
//!         address().kind("legal").line1("1 Main St").city("Austin").region("TX").postal("78701").country("US"),
//!         stock().kind("common").authorized(10_000_000).issued(8_000_000).reserved(2_000_000).par("0.00001"),
//!         owner().name("Ada Founder").holder("people/ada-founder/").percent("100"),
//!     ])
//!     .name("Acme")
//!     .website("https://acme.example")
//! }
//! # commandagi::design::declare(document()).unwrap();
//! ```
//!
//! An address's `kind` is `legal` (the formation documents and the tax-id notice), `mailing`, `office` or another word.
//! The document holds them as `website`, `addresses`, `stock` and `owners`, each absent when the file
//! writes none. A company file with the profile's identifiers is private.

use super::business::{spec, BusinessNode, Kind::Amount, Kind::Text, TagSpec};
use super::element::{attributes, elements};
use super::Json;

elements! {
    /// An address of the company by purpose: `.kind("legal").line1("1 Main St").country("US")`.
    address = "Address": leaf;
    /// A class of stock the charter authorizes: `.kind("common").authorized(10_000_000).issued(…)`.
    stock = "Stock": leaf;
    /// A beneficial owner: `.name("Ada Founder").percent("100")`.
    owner = "Owner": leaf;
}

attributes! {
    /// The fields of the legal profile that a company's other elements lack (`.kind()`, `.name()`, `.since()`,
    /// `.source()` and `.issued()` are `BusinessAttrs`'s, `.holder()` `GovernanceAttrs`'s).
    pub trait ProfileAttrs {
        website; line1; line2; city; region; postal; country; authorized; reserved; par; percent;
    }
}

/// The attributes this part adds to `company()` (text; `business.rs` lists them in the company's tag).
pub(crate) const PROFILE_COMPANY_PROPS: &[&str] = &["website"];

/// The tags this part adds.
pub(crate) const PROFILE_TAGS: &[TagSpec] = &[
    spec(
        "Address",
        &[("kind", Text), ("line1", Text), ("line2", Text), ("city", Text), ("region", Text), ("postal", Text), ("country", Text), ("source", Text)],
        &["kind", "line1", "country"],
    ),
    spec("Stock", &[("kind", Text), ("authorized", Amount), ("issued", Amount), ("reserved", Amount), ("par", Text), ("source", Text)], &["kind", "authorized"]),
    spec("Owner", &[("name", Text), ("holder", Text), ("percent", Amount), ("since", Text), ("source", Text)], &["name", "percent"]),
];

/// The lists of `company()` they fill.
pub(crate) const PROFILE_LISTS: &[(&str, &str)] = &[("addresses", "Address"), ("stock", "Stock"), ("owners", "Owner")];

/// The company document with its profile fields, each only when the file writes it.
pub(crate) fn profile_of(node: &BusinessNode, company: Json) -> Json {
    let mut company = company;
    for k in PROFILE_COMPANY_PROPS {
        if let Some(v) = node.prop(k) {
            company = company.with(k, v.clone());
        }
    }
    for (field, _) in PROFILE_LISTS {
        let list: Vec<Json> = node.many(field).iter().map(|x| Json::Obj(x.props.clone())).collect();
        if !list.is_empty() {
            company = company.with(field, list);
        }
    }
    company
}
