//! A PART'S FIELDS: what a part states beyond its value, each one KiCad `property` form on the part
//! (`foreign`, the form the board editor writes), so the BOM, the KiCad board and a 3D body's join read one thing. The
//! TypeScript SDK's `part-fields.ts`, form for form:
//!
//! ```text
//! .part_number(…).manufacturer(…).mpn(…).rating(…).attrs(json(…))   its engineering data (CommandAGI IPN,
//!                                                                    Manufacturer, MPN, Rating, attrs by name)
//! .draw("120 mA")                                                    its steady draw: the number with the unit it
//!                                                                    measures ("1.2 W")
//! .pinout_verified(…).footprint_verified(…)                         how its pinout and its pads are known ("memory",
//!                                                                    "datasheet <url> p. <page>, <YYYY-MM-DD>",
//!                                                                    "measured <YYYY-MM-DD>[, <how>]")
//! .package_dimensions(…)                                             its datasheet package ("<n> pins, pitch <mm> mm,
//!                                                                    body <l> x <w> mm")
//! .suppliers(json(r#"[{"name", "sku", "url", "currency", "priceBreaks", "moq", "orderMultiple", "leadTimeDays",
//!                      "stock", "source", "date"}]"#))
//! .alternates(json(r#"[{"manufacturer", "mpn", "note"}]"#))  .datasheet("https://…")
//! .lifecycle(json(r#"{"status", "lastTimeBuy", "onHand", "reorderPoint", "source", "date"}"#))
//! .compliance(json(r#"{"rohs", "rohsExemption", "reach", "conflictMinerals", "eccn", "certifications"}"#))
//! .shipping(json(r#"{"hsCode", "origin", "hazmat", "packaging"}"#))   its logistics (docs/formats.md § logistics)
//! ```
//!
//! A supplier is two properties: `<name> Part` (its sku; `LCSC Part` is what JLCPCB's assembly BOM reads) and
//! `<name> Terms` (the rest as compact JSON, a price break as `[quantity, unitPrice]`). `Datasheet` is KiCad's own
//! field; `Alternates` is JSON; a facet's fields are one property each (`Lifecycle`, `RoHS`, `HS Code` …). The
//! repository's decoder is `packages/domain/world/logistics.js`; the TypeScript and Python SDKs write and read the
//! same forms.
//!
//! `datasheet` says where the part's datasheet is (KiCad's `Datasheet` field); it claims nothing. `pinout_verified` and
//! `footprint_verified` claim that a page of a datasheet (or a measurement) was checked against the symbol or the
//! footprint, on a date: the pinout and footprint checks read them and `package_dimensions`.

use super::element::{rust_name, El};
use super::json::Json;

/// The text fields named by an argument, by the KiCad property each writes.
pub const PART_FIELDS: &[(&str, &str)] = &[
    ("partNumber", "CommandAGI IPN"),
    ("manufacturer", "Manufacturer"),
    ("mpn", "MPN"),
    ("rating", "Rating"),
    ("draw", "Draw"),
    ("pinoutVerified", "Pinout Verified"),
    ("footprintVerified", "Footprint Verified"),
    ("packageDimensions", "Package Dimensions"),
];
const FIELD_EXAMPLES: &[&str] = &[
    "MF-R-3K",
    "Yageo",
    "RC0805FR-073KL",
    "0.125 W",
    "120 mA",
    "datasheet https://www.ti.com/lit/ds/symlink/sn74lvc1t45.pdf p. 3, 2026-10-15",
    "measured 2026-10-15, calipers on the reel's part",
    "6 pins, pitch 0.95 mm, body 2.9 x 1.6 mm",
];

