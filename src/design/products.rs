//! A PRODUCT IN RUST — a `<name>.product.rs` declares one product a seller sells: its variants (options, SKU, price,
//! stock) and its discounts. The same elements, and the same document, as the TypeScript SDK's `products.ts`
//! (`.product.tsx`) and the Python SDK's `products.py`. Each `variant` is one record of it, by its `sku`, and each
//! `discount` one, by its `code`:
//!
//! ```
//! use commandagi::design::products::*;
//!
//! fn document() -> El {
//!     product([
//!         variant().sku("helmet-stl").name("STL pack").price_cents(1500).file("./helmet.zip"),
//!         discount().code("LAUNCH10").percent(10).ends_at("2026-12-01"),
//!     ])
//!     .name("Helmet files")
//!     .kind("digital")
//!     .status("active")
//! }
//! # commandagi::design::declare(document()).unwrap();
//! ```
//!
//! `kind` is `physical` (`shipping` is its policy, required) or `digital` (each variant names the `file` it sells; no
//! shipping, weight, stock or deposit). A physical variant with no `stock` is made to order. A `deposit`
//! (`{"percent": 1..99}` or `{"cents": n}`, at least 50 and less than the price) makes a variant a pre-order. A
//! `discount` takes `percent` or `amount_cents` off, between `starts_at` and `ends_at` (ISO 8601; a date-time with no
//! offset is UTC), for at most `usage_limit` orders. A run gives back `{format: "product", document, sources}` beside an
//! empty graph; `sources` names each variant's site by its sku (`variant#helmet-stl`).

use super::documents::{declare_document, DocTree, TagRule, Vocabulary};
use super::element::{attributes, elements};
use super::{Declared, Family};
pub use super::{fragment, json, El, Json, Value};

elements! {
    /// A product: one `variant()` and one `discount()` each.
    product: children;
    /// One variant: `.sku("helmet-stl").name("STL pack").price_cents(1500)`.
    variant: leaf;
    /// One discount: `.code("LAUNCH10").percent(10)` or `.amount_cents(500)`.
    discount: leaf;
}

attributes! {
    /// The fields of a product, of its variants and of its discounts, chained on their elements.
    pub trait ProductsAttrs {
        name; kind; slug; subtitle; description; images; status; shipping; fulfilment; quantity_terms; embodiment_kind; tax_code;
        sku; price_cents; compare_at_cents; stock; options; weight_grams; file; deposit;
        code; percent; amount_cents; starts_at; ends_at; usage_limit;
    }
}

pub(crate) const FAMILY: Family = Family { module: "products", roots: &["product"], declare };

pub const PRODUCT_KINDS: &[&str] = &["physical", "digital"];
pub const PRODUCT_STATUSES: &[&str] = &["draft", "active", "archived"];
pub const PRODUCT_FIELDS: &[&str] = &["name", "kind", "slug", "subtitle", "description", "images", "status", "shipping", "fulfilment", "quantityTerms", "embodimentKind", "taxCode"];
pub const VARIANT_FIELDS: &[&str] = &["sku", "name", "priceCents", "compareAtCents", "stock", "options", "weightGrams", "file", "deposit"];
pub const DISCOUNT_FIELDS: &[&str] = &["code", "percent", "amountCents", "startsAt", "endsAt", "usageLimit"];

const PRODUCT: Vocabulary = Vocabulary {
    format: "product",
    noun: "a product",
    root: "product",
    tags: &[
        TagRule::new("product", &[]).required(&["name", "kind"]).attrs(PRODUCT_FIELDS),
        TagRule::new("variant", &["product"]).key("sku").required(&["sku", "name", "priceCents"]).attrs(VARIANT_FIELDS),
        TagRule::new("discount", &["product"]).key("code").required(&["code"]).attrs(DISCOUNT_FIELDS),
    ],
    from_tree: product_of,
};

