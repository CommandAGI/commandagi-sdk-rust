//! A GOOD IN RUST — a `<name>.good.rs` declares one good a seller sells. The same elements, and the same document, as
//! the TypeScript SDK's `goods.ts` (`.good.tsx`) and the Python SDK's `goods.py`. The document is the body of the
//! platform's goods door (`POST /goods`); each `variant` is one record of it, by its `sku`:
//!
//! ```
//! use commandagi::design::goods::*;
//!
//! fn document() -> El {
//!     good([variant().sku("helmet-stl").name("STL pack").price_cents(1500).file("./helmet.zip")])
//!         .name("Helmet files")
//!         .kind("digital")
//!         .status("active")
//! }
//! # commandagi::design::declare(document()).unwrap();
//! ```
//!
//! `kind` is `physical` (`shipping` is its policy, required) or `digital` (each variant names the `file` it sells; no
//! shipping, weight or deposit). A `deposit` (`{"percent": 1..99}` or `{"cents": n}`, at least 50 and less than the
//! price) makes a variant a pre-order. A run gives back `{format: "good", document, sources}` beside an empty graph;
//! `sources` names each variant's site by its sku (`variant#helmet-stl`).

use super::documents::{declare_document, DocTree, TagRule, Vocabulary};
use super::element::{attributes, elements};
use super::{Declared, Family};
pub use super::{fragment, json, El, Json, Value};

elements! {
    /// A good: one `variant()` each.
    good: children;
    /// One variant: `.sku("helmet-stl").name("STL pack").price_cents(1500)`.
    variant: leaf;
}

attributes! {
    /// The fields of a good and of its variants, chained on their elements.
    pub trait GoodsAttrs {
        name; kind; slug; subtitle; description; images; status; shipping;
        sku; price_cents; inventory_qty; weight_grams; file; deposit;
    }
}

pub(crate) const FAMILY: Family = Family { module: "goods", roots: &["good"], declare };

pub const GOOD_KINDS: &[&str] = &["physical", "digital"];
pub const GOOD_STATUSES: &[&str] = &["draft", "active", "archived"];
pub const GOOD_FIELDS: &[&str] = &["name", "kind", "slug", "subtitle", "description", "images", "status", "shipping"];
pub const VARIANT_FIELDS: &[&str] = &["sku", "name", "priceCents", "inventoryQty", "weightGrams", "file", "deposit"];

const GOOD: Vocabulary = Vocabulary {
    format: "good",
    noun: "a good",
    root: "good",
    tags: &[
        TagRule::new("good", &[]).required(&["name", "kind"]).attrs(GOOD_FIELDS),
        TagRule::new("variant", &["good"]).key("sku").required(&["sku", "name", "priceCents"]).attrs(VARIANT_FIELDS),
    ],
    from_tree: good_of,
};

fn declare(root: El) -> Result<Declared, String> {
    declare_document(&root, &GOOD)
}

/// JavaScript's `length` of a string: its UTF-16 code units.
fn js_len(s: &str) -> usize {
    s.encode_utf16().count()
}

fn is_text(v: Option<&Json>, max: usize) -> bool {
    matches!(v, Some(Json::Str(s)) if !s.trim().is_empty() && js_len(s) <= max)
}

/// A whole number at least `min` (and a safe integer, as JavaScript's `Number.isSafeInteger`).
fn whole(v: Option<&Json>, min: f64) -> Option<f64> {
    match v {
        Some(Json::Num(n)) if n.fract() == 0.0 && *n >= min && n.abs() <= 9_007_199_254_740_991.0 => Some(*n),
        _ => None,
    }
}

fn said(v: Option<&Json>) -> String {
    v.map(Json::text).unwrap_or_else(|| "undefined".into())
}

/// A deposit as a variant declares it, or why it does not read (the TypeScript SDK's `depositOf`).
pub fn deposit_of(v: &Json, price: f64, at: &str) -> Result<Json, String> {
    let one = matches!(v, Json::Obj(fields) if fields.len() == 1);
    if one {
        if let Some(p) = v.get("percent") {
            return match whole(Some(p), 1.0).filter(|n| *n <= 99.0) {
                Some(n) => Ok(Json::obj().with("percent", Json::Num(n))),
                None => Err(format!("{at} deposit.percent is a whole number from 1 to 99, not {}", p.text())),
            };
        }
        if let Some(c) = v.get("cents") {
            return match whole(Some(c), 50.0).filter(|n| *n < price) {
                Some(n) => Ok(Json::obj().with("cents", Json::Num(n))),
                None => Err(format!("{at} deposit.cents is a whole number of cents, at least 50 and less than the price ({}), not {}", Json::Num(price).text(), c.text())),
            };
        }
    }
    Err(format!("{at} deposit is {{ percent }} or {{ cents }}, not {}", v.text()))
}