fn is_date(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() == 10 && b[4] == b'-' && b[7] == b'-' && b.iter().enumerate().all(|(i, c)| i == 4 || i == 7 || c.is_ascii_digit())
}
fn is_number(s: &str) -> bool {
    let mut parts = s.splitn(2, '.');
    let whole = parts.next().unwrap_or("");
    !whole.is_empty() && whole.bytes().all(|c| c.is_ascii_digit()) && parts.next().is_none_or(|f| !f.is_empty() && f.bytes().all(|c| c.is_ascii_digit()))
}
/// How a pinout or a footprint is known (the TypeScript SDK's `VERIFIED`).
fn is_verified(v: &str) -> bool {
    if v == "memory" {
        return true;
    }
    if let Some(rest) = v.strip_prefix("measured ") {
        let (date, how) = rest.split_once(", ").map_or((rest, None), |(d, h)| (d, Some(h)));
        return is_date(date) && how.is_none_or(|h| !h.is_empty());
    }
    let Some(rest) = v.strip_prefix("datasheet ") else { return false };
    let Some((url, rest)) = rest.split_once(" p. ") else { return false };
    let Some((page, date)) = rest.split_once(", ") else { return false };
    (url.starts_with("https://") || url.starts_with("http://")) && !url.contains(char::is_whitespace) && !page.is_empty() && page.bytes().all(|c| c.is_ascii_digit()) && is_date(date)
}
/// A datasheet's package numbers (the TypeScript SDK's `PACKAGE`).
fn is_package(v: &str) -> bool {
    let Some((pins, rest)) = v.split_once(" pins, pitch ") else { return false };
    let Some((pitch, rest)) = rest.split_once(" mm, body ") else { return false };
    let Some((l, rest)) = rest.split_once(" x ") else { return false };
    let Some(w) = rest.strip_suffix(" mm") else { return false };
    !pins.is_empty() && pins.bytes().all(|c| c.is_ascii_digit()) && is_number(pitch) && is_number(l) && is_number(w)
}
/// A part's steady draw: a number with the unit it measures, a current or a power (the TypeScript SDK's `DRAW`).
fn is_draw(v: &str) -> bool {
    let Some((number, unit)) = v.split_once(' ') else { return false };
    is_number(number) && matches!(unit, "A" | "mA" | "uA" | "\u{b5}A" | "W" | "mW" | "uW" | "\u{b5}W")
}
fn in_grammar(prop: &str, v: &str) -> bool {
    match prop {
        "draw" => is_draw(v),
        "pinoutVerified" | "footprintVerified" => is_verified(v),
        "packageDimensions" => is_package(v),
        _ => true,
    }
}

/// The logistics facets that are objects of fields: argument, then field → KiCad property, in the order written.
pub const FACET_FIELDS: &[(&str, &[(&str, &str)])] = &[
    (
        "lifecycle",
        &[("status", "Lifecycle"), ("lastTimeBuy", "Last Time Buy"), ("onHand", "On Hand"), ("reorderPoint", "Reorder Point"), ("source", "Lifecycle Source"), ("date", "Lifecycle Date")],
    ),
    (
        "compliance",
        &[("rohs", "RoHS"), ("rohsExemption", "RoHS Exemption"), ("reach", "REACH"), ("conflictMinerals", "Conflict Minerals"), ("eccn", "ECCN"), ("certifications", "Certifications")],
    ),
    ("shipping", &[("hsCode", "HS Code"), ("origin", "Country of Origin"), ("hazmat", "Hazmat"), ("packaging", "Packaging")]),
];
const CHOICES: &[(&str, &[&str])] = &[
    ("lifecycle.status", &["active", "nrnd", "eol", "obsolete"]),
    ("compliance.rohs", &["compliant", "exempt", "non-compliant"]),
    ("compliance.reach", &["compliant", "svhc", "non-compliant"]),
    ("compliance.conflictMinerals", &["none", "reported", "unknown"]),
];
const COUNTS: &[&str] = &["lifecycle.onHand", "lifecycle.reorderPoint"];
const DATES: &[&str] = &["lifecycle.lastTimeBuy", "lifecycle.date"];
const OFFER_TERMS: &[&str] = &["currency", "priceBreaks", "moq", "orderMultiple", "leadTimeDays", "stock", "url", "source", "date"];
/// The arguments a part's fields take (TypeScript names), in the order their forms are written.
pub const FIELD_ARGUMENTS: &[&str] = &["partNumber", "manufacturer", "mpn", "rating", "draw", "pinoutVerified", "footprintVerified", "packageDimensions", "attrs", "suppliers", "alternates", "datasheet", "lifecycle", "compliance", "shipping"];
/// Fields a part holds otherwise: its reference, its value, its footprint.
const OWN_FIELDS: &[&str] = &["Reference", "Value", "Footprint"];

