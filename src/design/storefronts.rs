//! A STOREFRONT IN RUST — `<name>.storefront.rs` declares one storefront its holder lists (CommandAGI's docs/formats.md
//! § a storefront in code): the store, who sells there, and what it offers. The same elements, and the same document,
//! as the TypeScript SDK's `storefronts.ts` (`.storefront.tsx`) and the Python SDK's `storefronts.py`:
//!
//! ```
//! use commandagi::design::storefronts::*;
//!
//! fn document() -> El {
//!     storefront([offers().listings("Inventory/listings.csv").read("Inventory/read.mjs")])
//!         .name("DigiKey")
//!         .venue("digikey")
//!         .seller("DigiKey")
//!         .url("https://www.digikey.com")
//!         .hold("10 minutes")
//! }
//! # commandagi::design::declare(document()).unwrap();
//! ```
//!
//! The holder is the account whose Drive holds the file, never an attribute. `offer().name(…).kind("service")` is one
//! offer, by its name, with `price("25.00 USD")`, `unit`, classification `codes` (product schemes, [`super::classification`]) and `tags`.

use super::classification::{classification_problem, PRODUCT_SCHEMES};
use super::documents::{declare_document, DocTree, TagRule, Vocabulary};
use super::element::{attributes, elements};
use super::{Declared, Family};
pub use super::{fragment, json, El, Json, Value};

elements! {
    /// A storefront: at most one `offers()` and any `offer()`s.
    storefront = "Storefront": children;
    /// A mirrored store's listings and the script that keeps them: `.listings("Inventory/listings.csv").read("Inventory/read.mjs")`.
    offers = "Offers": leaf;
    /// One offer, by its name: `.name("Ride").kind("service").unit("ride")`.
    offer = "Offer": leaf;
}

attributes! {
    /// The fields of a storefront and of its offers, chained on their elements.
    pub trait StorefrontsAttrs {
        name; venue; seller; seller_record; seller_id; url; source; hold; listings; read; kind; price; unit; codes; tags;
    }
}

pub(crate) const FAMILY: Family = Family { module: "storefronts", roots: &["Storefront"], declare };

pub const STOREFRONT_FIELDS: &[&str] = &["name", "venue", "seller", "sellerRecord", "sellerId", "url", "source", "hold"];
pub const STOREFRONT_MIRROR_FIELDS: &[&str] = &["listings", "read"];
pub const STOREFRONT_OFFER_FIELDS: &[&str] = &["name", "kind", "price", "unit", "codes", "tags"];

const STOREFRONT: Vocabulary = Vocabulary {
    format: "storefront",
    noun: "a storefront",
    root: "Storefront",
    tags: &[
        TagRule::new("Storefront", &[]).required(&["name", "venue", "seller"]).attrs(STOREFRONT_FIELDS),
        TagRule::new("Offers", &["Storefront"]).single().required(&["listings"]).attrs(STOREFRONT_MIRROR_FIELDS),
        TagRule::new("Offer", &["Storefront"]).key("name").required(&["name", "kind"]).attrs(STOREFRONT_OFFER_FIELDS),
    ],
    from_tree: storefront_of,
};

fn declare(root: El) -> Result<Declared, String> {
    declare_document(&root, &STOREFRONT)
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

fn is_venue(s: &str) -> bool {
    let b = s.as_bytes();
    !b.is_empty() && b.len() <= 40 && (b[0].is_ascii_lowercase() || b[0].is_ascii_digit()) && b.iter().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || *c == b'-')
}

fn is_hold(s: &str) -> bool {
    let Some((n, unit)) = s.split_once(' ') else { return false };
    let digits = !n.is_empty() && n.len() <= 4 && n.bytes().all(|c| c.is_ascii_digit()) && !n.starts_with('0');
    digits && matches!(unit, "second" | "seconds" | "minute" | "minutes" | "hour" | "hours" | "day" | "days")
}

fn is_price(s: &str) -> bool {
    let Some((amount, currency)) = s.split_once(' ') else { return false };
    let (whole, frac) = match amount.split_once('.') {
        Some((w, f)) => (w, Some(f)),
        None => (amount, None),
    };
    let digits = |x: &str| !x.is_empty() && x.bytes().all(|c| c.is_ascii_digit());
    digits(whole) && frac.map_or(true, digits) && currency.len() == 3 && currency.bytes().all(|c| c.is_ascii_uppercase())
}

fn optional(t: &DocTree, k: &str, max: usize, at: &str, out: Json) -> Result<Json, String> {
    match t.attr_of(k) {
        None => Ok(out),
        Some(v) => match short(Some(v), max) {
            Some(s) => Ok(out.with(k, Json::Str(s))),
            None => Err(format!("{at} {k} is a short text (at most {max}), not {}", v.text())),
        },
    }
}

fn texts(v: &Json, ok: impl Fn(&str) -> bool) -> Option<Vec<Json>> {
    match v {
        Json::Arr(items) if items.len() <= 20 => items.iter().map(|x| match x {
            Json::Str(s) if ok(s) => Some(Json::Str(s.clone())),
            _ => None,
        }).collect(),
        _ => None,
    }
}

