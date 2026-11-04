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
//!
//! A leg that says `.quote("cheapest")` (or `fastest`, a carrier `usps`, a carrier and its service code
//! `usps/PRIORITY_MAIL`) with its parcel `.kg(2.5).mm([300, 250, 220])` is quoted live by the shipping interface,
//! beside its stated cost (docs/shipping.md); a place's `.postal("94112")` is the postal code the carriers rate by.
//!
//! A supplier's `service([fee()…]).id("global-sourcing")` is a way to buy through it (pre-order, global sourcing,
//! consignment) with its review and buying days and its fees (per order, line or unit). A plan's `.build("2027-06-01")`
//! is the day the parts must be at the builder, and `source([use_().offer("LCSC")…]).part(…).strategy("split")` says
//! how one line is filled (single, fill, split, dual).

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
    /// A way to buy through the supplier: `service([fee()…]).id("global-sourcing").review_days(1).lead_time_days(9)`.
    service: children;
    /// What a service costs: `.amount(20).currency("USD").per("line")` (per order, line or unit, in groups of `each`).
    fee: leaf;
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
    /// How one line is filled: `source([use_()…]).part("KH-205580-DZ").strategy("split")`.
    source: children;
    /// One offer a line is filled from: `.offer("LCSC")`, `.mpn(…)` for an alternate's, `.qty(…)` or `.share(…)`.
    use_ = "use": leaf;
}

attributes! {
    /// The fields of a supplier, a plan and their parts, chained on their elements.
    pub trait LogisticsAttrs {
        name; url; lead_time_days; note; lat; lon; country; postal; address;
        carrier; service; transit_days; role; email; phone;
        quantity; ref_ = "ref"; qty; supplier; cost; currency; to;
        id; review_days; review_days_high; lead_time_days_high; amount; per; each;
        build; part; strategy; offer; mpn; share; quote; kg; mm;
    }
}

pub(crate) const FAMILY: Family = Family { module: "logistics", roots: &["supplier", "logistics"], declare };

pub const PLACE_FIELDS: &[&str] = &["name", "lat", "lon", "country", "postal", "address"];
pub const SUPPLIER_FIELDS: &[&str] = &["name", "url", "leadTimeDays", "note"];
pub const SHIPS_FIELDS: &[&str] = &["carrier", "service", "transitDays", "note"];
pub const CONTACT_FIELDS: &[&str] = &["name", "role", "email", "phone"];
pub const SERVICE_FIELDS: &[&str] = &["id", "name", "reviewDays", "reviewDaysHigh", "leadTimeDays", "leadTimeDaysHigh", "note"];
pub const FEE_FIELDS: &[&str] = &["amount", "currency", "per", "each", "note"];
pub const LOGISTICS_FIELDS: &[&str] = &["name", "quantity", "build"];
pub const SOURCE_FIELDS: &[&str] = &["part", "strategy"];
pub const USE_FIELDS: &[&str] = &["offer", "mpn", "qty", "share"];
const FEE_PER: &[&str] = &["order", "line", "unit"];
const STRATEGIES: &[&str] = &["single", "fill", "split", "dual"];
pub const BOM_FIELDS: &[&str] = &["ref", "qty"];
pub const LEG_FIELDS: &[&str] = &["carrier", "service", "transitDays", "cost", "currency", "quote", "kg", "mm", "to", "lat", "lon", "country", "postal"];