/// One field as the part carries it: `(property "MPN" "…" (at 0 0 0) (effects (font (size 1.27 1.27)) (hide yes)))`.
pub fn field_form(name: &str, value: &str) -> Json {
    let atom = |v: &str, q: bool| Json::Obj(vec![("kind".into(), "atom".into()), ("v".into(), v.into()), ("q".into(), q.into())]);
    let sym = |v: &str| atom(v, false);
    let list = |items: Vec<Json>| Json::Obj(vec![("kind".into(), "list".into()), ("items".into(), Json::Arr(items))]);
    list(vec![
        sym("property"),
        atom(name, true),
        atom(value, true),
        list(vec![sym("at"), sym("0"), sym("0"), sym("0")]),
        list(vec![sym("effects"), list(vec![sym("font"), list(vec![sym("size"), sym("1.27"), sym("1.27")])]), list(vec![sym("hide"), sym("yes")])]),
    ])
}

fn is_text(v: Option<&Json>) -> bool {
    matches!(v, Some(Json::Str(s)) if !s.trim().is_empty())
}

fn whole(v: Option<&Json>, min: f64) -> bool {
    matches!(v, Some(Json::Num(n)) if n.fract() == 0.0 && *n >= min)
}

fn code(v: Option<&Json>, len: usize) -> bool {
    matches!(v, Some(Json::Str(s)) if s.len() == len && s.bytes().all(|b| b.is_ascii_uppercase()))
}

fn date(v: Option<&Json>) -> bool {
    matches!(v, Some(Json::Str(s)) if s.len() == 10 && s.bytes().enumerate().all(|(i, c)| if i == 4 || i == 7 { c == b'-' } else { c.is_ascii_digit() }))
}

fn entries(v: &Json) -> &[(String, Json)] {
    match v {
        Json::Obj(e) => e,
        _ => &[],
    }
}

/// The argument that says a KiCad property, or none when `attrs` may hold it.
fn argument_of(name: &str) -> Option<&'static str> {
    if let Some((arg, _)) = PART_FIELDS.iter().find(|(_, n)| *n == name) {
        return Some(arg);
    }
    match name {
        "Datasheet" => return Some("datasheet"),
        "Alternates" => return Some("alternates"),
        _ => {}
    }
    if let Some((arg, _)) = FACET_FIELDS.iter().find(|(_, fields)| fields.iter().any(|(_, p)| *p == name)) {
        return Some(arg);
    }
    let supplier = |suffix: &str| name.strip_suffix(suffix).is_some_and(|head| !head.is_empty());
    (supplier(" Part") || supplier(" Terms")).then_some("suppliers")
}

