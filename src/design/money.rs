//! A MONEY ACCOUNT IN RUST — `<name>.money.rs` declares one account a person or a company holds at a financial
//! institution (CommandAGI's docs/financial-accounts.md): its connections, its counterparties and their payees, and its
//! recurring payments. The same elements, and the same document, as the TypeScript SDK's `money.ts` (`.money.tsx`) and
//! the Python SDK's `money.py`:
//!
//! ```
//! use commandagi::design::money::*;
//!
//! fn document() -> El {
//!     money_account([
//!         connection().id("direct").provider("mercury").account("acc_test_operating"),
//!         counterparty([at().connection("direct").recipient("rcp_test_landlord")]).id("landlord").name("Harbor Properties LLC"),
//!         recurring().id("rent").to("landlord").amount("4200.00").rail("ach").every("month").day(1).by("person:jacob").starts("2027-01-01"),
//!     ])
//!     .name("Operating")
//!     .kind("checking")
//!     .last4("0000")
//!     .currency("USD")
//! }
//! # commandagi::design::declare(document()).unwrap();
//! ```
//!
//! The file holds no bank or card number and no credential: `last4` is the most it says of a number, `account` and
//! `recipient` are the provider's own ids, and the sign-in is a connection of the holder.

use super::documents::{declare_document, DocTree, TagRule, Vocabulary};
use super::element::{attributes, elements};
use super::{Declared, Family};
pub use super::{fragment, json, El, Json, Value};

elements! {
    /// A money account: its `connection()`s, `counterparty()`s and `recurring()` payments.
    money_account = "MoneyAccount": children;
    /// One way to reach the account: `.id("direct").provider("mercury").account("acc_…")`.
    connection = "Connection": leaf;
    /// Someone the account pays or is paid by: its `at()`s, or the `.file(…)` of another money account of the holder.
    counterparty = "Counterparty": children;
    /// The payee at one connection: `.connection("direct").recipient("rcp_…")`.
    at = "At": leaf;
    /// A payment that repeats: `.id("rent").to("landlord").amount("4200.00").rail("ach").every("month")`.
    recurring = "Recurring": leaf;
}

attributes! {
    /// The fields of a money account and of its elements, chained on them.
    pub trait MoneyAttrs {
        name; kind; institution; last4; currency; ledger; books; statements; opened; closed; holder;
        id; provider; account; mode; file; connection; recipient; to; amount; rail; every; day; by; starts; ends; memo;
    }
}

pub(crate) const FAMILY: Family = Family { module: "money", roots: &["MoneyAccount"], declare };

pub const MONEY_KINDS: &[&str] = &["checking", "savings", "money-market", "card", "loan", "credit-line", "brokerage", "payments", "wallet", "other"];
pub const MONEY_RAILS: &[&str] = &["ach", "wire", "book", "rtp", "check"];
pub const MONEY_EVERY: &[&str] = &["week", "month", "quarter", "year"];
pub const MONEY_ACCOUNT_FIELDS: &[&str] = &["name", "kind", "institution", "last4", "currency", "ledger", "books", "statements", "opened", "closed", "holder"];
pub const MONEY_CONNECTION_FIELDS: &[&str] = &["id", "provider", "account", "mode"];
pub const MONEY_COUNTERPARTY_FIELDS: &[&str] = &["id", "name", "kind", "file"];
pub const MONEY_AT_FIELDS: &[&str] = &["connection", "recipient"];
pub const MONEY_RECURRING_FIELDS: &[&str] = &["id", "to", "amount", "rail", "every", "day", "by", "starts", "ends", "memo"];