const SUPPLIER: Vocabulary = Vocabulary {
    format: "supplier",
    noun: "a supplier",
    root: "supplier",
    tags: &[
        TagRule::new("supplier", &[]).required(&["name"]).attrs(SUPPLIER_FIELDS),
        TagRule::new("location", &["supplier"]).single().required(&["name", "lat", "lon"]).attrs(PLACE_FIELDS),
        TagRule::new("ships", &["supplier"]).required(&["carrier"]).attrs(SHIPS_FIELDS),
        TagRule::new("contact", &["supplier"]).attrs(CONTACT_FIELDS),
        TagRule::new("service", &["supplier"]).required(&["id"]).attrs(SERVICE_FIELDS),
        TagRule::new("fee", &["service"]).required(&["amount", "currency", "per"]).attrs(FEE_FIELDS),
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
        TagRule::new("source", &["logistics"]).required(&["part", "strategy"]).attrs(SOURCE_FIELDS),
        TagRule::new("use", &["source"]).required(&["offer"]).attrs(USE_FIELDS),
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
    text(t, &["postal", "address"])?;
    for k in ["postal", "address"] {
        if let Some(a @ Json::Str(s)) = t.attr_of(k) {
            if !s.is_empty() {
                out.set(k, a.clone());
            }
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
    let mut services = Vec::new();
    for c in &t.children {
        if c.tag == "ships" {
            text(c, &["carrier", "service", "note"])?;
            count(c, &["transitDays"])?;
            ships.push(fields(c, SHIPS_FIELDS));
        } else if c.tag == "contact" {
            text(c, CONTACT_FIELDS)?;
            contacts.push(fields(c, CONTACT_FIELDS));
        } else if c.tag == "service" {
            services.push(service_of(c)?);
        }
    }
    Ok(out.with("ships", ships).with("contacts", contacts).with("services", services))
}

fn currency(t: &DocTree) -> Result<(), String> {
    match t.attr_of("currency") {
        None => Ok(()),
        Some(Json::Str(s)) if s.len() == 3 && s.chars().all(|ch| ch.is_ascii_uppercase()) => Ok(()),
        _ => Err(format!(r#"{}(): currency is ISO 4217 ("USD")"#, t.tag)),
    }
}

fn service_of(t: &DocTree) -> Result<Json, String> {
    text(t, &["id", "name", "note"])?;
    count(t, &["reviewDays", "reviewDaysHigh", "leadTimeDays", "leadTimeDaysHigh"])?;
    let mut fees = Vec::new();
    for f in t.children.iter().filter(|f| f.tag == "fee") {
        text(f, &["note"])?;
        if !matches!(f.attr_of("amount").and_then(Json::as_f64), Some(n) if n.is_finite() && n >= 0.0) {
            return Err("fee(): amount is a number, 0 or more".into());
        }
        if f.attr_of("currency").is_none() {
            return Err("fee(): states its currency".into());
        }
        currency(f)?;
        if !matches!(f.attr_of("per"), Some(Json::Str(p)) if FEE_PER.contains(&p.as_str())) {
            return Err("fee(): per is order, line or unit".into());
        }
        if matches!(f.attr_of("each"), Some(v) if !matches!(v.as_f64(), Some(n) if n >= 1.0 && n.fract() == 0.0)) {
            return Err("fee(): each is a whole number, 1 or more".into());
        }
        fees.push(fields(f, FEE_FIELDS));
    }
    Ok(fields(t, SERVICE_FIELDS).with("fees", fees))
}

fn source_of(t: &DocTree) -> Result<Json, String> {
    text(t, &["part"])?;
    if !matches!(t.attr_of("strategy"), Some(Json::Str(s)) if STRATEGIES.contains(&s.as_str())) {
        return Err("source(): strategy is single, fill, split or dual".into());
    }
    let mut uses = Vec::new();
    for u in t.children.iter().filter(|u| u.tag == "use") {
        text(u, &["offer", "mpn"])?;
        if matches!(u.attr_of("qty"), Some(v) if !matches!(v.as_f64(), Some(n) if n > 0.0)) {
            return Err("use_(): qty is a number above 0".into());
        }
        if matches!(u.attr_of("share"), Some(v) if !matches!(v.as_f64(), Some(n) if n > 0.0 && n <= 1.0)) {
            return Err("use_(): share is a number above 0, at most 1".into());
        }
        uses.push(fields(u, USE_FIELDS));
    }
    Ok(fields(t, SOURCE_FIELDS).with("uses", uses))
}

fn leg_of(c: &DocTree) -> Result<Json, String> {
    text(c, &["carrier", "service", "to"])?;
    count(c, &["transitDays", "cost"])?;
    currency(c)?;
    if c.attr_of("cost").is_some() && c.attr_of("currency").is_none() {
        return Err("leg(): cost states its currency".into());
    }
    if matches!(c.attr_of("quote"), Some(v) if !matches!(v, Json::Str(w) if is_want(w))) {
        return Err("leg(): quote is cheapest, fastest, a carrier (usps), or a carrier and its service code (usps/PRIORITY_MAIL)".into());
    }
    if matches!(c.attr_of("kg"), Some(v) if !matches!(v.as_f64(), Some(n) if n.is_finite() && n > 0.0)) {
        return Err("leg(): kg is the parcel's weight in kilograms, above 0".into());
    }
    if matches!(c.attr_of("mm"), Some(v) if !matches!(v, Json::Arr(xs) if xs.len() == 3 && xs.iter().all(|x| matches!(x.as_f64(), Some(n) if n.is_finite() && n > 0.0)))) {
        return Err("leg(): mm is the parcel's [length, width, height] in millimetres".into());
    }
    if c.attr_of("quote").is_some() && (c.attr_of("kg").is_none() || c.attr_of("mm").is_none()) {
        return Err("leg(): quote needs its parcel: kg and mm".into());
    }
    let mut out = fields(c, &["carrier", "service", "transitDays", "cost", "currency", "quote", "kg", "mm"]);
    if ["to", "lat", "lon", "country"].iter().any(|k| c.attr_of(k).is_some()) {
        out.set("to", place(c, "to")?);
    }
    Ok(out)
}

/// What a leg's quote asks for, by its grammar: `cheapest`, `fastest`, a carrier, or a carrier and its service code.
fn is_want(w: &str) -> bool {
    if w == "cheapest" || w == "fastest" {
        return true;
    }
    let (carrier, service) = match w.split_once('/') {
        Some((c, s)) => (c, Some(s)),
        None => (w, None),
    };
    let carrier_ok = (2..=32).contains(&carrier.len()) && carrier.starts_with(|c: char| c.is_ascii_lowercase()) && carrier.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
    let service_ok = service.map_or(true, |s| (1..=64).contains(&s.len()) && s.chars().all(|c| c.is_ascii_alphanumeric() || "_.-".contains(c)));
    carrier_ok && service_ok
}

fn logistics_of(t: &DocTree) -> Result<Json, String> {
    text(t, &["name", "build"])?;
    if matches!(t.attr_of("build"), Some(Json::Str(b)) if !(b.len() == 10 && b.chars().enumerate().all(|(i, c)| if i == 4 || i == 7 { c == '-' } else { c.is_ascii_digit() }))) {
        return Err("logistics(): build is YYYY-MM-DD".into());
    }
    if matches!(t.attr_of("quantity"), Some(v) if !matches!(v.as_f64(), Some(n) if n >= 1.0 && n.fract() == 0.0)) {
        return Err("logistics(): quantity is a whole number, 1 or more".into());
    }
    let out = fields(t, LOGISTICS_FIELDS);
    let mut boms = Vec::new();
    let mut routes = Vec::new();
    let mut sources = Vec::new();
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
        } else if c.tag == "source" {
            sources.push(source_of(c)?);
        }
    }
    let mut out = out.with("boms", boms);
    if let Some(b) = t.single("builder") {
        out.set("builder", place(b, "name")?);
    }
    Ok(out.with("routes", routes).with("sources", sources))
}
