//! A MARKET RECORDING IN RUST — a `.market.rs`: quotes, trades, books, news, orders and their outcomes, and a venue's
//! account read back, one element per record, in time order. The same elements, and the same document, as the
//! TypeScript SDK's `market.ts` and the Python SDK's `market.py`:
//!
//! ```
//! use commandagi::design::market::*;
//!
//! fn document() -> El {
//!     market([
//!         quote().t("2026-10-01T13:30:00.000Z").sym("NRTH").last("185.09").bid("185.05").ask("185.13"),
//!         book([bid().price("185.05").qty("300"), ask().price("185.13").qty("200")]).t("2026-10-01T13:30:01.000Z").sym("NRTH"),
//!         order().t("2026-10-01T13:31:00.000Z").id("o-1").sym("NRTH").side("buy").qty("10").type_("limit").limit("184.50")
//!             .tif("day").venue("sim").mode("sim").by("person:ada").grant("g-sim"),
//!         order_update().t("2026-10-01T13:31:01.000Z").order("o-1").state("accepted").mode("sim"),
//!     ])
//! }
//! # commandagi::design::declare(document()).unwrap();
//! ```
//!
//! A recording grows: the host writes each new record as one element just before the closing `])`, so an empty
//! recording is `market([])`. Every field is text. A price or a quantity is decimal text (`"185.09"`), never a number:
//! a float would round it. `t` is an ISO time (`2026-10-01T13:30:00.000Z`). A tag with a hyphen in TypeScript is a
//! function with `_` here (`order-update` is `order_update()`); a field in camel case is a method in snake case
//! (`buyingPower` is `.buying_power(…)`); `type` and `ref` are `.type_(…)` and `.ref_(…)`.
//!
//! A run gives back `{format: "market", document: {records}, sources}` beside an empty graph. Each record is
//! `{kind, …fields}` with the TypeScript tag and names. A book's `bid()` and `ask()` children are its `bids` and `asks`
//! (`[price, qty]` pairs, in order). An account's `holding()`, `open_order()` and `problem()` children are its
//! `positions`, `open` and `problems`, each said only when present. What a record means (a venue's modes, the shape of
//! an id, the order of times) is the workbench's to check, not this module's.

use super::documents::{declare_document, DocTree, TagRule, Vocabulary};
use super::element::{attributes, elements, fn_name, rust_name};
use super::{Declared, Family};
pub use super::{fragment, json, El, Json, Value};

elements! {
    /// A market recording: its records, in time order.
    market: children;
    /// A note a person or the host wrote: `.t(…).text(…)`.
    note: leaf;
    /// A quote: `.t(…).sym("NRTH").last("185.09").bid(…).ask(…).vol(…)`.
    quote: leaf;
    /// A trade printed on the tape: `.t(…).sym(…).price(…).qty(…).side(…)`.
    trade: leaf;
    /// An order book at one time: its `bid()` and `ask()` levels, in order.
    book: children;
    /// One bid level of a book: `.price(…).qty(…)`.
    bid: leaf;
    /// One ask level of a book: `.price(…).qty(…)`.
    ask: leaf;
    /// A headline: `.t(…).headline(…).sym(…).source(…).tone(…)`.
    news: leaf;
    /// A position held: `.t(…).sym(…).qty(…).avg(…)`.
    position: leaf;
    /// A fill of an order: `.t(…).sym(…).side(…).price(…).qty(…)`.
    fill: leaf;
    /// An order sent, recorded before it is sent, with the principal and the grant that sent it.
    order: leaf;
    /// What became of an order: `.t(…).order(…).state(…).mode(…)`.
    order_update = "order-update": leaf;
    /// A cancel sent for an order.
    cancel: leaf;
    /// What became of a cancel.
    cancel_update = "cancel-update": leaf;
    /// An amend sent for an order.
    amend: leaf;
    /// What became of an amend.
    amend_update = "amend-update": leaf;
    /// A venue's account read back: its `holding()`, `open_order()` and `problem()` children.
    account: children;
    /// One holding of an account.
    holding: leaf;
    /// One open order of an account, as the venue says it.
    open_order = "open-order": leaf;
    /// What went wrong while the account was read: `.text(…)`.
    problem: leaf;
}

attributes! {
    /// The fields of every record, chained on their elements. Each value is text.
    pub trait MarketAttrs {
        t; text; sym; last; bid; ask; vol; price; qty; side; headline; source; tone; avg;
        by; order; mode; ref_; at; id; type_; tif; venue; grant; limit; stop; take_profit; stop_loss; instrument; account; connection;
        state; filled; error; src; cancel; amend; leg;
        currency; status; value; buying_power; cash; equity; day_change; day_change_pct;
        pl; pl_pct; client; submitted; avg_price;
    }
}