const MONEY: Vocabulary = Vocabulary {
    format: "money",
    noun: "a money account",
    root: "MoneyAccount",
    tags: &[
        TagRule::new("MoneyAccount", &[]).required(&["name", "kind"]).attrs(MONEY_ACCOUNT_FIELDS),
        TagRule::new("Connection", &["MoneyAccount"]).key("id").required(&["id", "provider", "account"]).attrs(MONEY_CONNECTION_FIELDS),
        TagRule::new("Counterparty", &["MoneyAccount"]).key("id").required(&["id", "name"]).attrs(MONEY_COUNTERPARTY_FIELDS),
        // An `at()` is named by its counterparty and its connection: no key of its own is unique in the whole document.
        TagRule::new("At", &["Counterparty"]).required(&["connection", "recipient"]).attrs(MONEY_AT_FIELDS),
        TagRule::new("Recurring", &["MoneyAccount"]).key("id").required(&["id", "to", "amount", "rail", "every", "by", "starts"]).attrs(MONEY_RECURRING_FIELDS),
    ],
    from_tree: money_account_of,
};

fn declare(root: El) -> Result<Declared, String> {
    declare_document(&root, &MONEY)
}

fn said(v: Option<&Json>) -> String {
    v.map(Json::text).unwrap_or_else(|| "undefined".into())
}

fn short(v: Option<&Json>, max: usize) -> Option<String> {
    match v {
        Some(Json::Str(s)) if !s.trim().is_empty() && s.encode_utf16().count() <= max => Some(s.clone()),
        _ => None,
    }
}

fn str_of(v: Option<&Json>) -> Option<&str> {
    v.and_then(Json::as_str)
}

/// A provider is named by its id; the platform's catalog names the providers (docs/financial-accounts.md), so a new one needs no SDK release.
fn provider_id(v: Option<&Json>) -> bool {
    let Some(Json::Str(s)) = v else { return false };
    let b = s.as_bytes();
    !b.is_empty() && b.len() <= 40 && b[0].is_ascii_lowercase() && b.iter().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || *c == b'-')
}

fn one_of(v: Option<&Json>, list: &[&str]) -> bool {
    str_of(v).map_or(false, |s| list.contains(&s))
}

fn is_id(s: &str) -> bool {
    let b = s.as_bytes();
    !b.is_empty() && b.len() <= 64 && b[0].is_ascii_alphanumeric() && b.iter().all(|c| c.is_ascii_alphanumeric() || matches!(c, b'.' | b'_' | b'-'))
}

fn is_day(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() == 10 && b[4] == b'-' && b[7] == b'-' && b.iter().enumerate().all(|(i, c)| i == 4 || i == 7 || c.is_ascii_digit())
}

fn is_decimal(s: &str) -> bool {
    let (whole, frac) = match s.split_once('.') {
        Some((w, f)) => (w, Some(f)),
        None => (s, None),
    };
    let digits = |x: &str, most: usize| !x.is_empty() && x.len() <= most && x.bytes().all(|c| c.is_ascii_digit());
    digits(whole, 15) && frac.map_or(true, |f| digits(f, 4))
}

fn is_principal(s: &str) -> bool {
    let rest = s.strip_prefix("person:").or_else(|| s.strip_prefix("agent:"));
    rest.map_or(false, |r| !r.is_empty() && r.len() <= 64 && r.bytes().all(|c| c.is_ascii_alphanumeric() || matches!(c, b'.' | b'_' | b'@' | b'-')))
}

fn is_money_file(s: &str) -> bool {
    let l = s.to_ascii_lowercase();
    [".money.tsx", ".money.jsx", ".money.py", ".money.rs"].iter().any(|e| l.ends_with(e))
}

/// The attributes of `fields` the tree has, in that order.
fn keep(t: &DocTree, fields: &[&str], mut out: Json) -> Json {
    for k in fields {
        if let Some(v) = t.attr_of(k) {
            if out.get(k).is_none() {
                out = out.with(k, v.clone());
            }
        }
    }
    out
}

fn texts(t: &DocTree, at: &str, fields: &[&str], max: usize) -> Result<(), String> {
    for k in fields {
        if let Some(v) = t.attr_of(k) {
            if short(Some(v), max).is_none() {
                return Err(format!("{at} {k} is a short text (at most {max}), not {}", v.text()));
            }
        }
    }
    Ok(())
}