/// One variant as its element declares it, for a good of `kind` (the TypeScript SDK's `variantOf`).
pub fn variant_of(t: &DocTree, kind: &str) -> Result<Json, String> {
    let sku = t.attr_of("sku");
    let Some(Json::Str(sku_text)) = sku.filter(|v| is_text(Some(v), 80)) else {
        return Err(format!("variant(): sku is a short text (at most 80), not {}", said(sku)));
    };
    let at = format!("variant().sku({})", Json::Str(sku_text.clone()).text());
    if !is_text(t.attr_of("name"), 120) {
        return Err(format!("{at} name is a short text (at most 120), not {}", said(t.attr_of("name"))));
    }
    let Some(price) = whole(t.attr_of("priceCents"), 50.0) else {
        return Err(format!("{at} priceCents is a whole number of cents, at least 50, not {}", said(t.attr_of("priceCents"))));
    };
    let mut out = Json::obj();
    out.set("sku", Json::Str(sku_text.clone()));
    out.set("name", t.attr_of("name").cloned().unwrap_or(Json::Null));
    out.set("priceCents", Json::Num(price));
    if let Some(q) = t.attr_of("inventoryQty") {
        let Some(n) = whole(Some(q), -1.0) else {
            return Err(format!("{at} inventoryQty is a whole number (-1: made to order), not {}", q.text()));
        };
        out.set("inventoryQty", Json::Num(n));
    }
    if kind == "digital" {
        if t.attr_of("weightGrams").is_some() {
            return Err(format!("{at} is digital: it has no weightGrams"));
        }
        if t.attr_of("deposit").is_some() {
            return Err(format!("{at} is digital: it takes no deposit (a deposit is a pre-order of a physical good)"));
        }
        if !is_text(t.attr_of("file"), 1000) {
            return Err(format!("{at} is digital: file names the file it sells (a path from this file's folder, or an address), not {}", said(t.attr_of("file"))));
        }
        out.set("file", t.attr_of("file").cloned().unwrap_or(Json::Null));
        return Ok(out);
    }
    if t.attr_of("file").is_some() {
        return Err(format!("{at} is physical: it ships, so it names no file"));
    }
    if let Some(w) = t.attr_of("weightGrams") {
        let Some(n) = whole(Some(w), 0.0) else {
            return Err(format!("{at} weightGrams is a whole number of grams, not {}", w.text()));
        };
        out.set("weightGrams", Json::Num(n));
    }
    if let Some(d) = t.attr_of("deposit") {
        out.set("deposit", deposit_of(d, price, &at)?);
    }
    Ok(out)
}

fn slug_ok(s: &str) -> bool {
    let b = s.as_bytes();
    (2..=80).contains(&b.len())
        && (b[0].is_ascii_lowercase() || b[0].is_ascii_digit())
        && b.iter().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || *c == b'-')
}

/// A good as its tree declares it (the TypeScript SDK's `goodOf`).
pub fn good_of(t: &DocTree) -> Result<Json, String> {
    let name = t.attr_of("name");
    let Some(Json::Str(name_text)) = name.filter(|v| is_text(Some(v), 120)) else {
        return Err(format!("good(): name is a short text (at most 120), not {}", said(name)));
    };
    let at = format!("good().name({})", Json::Str(name_text.clone()).text());
    let kind = match t.attr_of("kind") {
        Some(Json::Str(k)) if GOOD_KINDS.contains(&k.as_str()) => k.clone(),
        other => return Err(format!("{at} kind is {}, not {}", GOOD_KINDS.join(" or "), said(other))),
    };
    let mut out = Json::obj();
    out.set("name", Json::Str(name_text.clone()));
    out.set("kind", Json::Str(kind.clone()));
    if let Some(s) = t.attr_of("slug") {
        if !matches!(s, Json::Str(x) if slug_ok(x)) {
            return Err(format!("{at} slug is lower-case letters, digits and dashes (2 to 80), not {}", s.text()));
        }
        out.set("slug", s.clone());
    }
    if let Some(s) = t.attr_of("subtitle") {
        if !is_text(Some(s), 200) {
            return Err(format!("{at} subtitle is a short text (at most 200), not {}", s.text()));
        }
        out.set("subtitle", s.clone());
    }
    if let Some(d) = t.attr_of("description") {
        if !matches!(d, Json::Str(x) if js_len(x) <= 8000) {
            return Err(format!("{at} description is a text (at most 8000)"));
        }
        out.set("description", d.clone());
    }
    if let Some(i) = t.attr_of("images") {
        let ok = matches!(i, Json::Arr(items) if items.len() <= 20 && items.iter().all(|x| is_text(Some(x), 500)));
        if !ok {
            return Err(format!("{at} images is a list of at most 20 paths or URLs"));
        }
        out.set("images", i.clone());
    }
    if let Some(s) = t.attr_of("status") {
        if !matches!(s, Json::Str(x) if GOOD_STATUSES.contains(&x.as_str())) {
            return Err(format!("{at} status is {}, not {}", GOOD_STATUSES.join(", "), s.text()));
        }
        out.set("status", s.clone());
    }
    if kind == "digital" && t.attr_of("shipping").is_some() {
        return Err(format!("{at} is digital: it ships nothing, so it has no shipping"));
    }
    if kind == "physical" {
        match t.attr_of("shipping") {
            Some(s @ Json::Obj(_)) => {
                out.set("shipping", s.clone());
            }
            other => return Err(format!("{at} is physical: shipping is its shipping policy ({{ zones, from?, parcel? }}), not {}", said(other))),
        }
    }
    let variants: Vec<&DocTree> = t.children.iter().filter(|c| c.tag == "variant").collect();
    // A file not finished yet may hold none; the goods door refuses a good that sells nothing (`variants_required`).
    if variants.len() > 50 {
        return Err(format!("{at} sells at most 50 variant()s, not {}", variants.len()));
    }
    let mut list = Vec::new();
    for v in variants {
        list.push(variant_of(v, &kind)?);
    }
    out.set("variants", Json::Arr(list));
    Ok(out)
}
