//! A COMPANY'S FINANCE IN RUST — the elements `business.rs` reads inside a `company()` (the TypeScript SDK's
//! `business-finance.ts`, the Python SDK's `business_finance.py`; CommandAGI docs/company-substrate.md § finance):
//!
//! ```
//! use commandagi::design::business::*;
//!
//! fn document() -> El {
//!     company([
//!         books().journal("Northwind/books/main.journal"),
//!         budget().journal("Northwind/books/budget.journal"),
//!         bank_account().name("Operating").provider("First Bank").last4("4321").currency("USD")
//!             .ledger("Assets:Cash:Operating").kind("checking").statements("Northwind/bank/operating/"),
//!         payroll().provider("Gusto").ledger("Expenses:Payroll").register("Northwind/payroll/").schedule("semimonthly"),
//!         data_room([include().document("Northwind/Documents/Charter.pdf").section("corporate")]).name("Seed").expires("2026-12-31"),
//!     ])
//!     .name("Northwind")
//! }
//! # commandagi::design::declare(document()).unwrap();
//! ```
//!
//! Each element only refs files. A bank account names its journal account (`ledger`) and the folder of its
//! statements; at most the last four digits of its number, never the number. `finance_of` gives the company's
//! `finance` (None when it names none of these).

use super::business::{BusinessNode, Kind::Text, TagSpec};
use super::element::{attributes, elements};
use super::Json;

elements! {
    /// The budget: an hledger journal of periodic transactions, by its path.
    budget = "Budget": leaf;
    /// A bank account: `.name("Operating").ledger("Assets:Cash:Operating").statements("bank/")`. Never its number.
    bank_account = "BankAccount": leaf;
    /// A payroll: `.provider("Gusto").ledger("Expenses:Payroll").register("payroll/")`.
    payroll = "Payroll": leaf;
    /// A data room: the documents a share of it reaches, and the day it ends.
    data_room = "DataRoom": children;
    /// One document of a data room: `.document("Documents/charter.pdf").section("corporate")`.
    include = "Include": leaf;
}

attributes! {
    /// The fields of a company's finance that the company's own elements do not have.
    pub trait FinanceAttrs {
        provider; last4; currency; ledger; statements; opened; closed; register; schedule; document; section;
    }
}

const fn leaf(tag: &'static str, props: &'static [(&'static str, super::business::Kind)], required: &'static [&'static str]) -> TagSpec {
    TagSpec { tag, props, required, one: &[], many: &[] }
}

/// The finance tags.
pub(super) const FINANCE_TAGS: &[TagSpec] = &[
    leaf("Budget", &[("journal", Text)], &["journal"]),
    leaf(
        "BankAccount",
        &[
            ("name", Text),
            ("provider", Text),
            ("last4", Text),
            ("currency", Text),
            ("ledger", Text),
            ("kind", Text),
            ("statements", Text),
            ("opened", Text),
            ("closed", Text),
        ],
        &["name"],
    ),
    leaf("Payroll", &[("provider", Text), ("ledger", Text), ("register", Text), ("schedule", Text), ("jurisdiction", Text)], &["provider"]),
    TagSpec { tag: "DataRoom", props: &[("name", Text), ("about", Text), ("expires", Text)], required: &["name"], one: &[], many: &[("include", "Include")] },
    leaf("Include", &[("document", Text), ("section", Text)], &["document"]),
];

/// The children a `company()` takes for its finance, by the field each fills.
pub(super) const COMPANY_FINANCE_ONE: &[(&str, &str)] = &[("budget", "Budget")];
pub(super) const COMPANY_FINANCE_MANY: &[(&str, &str)] = &[("bankAccounts", "BankAccount"), ("payroll", "Payroll"), ("dataRooms", "DataRoom")];

/// A company's `finance` from its `company()` node, or None when it names none of it.
pub fn finance_of(node: &BusinessNode) -> Option<Json> {
    let rows = |field: &str| node.many(field).iter().map(|n| Json::Obj(n.props.clone())).collect::<Vec<_>>();
    let budget = node.one("budget").and_then(|n| n.prop("journal")).cloned().unwrap_or(Json::Null);
    let (banks, payroll) = (rows("bankAccounts"), rows("payroll"));
    let rooms: Vec<Json> = node
        .many("dataRooms")
        .iter()
        .map(|n| Json::Obj(n.props.clone()).with("include", n.many("include").iter().map(|i| Json::Obj(i.props.clone())).collect::<Vec<_>>()))
        .collect();
    let named = !matches!(budget, Json::Null) || !banks.is_empty() || !payroll.is_empty() || !rooms.is_empty();
    named.then(|| Json::obj().with("budget", budget).with("bankAccounts", banks).with("payroll", payroll).with("dataRooms", rooms))
}