fn id_of(t: &DocTree, tag: &str) -> Result<String, String> {
    match str_of(t.attr_of("id")) {
        Some(id) if is_id(id) => Ok(format!("<{tag} id=\"{id}\">")),
        _ => Err(format!("<{tag}> id is letters, digits, . _ - (at most 64), not {}", said(t.attr_of("id")))),
    }
}

fn connection_of(t: &DocTree) -> Result<Json, String> {
    let at = id_of(t, "Connection")?;
    if !provider_id(t.attr_of("provider")) {
        return Err(format!("{at} provider is a provider's id (lowercase letters, digits and -, as plaid or increase), not {}", said(t.attr_of("provider"))));
    }
    if short(t.attr_of("account"), 200).is_none() {
        return Err(format!("{at} account is the provider's own id of the account (not a bank number), not {}", said(t.attr_of("account"))));
    }
    if t.attr_of("mode").is_some() && !one_of(t.attr_of("mode"), &["live", "sandbox"]) {
        return Err(format!("{at} mode is live or sandbox, not {}", said(t.attr_of("mode"))));
    }
    Ok(keep(t, MONEY_CONNECTION_FIELDS, Json::obj()))
}

fn counterparty_of(t: &DocTree, connections: &[String]) -> Result<Json, String> {
    let at = id_of(t, "Counterparty")?;
    if short(t.attr_of("name"), 200).is_none() {
        return Err(format!("{at} name is a short text (at most 200), not {}", said(t.attr_of("name"))));
    }
    texts(t, &at, &["kind"], 80)?;
    let file = t.attr_of("file");
    if let Some(f) = file {
        if !short(Some(f), 1000).map_or(false, |s| is_money_file(&s)) {
            return Err(format!("{at} file is a ref to another money account (.money.tsx), not {}", f.text()));
        }
    }
    let mut ways = Vec::new();
    for w in t.children.iter().filter(|c| c.tag == "At") {
        let c = str_of(w.attr_of("connection")).filter(|c| connections.iter().any(|x| x == c));
        let Some(c) = c else {
            return Err(format!("{at} <At> connection is the id of a <Connection> of the account, not {}", said(w.attr_of("connection"))));
        };
        let Some(r) = short(w.attr_of("recipient"), 200) else {
            return Err(format!("{at} <At connection=\"{c}\"> recipient is the provider's id of the payee, not {}", said(w.attr_of("recipient"))));
        };
        if ways.iter().any(|x: &Json| x.get("connection").and_then(Json::as_str) == Some(c)) {
            return Err(format!("{at} names the connection {c} in two <At>s"));
        }
        ways.push(Json::obj().with("connection", Json::Str(c.to_string())).with("recipient", Json::Str(r)));
    }
    if file.is_some() && !ways.is_empty() {
        return Err(format!("{at} is another money account of the holder (file) or is reached through <At>, not both"));
    }
    if file.is_none() && ways.is_empty() {
        return Err(format!("{at} needs an <At connection recipient>, or the file of another money account"));
    }
    let out = keep(t, MONEY_COUNTERPARTY_FIELDS, Json::obj());
    Ok(if ways.is_empty() { out } else { out.with("at", Json::Arr(ways)) })
}