pub(crate) const FAMILY: Family = Family { module: "market", roots: &["market"], declare };

/// The fields of a holding (an account's `positions`).
pub const HOLDING_FIELDS: &[&str] = &["sym", "qty", "side", "avg", "price", "value", "pl", "plPct"];
/// The fields of an open order (an account's `open`).
pub const OPEN_ORDER_FIELDS: &[&str] =
    &["sym", "status", "ref", "client", "order", "side", "type", "tif", "state", "submitted", "qty", "filled", "avgPrice", "limit"];

const R: &[&str] = &["market"];

const MARKET: Vocabulary = Vocabulary {
    format: "market",
    noun: "a market recording",
    root: "market",
    tags: &[
        TagRule::new("market", &[]).attrs(&[]),
        TagRule::new("note", R).required(&["t", "text"]).attrs(&["t", "text"]),
        TagRule::new("quote", R).required(&["t", "sym"]).attrs(&["t", "sym", "last", "bid", "ask", "vol"]),
        TagRule::new("trade", R).required(&["t", "sym", "price", "qty"]).attrs(&["t", "sym", "price", "qty", "side"]),
        TagRule::new("book", R).required(&["t", "sym"]).attrs(&["t", "sym"]),
        TagRule::new("bid", &["book"]).required(&["price", "qty"]).attrs(&["price", "qty"]),
        TagRule::new("ask", &["book"]).required(&["price", "qty"]).attrs(&["price", "qty"]),
        TagRule::new("news", R).required(&["t", "headline"]).attrs(&["t", "headline", "sym", "source", "tone"]),
        TagRule::new("position", R).required(&["t", "sym", "qty", "avg"]).attrs(&["t", "sym", "qty", "avg"]),
        TagRule::new("fill", R)
            .required(&["t", "sym", "side", "price", "qty"])
            .attrs(&["t", "sym", "side", "price", "qty", "by", "order", "leg", "mode", "ref", "at"]),
        TagRule::new("order", R)
            .required(&["t", "id", "sym", "side", "qty", "type", "tif", "venue", "mode", "by", "grant"])
            .attrs(&["t", "id", "sym", "side", "qty", "type", "tif", "venue", "mode", "by", "grant", "limit", "stop", "takeProfit", "stopLoss", "instrument", "account", "connection"]),
        TagRule::new("order-update", R)
            .required(&["t", "order", "state", "mode"])
            .attrs(&["t", "order", "state", "mode", "leg", "ref", "filled", "avg", "error", "src"]),
        TagRule::new("cancel", R)
            .required(&["t", "id", "venue", "mode", "ref", "by", "grant"])
            .attrs(&["t", "id", "venue", "mode", "ref", "by", "grant", "order", "connection", "account"]),
        TagRule::new("cancel-update", R).required(&["t", "cancel", "state", "mode"]).attrs(&["t", "cancel", "state", "mode", "error"]),
        TagRule::new("amend", R)
            .required(&["t", "id", "order", "sym", "side", "qty", "type", "tif", "venue", "mode", "ref", "by", "grant"])
            .attrs(&["t", "id", "order", "sym", "side", "qty", "type", "tif", "venue", "mode", "ref", "by", "grant", "limit", "stop", "connection", "account"]),
        TagRule::new("amend-update", R).required(&["t", "amend", "state", "mode"]).attrs(&["t", "amend", "state", "mode", "ref", "error"]),
        TagRule::new("account", R).required(&["t", "venue", "mode"]).attrs(&[
            "t", "venue", "mode", "connection", "currency", "status", "value", "buyingPower", "cash", "equity", "dayChange", "dayChangePct",
        ]),
        TagRule::new("holding", &["account"]).required(&["sym", "qty"]).attrs(HOLDING_FIELDS),
        TagRule::new("open-order", &["account"]).required(&["sym", "status"]).attrs(OPEN_ORDER_FIELDS),
        TagRule::new("problem", &["account"]).required(&["text"]).attrs(&["text"]),
    ],
    from_tree: market_of,
};

/// The fields of each tag that are decimal text.
const DECIMALS: &[(&str, &[&str])] = &[
    ("quote", &["last", "bid", "ask", "vol"]),
    ("trade", &["price", "qty"]),
    ("bid", &["price", "qty"]),
    ("ask", &["price", "qty"]),
    ("position", &["qty", "avg"]),
    ("fill", &["price", "qty"]),
    ("order", &["qty", "limit", "stop", "takeProfit", "stopLoss"]),
    ("order-update", &["filled", "avg"]),
    ("amend", &["qty", "limit", "stop"]),
    ("account", &["value", "buyingPower", "cash", "equity", "dayChange", "dayChangePct"]),
    ("holding", &["qty", "avg", "price", "value", "pl", "plPct"]),
    ("open-order", &["qty", "filled", "avgPrice", "limit"]),
];