fn offer_of(c: &DocTree) -> Result<Json, String> {
    let Some(name) = short(c.attr_of("name"), 200) else {
        return Err(format!("<Offer> name is a short text (at most 200), not {}", said(c.attr_of("name"))));
    };
    let at = format!("<Offer name=\"{name}\">");
    let kind = match c.attr_of("kind") {
        Some(Json::Str(k)) if k == "service" || k == "good" => k.clone(),
        other => return Err(format!("{at} kind is service or good, not {}", said(other))),
    };
    let mut out = Json::obj().with("name", Json::Str(name)).with("kind", Json::Str(kind));
    if let Some(p) = c.attr_of("price") {
        match p {
            Json::Str(s) if is_price(s) => out = out.with("price", Json::Str(s.clone())),
            _ => return Err(format!("{at} price is an amount and an ISO currency (\"25.00 USD\"), not {}", p.text())),
        }
    }
    out = optional(c, "unit", 40, &at, out)?;
    // What kind of offer it is (super::classification): product schemes and free tags, by the one grammar.
    if let Some(problem) = classification_problem(c.attr_of("codes"), c.attr_of("tags"), Some(PRODUCT_SCHEMES)) {
        return Err(format!("{at} {problem}"));
    }
    for k in ["codes", "tags"] {
        if let Some(v) = c.attr_of(k) {
            out = out.with(k, v.clone());
        }
    }
    Ok(out)
}

/// A storefront as its tree declares it (the TypeScript SDK's `storefrontOf`).
pub fn storefront_of(t: &DocTree) -> Result<Json, String> {
    let Some(name) = short(t.attr_of("name"), 120) else {
        return Err(format!("<Storefront> name is a short text (at most 120), not {}", said(t.attr_of("name"))));
    };
    let at = format!("<Storefront name=\"{name}\">");
    let venue = match t.attr_of("venue") {
        Some(Json::Str(v)) if is_venue(v) => v.clone(),
        other => return Err(format!("{at} venue is the platform's word, lower-case letters, digits and dashes (\"digikey\", \"amazon\"), not {}", said(other))),
    };
    let Some(seller) = short(t.attr_of("seller"), 200) else {
        return Err(format!("{at} seller is the seller's name, a short text (at most 200), not {}", said(t.attr_of("seller"))));
    };
    let mut out = Json::obj().with("name", Json::Str(name)).with("venue", Json::Str(venue)).with("seller", Json::Str(seller));
    out = optional(t, "sellerRecord", 500, &at, out)?;
    out = optional(t, "sellerId", 200, &at, out)?;
    if let Some(u) = t.attr_of("url") {
        match short(Some(u), 500) {
            Some(s) if s.starts_with("https://") => out = out.with("url", Json::Str(s)),
            _ => return Err(format!("{at} url is an https:// address (at most 500), not {}", u.text())),
        }
    }
    out = optional(t, "source", 500, &at, out)?;
    if let Some(h) = t.attr_of("hold") {
        match h {
            Json::Str(s) if is_hold(s) => out = out.with("hold", Json::Str(s.clone())),
            _ => return Err(format!("{at} hold is a term like \"10 minutes\" (seconds, minutes, hours or days), not {}", h.text())),
        }
    }
    let mirrors: Vec<&DocTree> = t.children.iter().filter(|c| c.tag == "Offers").collect();
    if mirrors.len() > 1 {
        return Err(format!("{at} has one <Offers>, not {}", mirrors.len()));
    }
    if let Some(m) = mirrors.first() {
        let Some(listings) = short(m.attr_of("listings"), 500) else {
            return Err(format!("{at} <Offers> listings is a ref to the listings file (\"Inventory/listings.csv\"), not {}", said(m.attr_of("listings"))));
        };
        let mut mirror = Json::obj().with("listings", Json::Str(listings));
        if let Some(r) = m.attr_of("read") {
            let Some(read) = short(Some(r), 500) else {
                return Err(format!("{at} <Offers> read is a ref to the script that reads the store (\"Inventory/read.mjs\"), not {}", r.text()));
            };
            mirror = mirror.with("read", Json::Str(read));
        }
        out = out.with("mirror", mirror);
    }
    let offers: Vec<&DocTree> = t.children.iter().filter(|c| c.tag == "Offer").collect();
    if offers.len() > 500 {
        return Err(format!("{at} has at most 500 <Offer>s, not {}", offers.len()));
    }
    let mut list = Vec::new();
    let mut names: Vec<String> = Vec::new();
    for c in offers {
        let o = offer_of(c)?;
        let n = c.attr_of("name").map(Json::text).unwrap_or_default();
        if names.contains(&n) {
            return Err(format!("{at} has two <Offer>s named {n}"));
        }
        names.push(n);
        list.push(o);
    }
    Ok(out.with("offers", Json::Arr(list)))
}
