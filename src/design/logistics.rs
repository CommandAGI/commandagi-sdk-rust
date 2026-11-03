//! A SUPPLIER AND A LOGISTICS PLAN IN RUST — a `.supplier.rs` and a `.logistics.rs`. The same elements, and the same
//! documents, as the TypeScript SDK's `logistics.ts` and the Python SDK's `logistics.py`:
//!
//! ```
//! use commandagi::design::logistics::*;
//!
//! fn document() -> El {
//!     logistics([
//!         bom().ref_("Tile kit.3d.tsx"),
//!         builder().name("the builder's bench, Austin").lat(30.267).lon(-97.743).country("US"),
//!         route([
//!             leg().carrier("DHL").service("Express pickup").to("DHL hub, Hong Kong").lat(22.308).lon(113.918).transit_days(1).cost(6).currency("USD"),
//!             leg().carrier("DHL").service("Express Worldwide").transit_days(4).cost(32).currency("USD"),
//!         ])
//!         .supplier("suppliers/jlcpcb.supplier.tsx"),
//!     ])
//!     .name("tile kit")
//!     .quantity(10)
//! }
//! # commandagi::design::declare(document()).unwrap();
//! ```
//!
//! A supplier is `supplier([location()…, ships()…, contact()…]).name("JLCPCB")`. A route's legs run in order from the
//! supplier's `location()`; a leg that names a place (`to`, `lat`, `lon`) ends there, and a leg that names none ends
//! at the builder. Refs are by the plan's folder. Nothing adds a default.

use super::documents::{declare_document, DocTree, TagRule, Vocabulary};
use super::element::{attributes, elements, rust_name};
use super::{Declared, Family};
pub use super::{fragment, json, El, Json, Value};

elements! {
    /// A supplier: its location, the carriers it ships with and its contacts.
    supplier: children;
    /// Where the supplier ships from: `.name(…).lat(…).lon(…).country("CN")`.
    location: leaf;
    /// A carrier's service the supplier ships with: `.carrier("DHL").service(…).transit_days(5)`.
    ships: leaf;
    /// A person or a desk to ask: `.role("sales").email(…)`.
    contact: leaf;
    /// A logistics plan: the BOM's designs, the builder and the routes.
    logistics: children;
    /// A design whose roll-up is (part of) the bill of materials: `.ref_("Tile kit.3d.tsx")`.
    bom: leaf;
    /// Where the product is built, as `location()`.
    builder: leaf;
    /// One supplier's way to the builder: `route([leg()…]).supplier("suppliers/x.supplier.tsx")`.
    route: children;
    /// One leg of a route: `.carrier(…).service(…).transit_days(…).cost(…).currency(…)`, and `.to(…).lat(…).lon(…)`
    /// when it ends somewhere before the builder.
    leg: leaf;
}

attributes! {
    /// The fields of a supplier, a plan and their parts, chained on their elements.
    pub trait LogisticsAttrs {
        name; url; lead_time_days; note; lat; lon; country; address;
        carrier; service; transit_days; role; email; phone;
        quantity; ref_ = "ref"; qty; supplier; cost; currency; to;
    }
}

pub(crate) const FAMILY: Family = Family { module: "logistics", roots: &["supplier", "logistics"], declare };

pub const PLACE_FIELDS: &[&str] = &["name", "lat", "lon", "country", "address"];
pub const SUPPLIER_FIELDS: &[&str] = &["name", "url", "leadTimeDays", "note"];
pub const SHIPS_FIELDS: &[&str] = &["carrier", "service", "transitDays", "note"];
pub const CONTACT_FIELDS: &[&str] = &["name", "role", "email", "phone"];
pub const LOGISTICS_FIELDS: &[&str] = &["name", "quantity"];
pub const BOM_FIELDS: &[&str] = &["ref", "qty"];
pub const LEG_FIELDS: &[&str] = &["carrier", "service", "transitDays", "cost", "currency", "to", "lat", "lon", "country"];

const SUPPLIER: Vocabulary = Vocabulary {
    format: "supplier",
    noun: "a supplier",
    root: "supplier",
    tags: &[
        TagRule::new("supplier", &[]).required(&["name"]).attrs(SUPPLIER_FIELDS),
        TagRule::new("location", &["supplier"]).single().required(&["name", "lat", "lon"]).attrs(PLACE_FIELDS),
        TagRule::new("ships", &["supplier"]).required(&["carrier"]).attrs(SHIPS_FIELDS),
        TagRule::new("contact", &["supplier"]).attrs(CONTACT_FIELDS),
    ],
    from_tree: supplier_of,
};

const LOGISTICS: Vocabulary = Vocabulary {
    format: "logistics",
    noun: "a logistics plan",
    root: "logistics",
    tags: &[
        TagRule::new("logistics", &[]).attrs(LOGISTICS_FIELDS),
        TagRule::new("bom", &["logistics"]).required(&["ref"]).attrs(BOM_FIELDS),
        TagRule::new("builder", &["logistics"]).single().required(&["name", "lat", "lon"]).attrs(PLACE_FIELDS),
        TagRule::new("route", &["logistics"]).required(&["supplier"]).attrs(&["supplier"]),
        TagRule::new("leg", &["route"]).required(&["carrier"]).attrs(LEG_FIELDS),
    ],
    from_tree: logistics_of,
};

fn declare(root: El) -> Result<Declared, String> {
    declare_document(&root, if root.tag == "supplier" { &SUPPLIER } else { &LOGISTICS })
}