/// An empty recording's document: what `market([])` declares.
pub fn empty() -> Json {
    Json::obj().with("records", Vec::<Json>::new())
}

fn declare(root: El) -> Result<Declared, String> {
    declare_document(&root, &MARKET)
}

/// `^-?\d+(\.\d+)?$`: no exponent, no `+`, no separators, as the TypeScript and Python SDKs read it.
fn is_decimal(s: &str) -> bool {
    let s = s.strip_prefix('-').unwrap_or(s);
    let (whole, frac) = match s.split_once('.') {
        Some((w, f)) => (w, Some(f)),
        None => (s, None),
    };
    let digits = |p: &str| !p.is_empty() && p.bytes().all(|b| b.is_ascii_digit());
    digits(whole) && frac.is_none_or(digits)
}

/// `^\d{4}-\d\d-\d\dT\d\d:\d\d(:\d\d(\.\d+)?)?(Z|[+-]\d\d:\d\d)$`, the shape all three SDKs check.
fn is_iso_time(s: &str) -> bool {
    fn digits(b: &[u8], n: usize) -> bool {
        b.len() >= n && b[..n].iter().all(u8::is_ascii_digit)
    }
    let b = s.as_bytes();
    if b.len() < 17 {
        return false;
    }
    let head = digits(b, 4) && b.get(4) == Some(&b'-') && digits(&b[5..], 2) && b.get(7) == Some(&b'-') && digits(&b[8..], 2);
    if !(head && b.get(10) == Some(&b'T') && digits(&b[11..], 2) && b.get(13) == Some(&b':') && digits(&b[14..], 2)) {
        return false;
    }
    let mut rest = &b[16..];
    if rest.first() == Some(&b':') {
        if !digits(&rest[1..], 2) {
            return false;
        }
        rest = &rest[3..];
        if rest.first() == Some(&b'.') {
            let n = rest[1..].iter().take_while(|c| c.is_ascii_digit()).count();
            if n == 0 {
                return false;
            }
            rest = &rest[1 + n..];
        }
    }
    rest == b"Z" || (rest.len() == 6 && matches!(rest[0], b'+' | b'-') && digits(&rest[1..], 2) && rest[3] == b':' && digits(&rest[4..], 2))
}

/// Every field is text; a decimal field is plain decimal text and `t` is an ISO time.
fn check(t: &DocTree) -> Result<(), String> {
    let at = format!("{}()", fn_name(t.tag));
    let decimals = DECIMALS.iter().find(|(tag, _)| *tag == t.tag).map_or(&[][..], |(_, d)| *d);
    for (k, v) in &t.attrs {
        let name = rust_name(k);
        let decimal = decimals.contains(&k.as_str());
        let s = match v {
            Json::Str(s) => s,
            Json::Num(_) if decimal => return Err(format!(r#"{at}: {name} is decimal text ("185.09"), not a number"#)),
            Json::Num(_) => return Err(format!("{at}: {name} is text, not a number")),
            other => return Err(format!("{at}: {name} is text, not {}", other.text())),
        };
        if decimal && !is_decimal(s) {
            return Err(format!(r#"{at}: {name} is decimal text ("185.09"), not {s:?}"#));
        }
        if k == "t" && !is_iso_time(s) {
            return Err(format!(r#"{at}: t is an ISO time ("2026-10-01T13:30:00.000Z"), not {s:?}"#));
        }
    }
    t.children.iter().try_for_each(check)
}

fn record_of(t: &DocTree) -> Json {
    let mut out = Json::obj().with("kind", t.tag);
    for (k, v) in &t.attrs {
        out.set(k, v.clone());
    }
    match t.tag {
        "book" => {
            let levels = |side: &str| -> Vec<Json> {
                t.children
                    .iter()
                    .filter(|c| c.tag == side)
                    .map(|c| Json::from(vec![c.attr_of("price").cloned().unwrap_or(Json::Null), c.attr_of("qty").cloned().unwrap_or(Json::Null)]))
                    .collect()
            };
            out.set("bids", levels("bid"));
            out.set("asks", levels("ask"));
        }
        "account" => {
            for (tag, key) in [("holding", "positions"), ("open-order", "open"), ("problem", "problems")] {
                let items: Vec<Json> = t
                    .children
                    .iter()
                    .filter(|c| c.tag == tag)
                    .map(|c| if tag == "problem" { c.attr_of("text").cloned().unwrap_or(Json::Null) } else { c.record() })
                    .collect();
                if !items.is_empty() {
                    out.set(key, items);
                }
            }
        }
        _ => {}
    }
    out
}

fn market_of(t: &DocTree) -> Result<Json, String> {
    t.children.iter().try_for_each(check)?;
    Ok(Json::obj().with("records", t.children.iter().map(record_of).collect::<Vec<_>>()))
}