/// Suppliers' offers, each term checked; a wrong one is refused by name. A board's `fab().suppliers(…)` reads them too.
pub(crate) fn check_offers(at: &str, v: &Json) -> Result<Json, String> {
    let list = match v {
        Json::Arr(l) if !l.is_empty() => l,
        _ => return Err(format!(r#"{at}: suppliers is a list of offers ([{{ name: "LCSC", sku: "C25744", … }}])"#)),
    };
    let mut names: Vec<&str> = Vec::new();
    for (i, s) in list.iter().enumerate() {
        let at = format!("{at}: suppliers[{i}]");
        if !matches!(s, Json::Obj(_)) {
            return Err(format!("{at} is an offer ({{ name, sku, currency, priceBreaks, … }})"));
        }
        for (k, _) in entries(s) {
            if k != "name" && k != "sku" && !OFFER_TERMS.contains(&k.as_str()) {
                return Err(format!("{at}: {k} is not a term of an offer (it takes name, sku, {})", OFFER_TERMS.join(", ")));
            }
        }
        let name = match s.get("name") {
            Some(Json::Str(n)) if !n.trim().is_empty() && !n.contains(['{', '}', '"']) => n.as_str(),
            _ => return Err(format!(r#"{at}: name is the supplier's name ("LCSC")"#)),
        };
        if names.contains(&name) {
            return Err(format!("{at}: two offers of {name}"));
        }
        names.push(name);
        if s.get("sku").is_some() && !is_text(s.get("sku")) {
            return Err(format!("{at}: sku is the supplier's part number, text"));
        }
        for k in ["url", "source"] {
            if s.get(k).is_some() && !is_text(s.get(k)) {
                return Err(format!("{at}: {k} is text"));
            }
        }
        if s.get("currency").is_some() && !code(s.get("currency"), 3) {
            return Err(format!(r#"{at}: currency is an ISO 4217 code ("USD")"#));
        }
        if s.get("date").is_some() && !date(s.get("date")) {
            return Err(format!("{at}: date is YYYY-MM-DD"));
        }
        for k in ["moq", "orderMultiple"] {
            if s.get(k).is_some() && !whole(s.get(k), 1.0) {
                return Err(format!("{at}: {k} is a whole number, 1 or more"));
            }
        }
        if s.get("stock").is_some() && !whole(s.get("stock"), 0.0) {
            return Err(format!("{at}: stock is a whole number"));
        }
        if s.get("leadTimeDays").is_some() && !matches!(s.get("leadTimeDays"), Some(Json::Num(n)) if *n >= 0.0) {
            return Err(format!("{at}: leadTimeDays is a number of days"));
        }
        if let Some(b) = s.get("priceBreaks") {
            let ok = matches!(b, Json::Arr(l) if l.iter().all(|b| entries(b).len() == 2 && whole(b.get("quantity"), 1.0) && matches!(b.get("unitPrice"), Some(Json::Num(p)) if *p >= 0.0)));
            if !ok {
                return Err(format!("{at}: priceBreaks is a list of {{ quantity, unitPrice }} (quantity a whole number from 1, unitPrice 0 or more)"));
            }
        }
    }
    Ok(v.clone())
}

/// One facet's fields (`lifecycle`, `compliance`, `shipping`), each checked; a board's `fab()` reads them too.
pub(crate) fn check_facet(at: &str, arg: &str, v: &Json) -> Result<Json, String> {
    let fields = FACET_FIELDS.iter().find(|(a, _)| *a == arg).map(|(_, f)| *f).unwrap_or_default();
    let names: Vec<&str> = fields.iter().map(|(k, _)| *k).collect();
    if !matches!(v, Json::Obj(_)) {
        return Err(format!("{at}: {arg} is an object ({{ {} }})", names.join(", ")));
    }
    for (k, _) in entries(v) {
        if !names.contains(&k.as_str()) {
            return Err(format!("{at}: {arg}.{k} is not read (it takes {})", names.join(", ")));
        }
    }
    if arg == "lifecycle" && v.get("status").is_none() {
        return Err(format!("{at}: lifecycle needs its status ({})", CHOICES[0].1.join(", ")));
    }
    for k in &names {
        let Some(x) = v.get(k) else { continue };
        let key = format!("{arg}.{k}");
        if COUNTS.contains(&key.as_str()) {
            if !whole(Some(x), 0.0) {
                return Err(format!("{at}: {key} is a whole number"));
            }
        } else if *k == "certifications" {
            if !matches!(x, Json::Arr(l) if l.iter().all(|c| is_text(Some(c)) && !c.as_str().unwrap_or_default().contains(','))) {
                return Err(format!("{at}: {key} is a list of text, each with no comma"));
            }
        } else {
            if !is_text(Some(x)) {
                return Err(format!("{at}: {key} is text"));
            }
            let s = x.as_str().unwrap_or_default();
            if let Some((_, choices)) = CHOICES.iter().find(|(c, _)| *c == key) {
                if !choices.contains(&s) {
                    return Err(format!("{at}: {key} is one of {} (got \"{s}\")", choices.join(", ")));
                }
            }
            if DATES.contains(&key.as_str()) && !date(Some(x)) {
                return Err(format!("{at}: {key} is YYYY-MM-DD"));
            }
            if key == "shipping.origin" && !code(Some(x), 2) && x.as_str() != Some("unknown") {
                return Err(format!(r#"{at}: {key} is an ISO 3166 alpha-2 code ("CN"), or "unknown""#));
            }
        }
    }
    Ok(v.clone())
}

fn offer_forms(at: &str, v: &Json) -> Result<Vec<Json>, String> {
    let mut forms = Vec::new();
    if let Json::Arr(offers) = check_offers(at, v)? {
        for s in &offers {
            let name = s.get("name").and_then(Json::as_str).unwrap_or_default();
            let mut terms = Json::obj();
            for k in OFFER_TERMS {
                let Some(t) = s.get(k) else { continue };
                let t = match (*k, t) {
                    ("priceBreaks", Json::Arr(breaks)) => {
                        Json::Arr(breaks.iter().map(|b| Json::Arr(vec![b.get("quantity").cloned().unwrap_or(Json::Null), b.get("unitPrice").cloned().unwrap_or(Json::Null)])).collect())
                    }
                    _ => t.clone(),
                };
                terms.set(k, t);
            }
            let sku = s.get("sku").and_then(Json::as_str);
            if let Some(sku) = sku {
                forms.push(field_form(&format!("{name} Part"), sku));
            }
            if !entries(&terms).is_empty() || sku.is_none() {
                forms.push(field_form(&format!("{name} Terms"), &terms.text()));
            }
        }
    }
    Ok(forms)
}

/// A facet's value as its property says it: a list joined by ", ", a number as JSON writes it, a text as it is.
fn facet_text(x: &Json) -> String {
    match x {
        Json::Str(s) => s.clone(),
        Json::Arr(l) => l.iter().map(facet_text).collect::<Vec<_>>().join(", "),
        other => other.text(),
    }
}

/// The `property` forms of a part's fields, from its attributes: the text fields, `attrs`, then its logistics. A wrong one
/// is refused by its Rust name, with `at` (the element's call) first.
pub(crate) fn part_field_forms(at: &str, el: &El) -> Result<Vec<Json>, String> {
    let mut forms = Vec::new();
    for (i, (prop, name)) in PART_FIELDS.iter().enumerate() {
        match el.attr_of(prop) {
            None => {}
            Some(Json::Str(v)) if !v.trim().is_empty() && in_grammar(prop, v) => forms.push(field_form(name, v)),
            Some(_) => return Err(format!("{at}: {} is text ({:?})", rust_name(prop), FIELD_EXAMPLES[i])),
        }
    }
    match el.attr_of("attrs") {
        None => {}
        Some(Json::Obj(attrs)) if attrs.iter().all(|(_, v)| is_text(Some(v))) => {
            for (name, v) in attrs {
                if let Some(arg) = argument_of(name) {
                    return Err(format!("{at}: attrs names {name}: say it with {}", rust_name(arg)));
                }
                if name.trim().is_empty() || OWN_FIELDS.contains(&name.as_str()) {
                    return Err(format!("{at}: attrs names {name}, which is the part's own (its name, value or footprint)"));
                }
                forms.push(field_form(name, v.as_str().unwrap_or_default()));
            }
        }
        Some(_) => return Err(format!("{at}: attrs is an object of text ({{ Tolerance: \"1%\" }})")),
    }
    if let Some(v) = el.attr_of("suppliers") {
        forms.extend(offer_forms(at, v)?);
    }
    if let Some(v) = el.attr_of("alternates") {
        let alternate = |a: &Json| {
            matches!(a, Json::Obj(_))
                && is_text(a.get("manufacturer"))
                && is_text(a.get("mpn"))
                && (a.get("note").is_none() || is_text(a.get("note")))
                && entries(a).iter().all(|(k, _)| ["manufacturer", "mpn", "note"].contains(&k.as_str()))
        };
        if !matches!(v, Json::Arr(l) if !l.is_empty() && l.iter().all(alternate)) {
            return Err(format!("{at}: alternates is a list of {{ manufacturer, mpn, note? }}"));
        }
        forms.push(field_form("Alternates", &v.text()));
    }
    if let Some(v) = el.attr_of("datasheet") {
        if !is_text(Some(v)) {
            return Err(format!("{at}: datasheet is a URL or a path"));
        }
        forms.push(field_form("Datasheet", v.as_str().unwrap_or_default()));
    }
    for (arg, fields) in FACET_FIELDS {
        let Some(v) = el.attr_of(arg) else { continue };
        let facet = check_facet(at, arg, v)?;
        for (k, prop) in *fields {
            if let Some(x) = facet.get(k) {
                forms.push(field_form(prop, &facet_text(x)));
            }
        }
    }
    Ok(forms)
}

/// A part's fields read back from its node's inputs (the `property` forms it carries): the named ones by their
/// argument (TypeScript names), the logistics by theirs, any other in `attrs`. A part with none gives `{}`.
pub fn part_fields(inputs: &Json) -> Json {
    let mut out = Json::obj();
    let mut attrs = Json::obj();
    let mut suppliers: Vec<Json> = Vec::new();
    fn supplier<'a>(suppliers: &'a mut Vec<Json>, name: &str) -> &'a mut Json {
        let i = match suppliers.iter().position(|s| s.get("name").and_then(Json::as_str) == Some(name)) {
            Some(i) => i,
            None => {
                suppliers.push(Json::obj().with("name", name));
                suppliers.len() - 1
            }
        };
        &mut suppliers[i]
    }
    let forms: &[Json] = match inputs.get("foreign") {
        Some(Json::Arr(f)) => f,
        _ => &[],
    };
    for form in forms {
        let items: &[Json] = match form.get("items") {
            Some(Json::Arr(i)) if form.get("kind").and_then(Json::as_str) == Some("list") => i,
            _ => continue,
        };
        let atom = |i: usize| items.get(i).and_then(|a| a.get("v")).and_then(Json::as_str);
        let (Some("property"), Some(name), Some(value)) = (atom(0), atom(1), atom(2)) else { continue };
        if OWN_FIELDS.contains(&name) {
            continue;
        }
        match argument_of(name) {
            Some(arg) if PART_FIELDS.iter().any(|(a, _)| *a == arg) => {
                out.set(arg, value);
            }
            Some("suppliers") if name.ends_with(" Part") => {
                supplier(&mut suppliers, &name[..name.len() - 5]).set("sku", value);
            }
            Some("suppliers") => {
                let terms = Json::parse(value).unwrap_or_else(|_| Json::obj());
                let s = supplier(&mut suppliers, &name[..name.len() - 6]);
                for k in OFFER_TERMS {
                    let Some(t) = terms.get(k) else { continue };
                    let t = match (*k, t) {
                        ("priceBreaks", Json::Arr(breaks)) => Json::Arr(
                            breaks
                                .iter()
                                .map(|b| match b {
                                    Json::Arr(qp) if qp.len() == 2 => Json::obj().with("quantity", qp[0].clone()).with("unitPrice", qp[1].clone()),
                                    other => other.clone(),
                                })
                                .collect(),
                        ),
                        _ => t.clone(),
                    };
                    s.set(k, t);
                }
            }
            Some("alternates") => {
                out.set("alternates", Json::parse(value).unwrap_or(Json::Null));
            }
            Some("datasheet") => {
                out.set("datasheet", value);
            }
            Some(arg) => {
                let fields = FACET_FIELDS.iter().find(|(a, _)| *a == arg).map(|(_, f)| *f).unwrap_or_default();
                let Some((key, _)) = fields.iter().find(|(_, p)| *p == name) else { continue };
                let x = if *key == "certifications" {
                    Json::Arr(value.split(',').map(str::trim).filter(|c| !c.is_empty()).map(Json::from).collect())
                } else if COUNTS.contains(&format!("{arg}.{key}").as_str()) {
                    value.trim().parse::<f64>().map(Json::Num).unwrap_or(Json::Null)
                } else {
                    Json::from(value)
                };
                let mut facet = out.get(arg).cloned().unwrap_or_else(Json::obj);
                facet.set(key, x);
                out.set(arg, facet);
            }
            None => {
                attrs.set(name, value);
            }
        }
    }
    if !suppliers.is_empty() {
        out.set("suppliers", Json::Arr(suppliers));
    }
    if !entries(&attrs).is_empty() {
        out.set("attrs", attrs);
    }
    out
}
