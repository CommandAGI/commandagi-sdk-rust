//! A COMPANY'S PRICED ROUND, PRO FORMA, IN RUST — the same `<Round>` and `<Investment>` as the TypeScript SDK's
//! `business-equity.ts` and the Python SDK's `business_equity.py` (docs/formats.md § companies, "a round, pro forma";
//! docs/company-substrate.md § equity instruments):
//!
//! ```
//! use commandagi::design::business::*;
//!
//! fn document() -> El {
//!     company([
//!         cap_table().ocf("Kite Works/captable/"),
//!         round([investment().stakeholder("Lead Fund").amount("5000000")])
//!             .name("Series A").date("2027-03-01").pre_money("20000000").currency("USD").pool_target("0.10"),
//!     ])
//!     .name("Kite Works")
//! }
//! # commandagi::design::declare(document()).unwrap();
//! ```
//!
//! A round is a model, not a record: the cap table (Open Cap Table Format) holds what happened, and the company app
//! computes the price, each SAFE's and note's conversion and every holder's part after it. The company document's
//! `rounds` lists each round with its investments, in the order written.

use super::super::element::{attributes, elements};
use super::{spec, BusinessNode, Json, Kind::*, TagSpec};

elements! {
    /// A priced round before it closes: its investments.
    round = "Round": children;
    /// One investment in a round: `.stakeholder("Lead Fund").amount("5000000")`.
    investment = "Investment": leaf;
}

attributes! {
    /// The fields of a round and its investments that a company's other elements do not have (`name`, `date`,
    /// `currency` and `amount` are theirs).
    pub trait EquityAttrs {
        pre_money; raise; pool_target; pool_top_up; price_places; stock_class; stakeholder; instrument;
    }
}

/// The tags (the vocabulary of `business.rs`, which takes them in).
pub(super) const EQUITY_TAGS: &[TagSpec] = &[
    TagSpec {
        tag: "Round",
        props: &[
            ("name", Text),
            ("date", Text),
            ("preMoney", Amount),
            ("raise", Amount),
            ("currency", Text),
            ("poolTarget", Amount),
            ("poolTopUp", Amount),
            ("pricePlaces", Amount),
            ("stockClass", Text),
        ],
        required: &["name", "preMoney"],
        one: &[],
        many: &[("investments", "Investment")],
    },
    spec("Investment", &[("stakeholder", Text), ("amount", Amount), ("instrument", Text)], &["stakeholder", "amount"]),
];

/// The company document's `rounds`: each round's attributes and its investments, in the order written.
pub(super) fn rounds_of(node: &BusinessNode) -> Vec<Json> {
    node.many("rounds")
        .iter()
        .map(|r| Json::Obj(r.props.clone()).with("investments", r.many("investments").iter().map(|i| Json::Obj(i.props.clone())).collect::<Vec<_>>()))
        .collect()
}