fn declare(root: El) -> Result<Declared, String> {
    declare_document(&root, &PRODUCT)
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

fn code_ok(s: &str) -> bool {
    (3..=40).contains(&s.len()) && s.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_')
}

/// Days since 1970-01-01 of a proleptic Gregorian date (Howard Hinnant's `days_from_civil`).
fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// The instant an ISO 8601 date or date-time names, in ms (a date-time with no offset is UTC), or none:
/// `YYYY-MM-DD`, then optionally `THH:MM`, `:SS`, `.sss` and `Z` or `±HH:MM`.
fn instant_of(v: &Json) -> Option<i64> {
    let Json::Str(s) = v else { return None };
    let b = s.as_bytes();
    let digits = |from: usize, n: usize| -> Option<i64> {
        let part = b.get(from..from + n)?;
        part.iter().all(u8::is_ascii_digit).then(|| part.iter().fold(0, |acc, c| acc * 10 + i64::from(c - b'0')))
    };
    let at = |i: usize, c: u8| b.get(i) == Some(&c);
    let (y, mo, d) = (digits(0, 4)?, digits(5, 2)?, digits(8, 2)?);
    if !at(4, b'-') || !at(7, b'-') {
        return None;
    }
    let (mut h, mut mi, mut sec, mut ms, mut off) = (0, 0, 0, 0, 0);
    let mut i = 10;
    if i < b.len() {
        if !at(i, b'T') || !at(i + 3, b':') {
            return None;
        }
        h = digits(i + 1, 2)?;
        mi = digits(i + 4, 2)?;
        i += 6;
        if at(i, b':') {
            sec = digits(i + 1, 2)?;
            i += 3;
            if at(i, b'.') {
                let n = b[i + 1..].iter().take_while(|c| c.is_ascii_digit()).count();
                if !(1..=3).contains(&n) {
                    return None;
                }
                ms = digits(i + 1, n)? * 10_i64.pow(3 - n as u32);
                i += 1 + n;
            }
        }
        if at(i, b'Z') {
            i += 1;
        } else if at(i, b'+') || at(i, b'-') {
            if !at(i + 3, b':') {
                return None;
            }
            off = (if at(i, b'-') { -1 } else { 1 }) * (digits(i + 1, 2)? * 60 + digits(i + 4, 2)?);
            i += 6;
        }
        if i != b.len() {
            return None;
        }
    }
    let leap = (y % 4 == 0 && y % 100 != 0) || y % 400 == 0;
    let days = [31, if leap { 29 } else { 28 }, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    if !(1..=12).contains(&mo) || d < 1 || d > days[(mo - 1) as usize] || h > 23 || mi > 59 || sec > 59 {
        return None;
    }
    Some(((days_from_civil(y, mo, d) * 24 + h) * 60 + mi - off) * 60_000 + sec * 1000 + ms)
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

/// One variant as its element declares it, for a product of `kind` (the TypeScript SDK's `variantOf`).
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
    if let Some(c) = t.attr_of("compareAtCents") {
        let Some(n) = whole(Some(c), price + 1.0) else {
            return Err(format!("{at} compareAtCents is a whole number of cents, more than the price ({}), not {}", Json::Num(price).text(), c.text()));
        };
        out.set("compareAtCents", Json::Num(n));
    }
    let stock = t.attr_of("stock");
    if kind == "physical" {
        if let Some(q) = stock {
            let Some(n) = whole(Some(q), 0.0) else {
                return Err(format!("{at} stock is a whole number, at least 0 (no stock: made to order), not {}", q.text()));
            };
            out.set("stock", Json::Num(n));
        }
    }
    if let Some(o) = t.attr_of("options") {
        let ok = matches!(o, Json::Obj(fields) if fields.len() <= 20
            && fields.iter().all(|(k, x)| !k.trim().is_empty() && matches!(x, Json::Str(_) | Json::Num(_) | Json::Bool(_))));
        if !ok {
            return Err(format!("{at} options is an object of at most 20 names, each a text, a number or a boolean, not {}", o.text()));
        }
        out.set("options", o.clone());
    }
    if kind == "digital" {
        if stock.is_some() {
            return Err(format!("{at} is digital: it has no stock (a file does not run out)"));
        }
        if t.attr_of("weightGrams").is_some() {
            return Err(format!("{at} is digital: it has no weightGrams"));
        }
        if t.attr_of("deposit").is_some() {
            return Err(format!("{at} is digital: it takes no deposit (a deposit is a pre-order of a physical product)"));
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

/// One discount as its element declares it (the TypeScript SDK's `discountOf`).
pub fn discount_of(t: &DocTree) -> Result<Json, String> {
    let code = t.attr_of("code");
    let Some(Json::Str(code_text)) = code.filter(|v| matches!(v, Json::Str(s) if code_ok(s))) else {
        return Err(format!("discount(): code is letters, digits, - and _ (3 to 40), not {}", said(code)));
    };
    let at = format!("discount().code({})", Json::Str(code_text.clone()).text());
    let mut out = Json::obj();
    out.set("code", Json::Str(code_text.clone()));
    let (percent, amount) = (t.attr_of("percent"), t.attr_of("amountCents"));
    if percent.is_some() == amount.is_some() {
        return Err(format!("{at} takes off one of percent or amountCents, not {}", if percent.is_some() { "both" } else { "neither" }));
    }
    if let Some(p) = percent {
        let Some(n) = whole(Some(p), 1.0).filter(|n| *n <= 100.0) else {
            return Err(format!("{at} percent is a whole number from 1 to 100, not {}", p.text()));
        };
        out.set("percent", Json::Num(n));
    }
    if let Some(a) = amount {
        let Some(n) = whole(Some(a), 1.0) else {
            return Err(format!("{at} amountCents is a whole number of cents, at least 1, not {}", a.text()));
        };
        out.set("amountCents", Json::Num(n));
    }
    let mut start = None;
    if let Some(s) = t.attr_of("startsAt") {
        start = instant_of(s);
        if start.is_none() {
            return Err(format!("{at} startsAt is an ISO 8601 date or date-time (\"2026-11-08\", \"2026-11-08T09:00:00Z\"), not {}", s.text()));
        }
        out.set("startsAt", s.clone());
    }
    if let Some(e) = t.attr_of("endsAt") {
        let Some(end) = instant_of(e) else {
            return Err(format!("{at} endsAt is an ISO 8601 date or date-time (\"2026-11-08\", \"2026-11-08T09:00:00Z\"), not {}", e.text()));
        };
        if start.is_some_and(|s| end <= s) {
            let plain = |v: &Json| v.as_str().unwrap_or_default().to_string();
            return Err(format!("{at} endsAt ({}) is after startsAt ({})", plain(e), t.attr_of("startsAt").map(plain).unwrap_or_default()));
        }
        out.set("endsAt", e.clone());
    }
    if let Some(u) = t.attr_of("usageLimit") {
        let Some(n) = whole(Some(u), 1.0) else {
            return Err(format!("{at} usageLimit is a whole number, at least 1, not {}", u.text()));
        };
        out.set("usageLimit", Json::Num(n));
    }
    Ok(out)
}

fn slug_ok(s: &str) -> bool {
    let b = s.as_bytes();
    (2..=80).contains(&b.len())
        && (b[0].is_ascii_lowercase() || b[0].is_ascii_digit())
        && b.iter().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || *c == b'-')
}

/// A product as its tree declares it (the TypeScript SDK's `productOf`).
pub fn product_of(t: &DocTree) -> Result<Json, String> {
    let name = t.attr_of("name");
    let Some(Json::Str(name_text)) = name.filter(|v| is_text(Some(v), 120)) else {
        return Err(format!("product(): name is a short text (at most 120), not {}", said(name)));
    };
    let at = format!("product().name({})", Json::Str(name_text.clone()).text());
    let kind = match t.attr_of("kind") {
        Some(Json::Str(k)) if PRODUCT_KINDS.contains(&k.as_str()) => k.clone(),
        other => return Err(format!("{at} kind is {}, not {}", PRODUCT_KINDS.join(" or "), said(other))),
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
        if !matches!(s, Json::Str(x) if PRODUCT_STATUSES.contains(&x.as_str())) {
            return Err(format!("{at} status is {}, not {}", PRODUCT_STATUSES.join(", "), s.text()));
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
    if let Some(f) = t.attr_of("fulfilment") {
        if !matches!(f, Json::Obj(_)) {
            return Err(format!("{at} fulfilment is an object (how a paid line is fulfilled), not {}", f.text()));
        }
        out.set("fulfilment", f.clone());
    }
    if let Some(q) = t.attr_of("quantityTerms") {
        if !matches!(q, Json::Obj(_)) {
            return Err(format!("{at} quantityTerms is an object, not {}", q.text()));
        }
        out.set("quantityTerms", q.clone());
    }
    if let Some(e) = t.attr_of("embodimentKind") {
        if !is_text(Some(e), 80) {
            return Err(format!("{at} embodimentKind is a short text (at most 80), not {}", e.text()));
        }
        out.set("embodimentKind", e.clone());
    }
    if let Some(c) = t.attr_of("taxCode") {
        let ok = matches!(c, Json::Str(s) if s.len() == 13 && s.starts_with("txcd_") && s[5..].bytes().all(|b| b.is_ascii_digit()));
        if !ok {
            return Err(format!("{at} taxCode is a Stripe product tax code like txcd_99999999, not {}", c.text()));
        }
        out.set("taxCode", c.clone());
    }
    let variants: Vec<&DocTree> = t.children.iter().filter(|c| c.tag == "variant").collect();
    // A file not finished yet may hold none; the checkout refuses a product that sells nothing.
    if variants.len() > 50 {
        return Err(format!("{at} sells at most 50 variant()s, not {}", variants.len()));
    }
    let mut list = Vec::new();
    for v in variants {
        list.push(variant_of(v, &kind)?);
    }
    out.set("variants", Json::Arr(list));
    let discounts: Vec<&DocTree> = t.children.iter().filter(|c| c.tag == "discount").collect();
    if discounts.len() > 50 {
        return Err(format!("{at} has at most 50 discount()s, not {}", discounts.len()));
    }
    let mut list = Vec::new();
    // A buyer types a code in any case, so two codes that differ only in case are one code.
    let mut codes: Vec<(String, String)> = Vec::new();
    for d in discounts {
        let one = discount_of(d)?;
        let code = one.get("code").and_then(Json::as_str).unwrap_or_default().to_string();
        if let Some((_, was)) = codes.iter().find(|(lower, _)| *lower == code.to_lowercase()) {
            return Err(format!("{at} has two discount()s with one code: \"{was}\" and \"{code}\" (a code is read in any case)"));
        }
        codes.push((code.to_lowercase(), code));
        list.push(one);
    }
    out.set("discounts", Json::Arr(list));
    Ok(out)
}
