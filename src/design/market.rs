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
//!
//! An option contract is one `sym` in one spelling, `<underlying> <expiry> <C|P> <strike>[ x<multiplier>]`
//! (`"AAPL 2026-12-18 C 200"`): an ISO date, C or P, the strike as decimal text with no trailing zeros, and the
//! multiplier only when it is not 100. [`option_symbol`] writes it and [`parse_option`] reads it, as the TypeScript
//! SDK's `optionSymbol` and `parseOption` do; a `sym` with a space in any record must be a contract in that spelling. An
//! option `order()` may say `.effect("open")` or `.effect("close")`.

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
        by; order; mode; ref_; at; id; type_; tif; venue; grant; limit; stop; take_profit; stop_loss; effect; instrument; account; connection;
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
            .attrs(&["t", "id", "sym", "side", "qty", "type", "tif", "venue", "mode", "by", "grant", "limit", "stop", "takeProfit", "stopLoss", "effect", "instrument", "account", "connection"]),
        TagRule::new("order-update", R)
            .required(&["t", "order", "state", "mode"])
            .attrs(&["t", "order", "state", "mode", "leg", "ref", "filled", "avg", "error", "src"]),
        TagRule::new("cancel", R)
            .required(&["t", "id", "venue", "mode", "ref", "by", "grant"])
            .attrs(&["t", "id", "venue", "mode", "ref", "by", "grant", "order", "leg", "connection", "account"]),
        TagRule::new("cancel-update", R).required(&["t", "cancel", "state", "mode"]).attrs(&["t", "cancel", "state", "mode", "error"]),
        TagRule::new("amend", R)
            .required(&["t", "id", "order", "sym", "side", "qty", "type", "tif", "venue", "mode", "ref", "by", "grant"])
            .attrs(&["t", "id", "order", "sym", "side", "qty", "type", "tif", "venue", "mode", "ref", "by", "grant", "limit", "stop", "effect", "connection", "account"]),
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

/// The multiplier a contract has when its symbol writes none: the standard US equity option.
pub const STANDARD_MULTIPLIER: &str = "100";
const OPTION_FORM: &str = r#"<underlying> <expiry YYYY-MM-DD> <C|P> <strike>[ x<multiplier>], for example "AAPL 2026-12-18 C 200""#;

/// An option contract's parts, as text: the expiry an ISO date, the right `C` or `P`, the strike and the multiplier
/// decimal text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OptionContract {
    pub underlying: String,
    pub expiry: String,
    pub right: String,
    pub strike: String,
    pub multiplier: String,
}

/// Decimal text without leading or trailing zeros (`"0200.50"` is `"200.5"`), or None.
fn canonical(s: &str) -> Option<String> {
    let s = s.trim();
    let (whole, frac) = s.split_once('.').map_or((s, None), |(w, f)| (w, Some(f)));
    let digits = |p: &str| !p.is_empty() && p.bytes().all(|b| b.is_ascii_digit());
    if !digits(whole) || !frac.is_none_or(digits) {
        return None;
    }
    let whole = whole.trim_start_matches('0');
    let whole = if whole.is_empty() { "0" } else { whole };
    let frac = frac.unwrap_or("").trim_end_matches('0');
    Some(if frac.is_empty() { whole.to_string() } else { format!("{whole}.{frac}") })
}

/// `YYYY-MM-DD`, and a day the month has.
fn is_date(s: &str) -> bool {
    let b = s.as_bytes();
    if b.len() != 10 || b[4] != b'-' || b[7] != b'-' || !b.iter().enumerate().all(|(i, c)| i == 4 || i == 7 || c.is_ascii_digit()) {
        return false;
    }
    let (y, m, d): (u32, u32, u32) = (s[..4].parse().unwrap_or(0), s[5..7].parse().unwrap_or(0), s[8..].parse().unwrap_or(0));
    let leap = (y % 4 == 0 && y % 100 != 0) || y % 400 == 0;
    let days = match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => 0,
    };
    (1..=days).contains(&d)
}

/// The contract a symbol names, or `Ok(None)` for a plain symbol (one with no space). A symbol with a space is a
/// contract and must be spelled as one, as the TypeScript SDK's `parseOption` reads it: anything else is an error that
/// names the part and the spelling to write.
pub fn parse_option(sym: &str) -> Result<Option<OptionContract>, String> {
    if !sym.contains(' ') {
        return Ok(None);
    }
    let parts: Vec<&str> = sym.split(' ').collect();
    if !(4..=5).contains(&parts.len()) || parts.iter().any(|p| p.is_empty()) {
        return Err(format!("an option contract is {OPTION_FORM}, not {sym:?}"));
    }
    let (underlying, expiry, right, strike) = (parts[0], parts[1], parts[2], parts[3]);
    if underlying.len() > 24 || !underlying.bytes().all(|b| b.is_ascii_alphanumeric() || b"._/-".contains(&b)) {
        return Err(format!("an option's underlying is letters, digits and . _ / - (at most 24), not {underlying:?}"));
    }
    if !is_date(expiry) {
        return Err(format!("an option's expiry is a date, YYYY-MM-DD, not {expiry:?}"));
    }
    if right != "C" && right != "P" {
        return Err(format!("an option's right is C (a call) or P (a put), not {right:?}"));
    }
    let k = match canonical(strike) {
        Some(k) if k.bytes().any(|b| (b'1'..=b'9').contains(&b)) => k,
        _ => return Err(format!("an option's strike is decimal text above 0, not {strike:?}")),
    };
    if k != strike {
        return Err(format!("an option's strike is written {k}, not {strike} (one spelling for each contract)"));
    }
    let mut multiplier = STANDARD_MULTIPLIER.to_string();
    if let Some(mult) = parts.get(4) {
        let n = mult.strip_prefix('x').filter(|n| !n.is_empty() && !n.starts_with('0') && n.bytes().all(|b| b.is_ascii_digit()));
        let Some(n) = n else {
            return Err(format!(r#"an option's multiplier is x and a whole number above 0 ("x10"), not {mult:?}"#));
        };
        if n == STANDARD_MULTIPLIER {
            return Err(format!("a multiplier of {STANDARD_MULTIPLIER} is not written: {} is that contract", parts[..4].join(" ")));
        }
        multiplier = n.to_string();
    }
    Ok(Some(OptionContract { underlying: underlying.into(), expiry: expiry.into(), right: right.into(), strike: strike.into(), multiplier }))
}

/// A contract's one spelling, from its parts: `right` is C, P, call or put; `strike` and `multiplier` are decimal text
/// (`None` is 100, which is not written). An error as `parse_option` gives one.
pub fn option_symbol(underlying: &str, expiry: &str, right: &str, strike: &str, multiplier: Option<&str>) -> Result<String, String> {
    let r = match right.to_ascii_lowercase().as_str() {
        "c" | "call" => "C",
        "p" | "put" => "P",
        _ => right,
    };
    let k = canonical(strike).unwrap_or_else(|| strike.to_string());
    let m = multiplier.map_or(STANDARD_MULTIPLIER.to_string(), |m| canonical(m).unwrap_or_else(|| m.to_string()));
    let sym = if m == STANDARD_MULTIPLIER { format!("{underlying} {expiry} {r} {k}") } else { format!("{underlying} {expiry} {r} {k} x{m}") };
    parse_option(&sym)?;
    Ok(sym)
}

/// Every field is text; a decimal field is plain decimal text, `t` is an ISO time, and a `sym` with a space is an
/// option contract in its one spelling.
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
        if k == "sym" {
            parse_option(s).map_err(|e| format!("{at}: sym: {e}"))?;
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