fn recurring_of(t: &DocTree, parties: &[String]) -> Result<Json, String> {
    let at = id_of(t, "Recurring")?;
    if !str_of(t.attr_of("to")).map_or(false, |s| parties.iter().any(|p| p == s)) {
        return Err(format!("{at} to is the id of a <Counterparty> of the account, not {}", said(t.attr_of("to"))));
    }
    if !str_of(t.attr_of("amount")).map_or(false, is_decimal) {
        return Err(format!("{at} amount is a decimal text (\"4200.00\"), not {}", said(t.attr_of("amount"))));
    }
    if !one_of(t.attr_of("rail"), MONEY_RAILS) {
        return Err(format!("{at} rail is {}, not {}", MONEY_RAILS.join(", "), said(t.attr_of("rail"))));
    }
    if !one_of(t.attr_of("every"), MONEY_EVERY) {
        return Err(format!("{at} every is {}, not {}", MONEY_EVERY.join(", "), said(t.attr_of("every"))));
    }
    let mut out = keep(t, MONEY_RECURRING_FIELDS, Json::obj());
    if let Some(d) = t.attr_of("day") {
        let n = match d {
            Json::Num(n) => Some(*n),
            Json::Str(s) if !s.is_empty() && s.len() <= 2 && s.bytes().all(|c| c.is_ascii_digit()) => s.parse::<f64>().ok(),
            _ => None,
        };
        let most = if str_of(t.attr_of("every")) == Some("week") { 7.0 } else { 31.0 };
        match n {
            Some(n) if n.fract() == 0.0 && (1.0..=most).contains(&n) => out = out.with("day", Json::Num(n)),
            _ => return Err(format!("{at} day is a whole number from 1 to {most}, not {}", d.text())),
        }
    }
    if !str_of(t.attr_of("by")).map_or(false, is_principal) {
        return Err(format!("{at} by is the principal whose grant makes each payment (person:<id> or agent:<id>), not {}", said(t.attr_of("by"))));
    }
    for k in ["starts", "ends"] {
        if let Some(v) = t.attr_of(k) {
            if !str_of(Some(v)).map_or(false, is_day) {
                return Err(format!("{at} {k} is a day (YYYY-MM-DD), not {}", v.text()));
            }
        }
    }
    if t.attr_of("starts").is_none() {
        return Err(format!("{at} needs starts"));
    }
    texts(t, &at, &["memo"], 140)?;
    Ok(out)
}

/// A money account as its tree declares it (the TypeScript SDK's `moneyAccountOf`).
pub fn money_account_of(t: &DocTree) -> Result<Json, String> {
    let Some(name) = short(t.attr_of("name"), 120) else {
        return Err(format!("<MoneyAccount> name is a short text (at most 120), not {}", said(t.attr_of("name"))));
    };
    let at = format!("<MoneyAccount name=\"{name}\">");
    if !one_of(t.attr_of("kind"), MONEY_KINDS) {
        return Err(format!("{at} kind is {}, not {}", MONEY_KINDS.join(", "), said(t.attr_of("kind"))));
    }
    if let Some(v) = t.attr_of("last4") {
        if !str_of(Some(v)).map_or(false, |s| s.len() == 4 && s.bytes().all(|c| c.is_ascii_digit())) {
            return Err(format!("{at} last4 is the last four digits and no more, not {}", v.text()));
        }
    }
    if let Some(v) = t.attr_of("currency") {
        if !str_of(Some(v)).map_or(false, |s| s.len() == 3 && s.bytes().all(|c| c.is_ascii_uppercase())) {
            return Err(format!("{at} currency is three capitals (ISO 4217), not {}", v.text()));
        }
    }
    for k in ["opened", "closed"] {
        if let Some(v) = t.attr_of(k) {
            if !str_of(Some(v)).map_or(false, is_day) {
                return Err(format!("{at} {k} is a day (YYYY-MM-DD), not {}", v.text()));
            }
        }
    }
    texts(t, &at, &["institution", "ledger", "books", "statements", "holder"], 500)?;
    let out = keep(t, MONEY_ACCOUNT_FIELDS, Json::obj());
    let connections = t.children.iter().filter(|c| c.tag == "Connection").map(connection_of).collect::<Result<Vec<_>, _>>()?;
    let ids: Vec<String> = connections.iter().filter_map(|c| c.get("id").and_then(Json::as_str).map(str::to_string)).collect();
    let counterparties = t.children.iter().filter(|c| c.tag == "Counterparty").map(|c| counterparty_of(c, &ids)).collect::<Result<Vec<_>, _>>()?;
    let parties: Vec<String> = counterparties.iter().filter_map(|c| c.get("id").and_then(Json::as_str).map(str::to_string)).collect();
    let recurring = t.children.iter().filter(|c| c.tag == "Recurring").map(|c| recurring_of(c, &parties)).collect::<Result<Vec<_>, _>>()?;
    Ok(out.with("connections", Json::Arr(connections)).with("counterparties", Json::Arr(counterparties)).with("recurring", Json::Arr(recurring)))
}
