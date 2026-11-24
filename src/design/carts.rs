//! A CART IN RUST — `Carts/<seller>.cart.rs` is a buyer's cart at one seller: one `item` for each product variant, by
//! its `sku`, and one `offer` for each offer that is not a product, by its `listing` id. The same elements, and the same
//! document, as the TypeScript SDK's `carts.ts` (`.cart.tsx`) and the Python SDK's `carts.py`:
//!
//! ```
//! use commandagi::design::carts::*;
//!
//! fn document() -> El {
//!     cart([item().sku("helmet-stl").qty(1), offer().listing("lst_print-farm-hour").qty(3)]).seller("acct:org:helmetworks")
//! }
//! # commandagi::design::declare(document()).unwrap();
//! ```
//!
//! A cart names no price: the checkout prices each line from the seller's product files and listings when the buyer
//! pays.

use super::documents::{declare_document, DocTree, TagRule, Vocabulary};
use super::element::{attributes, elements};
use super::{Declared, Family};
pub use super::{fragment, json, El, Json, Value};

elements! {
    /// A cart: one `item()` or `offer()` for each line.
    cart: children;
    /// A line of a product variant, by its SKU: `.sku("helmet-stl").qty(1)`.
    item: leaf;
    /// A line of an offer that is not a product, by its listing id: `.listing("lst_print-farm-hour").qty(3)`.
    offer: leaf;
}

attributes! {
    /// The fields of a cart and of its lines, chained on their elements.
    pub trait CartsAttrs {
        seller; sku; listing; qty;
    }
}

pub(crate) const FAMILY: Family = Family { module: "carts", roots: &["cart"], declare };

pub const CART_FIELDS: &[&str] = &["seller"];
pub const ITEM_FIELDS: &[&str] = &["sku", "qty"];
pub const OFFER_FIELDS: &[&str] = &["listing", "qty"];

// A line names no price (priceCents, amountCents …): it is not read, so the file cannot hold one.
const CART: Vocabulary = Vocabulary {
    format: "cart",
    noun: "a cart",
    root: "cart",
    tags: &[
        TagRule::new("cart", &[]).required(&["seller"]).attrs(CART_FIELDS),
        TagRule::new("item", &["cart"]).key("sku").required(&["sku", "qty"]).attrs(ITEM_FIELDS),
        TagRule::new("offer", &["cart"]).key("listing").required(&["listing", "qty"]).attrs(OFFER_FIELDS),
    ],
    from_tree: cart_of,
};

fn declare(root: El) -> Result<Declared, String> {
    declare_document(&root, &CART)
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

fn seller_ok(s: &str) -> bool {
    let rest = s.strip_prefix("acct:user:").or_else(|| s.strip_prefix("acct:org:"));
    matches!(rest, Some(r) if (1..=200).contains(&r.encode_utf16().count()) && !r.chars().any(char::is_whitespace))
}

fn qty_of(t: &DocTree, at: &str) -> Result<Json, String> {
    match t.attr_of("qty") {
        Some(Json::Num(n)) if n.fract() == 0.0 && (1.0..=999.0).contains(n) => Ok(Json::Num(*n)),
        other => Err(format!("{at} qty is a whole number from 1 to 999, not {}", said(other))),
    }
}

/// A cart as its tree declares it (the TypeScript SDK's `cartOf`).
pub fn cart_of(t: &DocTree) -> Result<Json, String> {
    let seller = t.attr_of("seller");
    let Some(Json::Str(seller_text)) = seller.filter(|v| matches!(v, Json::Str(s) if seller_ok(s))) else {
        return Err(format!("cart(): seller is an account, acct:user:<id> or acct:org:<id>, not {}", said(seller)));
    };
    let at = format!("cart().seller({})", Json::Str(seller_text.clone()).text());
    let lines: Vec<&DocTree> = t.children.iter().filter(|c| c.tag == "item" || c.tag == "offer").collect();
    if lines.len() > 100 {
        return Err(format!("{at} holds at most 100 lines (item()s and offer()s), not {}", lines.len()));
    }
    let (mut items, mut offers) = (Vec::new(), Vec::new());
    for c in lines {
        if c.tag == "item" {
            let Some(sku) = short(c.attr_of("sku"), 80) else {
                return Err(format!("item(): sku is a short text (at most 80), not {}", said(c.attr_of("sku"))));
            };
            let qty = qty_of(c, &format!("item().sku({})", Json::Str(sku.clone()).text()))?;
            items.push(Json::obj().with("sku", Json::Str(sku)).with("qty", qty));
        } else {
            let Some(listing) = short(c.attr_of("listing"), 200) else {
                return Err(format!("offer(): listing is a short text (at most 200), not {}", said(c.attr_of("listing"))));
            };
            let qty = qty_of(c, &format!("offer().listing({})", Json::Str(listing.clone()).text()))?;
            offers.push(Json::obj().with("listing", Json::Str(listing)).with("qty", qty));
        }
    }
    Ok(Json::obj().with("seller", Json::Str(seller_text.clone())).with("items", Json::Arr(items)).with("offers", Json::Arr(offers)))
}