/// The fields a record says, in `keys`' order: an empty text is not said, as the TypeScript SDK drops it.
fn fields(t: &DocTree, keys: &[&str]) -> Json {
    let mut out = Json::obj();
    for k in keys {
        if let Some(v) = t.attr_of(k).filter(|v| **v != Json::from("")) {
            out.set(k, v.clone());
        }
    }
    out
}

fn text(t: &DocTree, keys: &[&str]) -> Result<(), String> {
    for k in keys {
        if matches!(t.attr_of(k), Some(v) if !matches!(v, Json::Str(_))) {
            return Err(format!("{}(): {} is text", t.tag, rust_name(k)));
        }
    }
    Ok(())
}

fn count(t: &DocTree, keys: &[&str]) -> Result<(), String> {
    for k in keys {
        if matches!(t.attr_of(k), Some(v) if !matches!(v.as_f64(), Some(n) if n.is_finite() && n >= 0.0)) {
            return Err(format!("{}(): {} is a number, 0 or more", t.tag, rust_name(k)));
        }
    }
    Ok(())
}

fn place(t: &DocTree, name: &str) -> Result<Json, String> {
    let label = match t.attr_of(name) {
        Some(Json::Str(s)) if !s.trim().is_empty() => s.clone(),
        _ => return Err(format!("{}(): {} is the place's name", t.tag, rust_name(name))),
    };
    let lat = t.attr_of("lat").and_then(Json::as_f64).filter(|v| (-90.0..=90.0).contains(v));
    let lat = lat.ok_or_else(|| format!("{}(): lat is degrees north, -90 to 90", t.tag))?;
    let lon = t.attr_of("lon").and_then(Json::as_f64).filter(|v| (-180.0..=180.0).contains(v));
    let lon = lon.ok_or_else(|| format!("{}(): lon is degrees east, -180 to 180", t.tag))?;
    let mut out = Json::obj().with("name", label).with("lat", lat).with("lon", lon);
    match t.attr_of("country") {
        None => {}
        Some(Json::Str(c)) if c.len() == 2 && c.chars().all(|ch| ch.is_ascii_uppercase()) => {
            out.set("country", c.as_str());
        }
        _ => return Err(format!(r#"{}(): country is ISO 3166 alpha-2 ("CN")"#, t.tag)),
    }
    text(t, &["address"])?;
    if let Some(a @ Json::Str(s)) = t.attr_of("address") {
        if !s.is_empty() {
            out.set("address", a.clone());
        }
    }
    Ok(out)
}

fn supplier_of(t: &DocTree) -> Result<Json, String> {
    text(t, &["name", "url", "note"])?;
    count(t, &["leadTimeDays"])?;
    let mut out = fields(t, SUPPLIER_FIELDS);
    if let Some(l) = t.single("location") {
        out.set("location", place(l, "name")?);
    }
    let mut ships = Vec::new();
    let mut contacts = Vec::new();
    for c in &t.children {
        if c.tag == "ships" {
            text(c, &["carrier", "service", "note"])?;
            count(c, &["transitDays"])?;
            ships.push(fields(c, SHIPS_FIELDS));
        } else if c.tag == "contact" {
            text(c, CONTACT_FIELDS)?;
            contacts.push(fields(c, CONTACT_FIELDS));
        }
    }
    Ok(out.with("ships", ships).with("contacts", contacts))
}

fn leg_of(c: &DocTree) -> Result<Json, String> {
    text(c, &["carrier", "service", "to"])?;
    count(c, &["transitDays", "cost"])?;
    match c.attr_of("currency") {
        None => {}
        Some(Json::Str(s)) if s.len() == 3 && s.chars().all(|ch| ch.is_ascii_uppercase()) => {}
        _ => return Err(r#"leg(): currency is ISO 4217 ("USD")"#.into()),
    }
    if c.attr_of("cost").is_some() && c.attr_of("currency").is_none() {
        return Err("leg(): cost states its currency".into());
    }
    let mut out = fields(c, &["carrier", "service", "transitDays", "cost", "currency"]);
    if ["to", "lat", "lon", "country"].iter().any(|k| c.attr_of(k).is_some()) {
        out.set("to", place(c, "to")?);
    }
    Ok(out)
}

fn logistics_of(t: &DocTree) -> Result<Json, String> {
    text(t, &["name"])?;
    if matches!(t.attr_of("quantity"), Some(v) if !matches!(v.as_f64(), Some(n) if n >= 1.0 && n.fract() == 0.0)) {
        return Err("logistics(): quantity is a whole number, 1 or more".into());
    }
    let out = fields(t, LOGISTICS_FIELDS);
    let mut boms = Vec::new();
    let mut routes = Vec::new();
    for c in &t.children {
        if c.tag == "bom" {
            text(c, &["ref"])?;
            if matches!(c.attr_of("qty"), Some(v) if !matches!(v.as_f64(), Some(n) if n > 0.0)) {
                return Err("bom(): qty is a number above 0".into());
            }
            boms.push(fields(c, BOM_FIELDS));
        } else if c.tag == "route" {
            text(c, &["supplier"])?;
            let legs = c.children.iter().filter(|l| l.tag == "leg").map(leg_of).collect::<Result<Vec<_>, _>>()?;
            routes.push(fields(c, &["supplier"]).with("legs", legs));
        }
    }
    let mut out = out.with("boms", boms);
    if let Some(b) = t.single("builder") {
        out.set("builder", place(b, "name")?);
    }
    Ok(out.with("routes", routes))
}
