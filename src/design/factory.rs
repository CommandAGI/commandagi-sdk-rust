//! A PLANT IN A SCENE — the `operations` system a design's `slot().name("scene")` holds (docs/formats.md § factories,
//! in the CommandAGI repository): items with their BOMs, locations, stations of machines, routings of ops, work
//! orders, resources, moves, deliveries and shipments, run by the simulation engine as discrete events whose material
//! is entities. The same plant as the TypeScript SDK's (`factory-operations.ts`) and the Python SDK's
//! (`commandagi.design.factory`):
//!
//! ```
//! use commandagi::design::factory::*;
//! use commandagi::design::Json;
//!
//! let line = Json::parse(r#"{ "id": "line", "kind": "operations",
//!   "items": [{ "id": "kit" }, { "id": "motor", "bom": [{ "item": "kit", "qty": 1 }] }],
//!   "locations": [{ "id": "kits", "kind": "store", "capacity": 500, "stock": [{ "item": "kit", "qty": 400 }] }, { "id": "fg", "kind": "store", "capacity": 500 }],
//!   "stations": [{ "id": "wind", "machines": 4, "in": "kits", "out": "fg" }],
//!   "routings": [{ "item": "motor", "to": "fg", "ops": [{ "id": "wind", "station": "wind", "cycle_s": 75, "consumes": [{ "item": "kit", "qty": 1 }] }] }] }"#).unwrap();
//! check_operations_system(&line).unwrap();
//! assert_eq!(machines_for(2000.0 / 57600.0, 90.0, 1.0, 0.9, 0.85).unwrap(), 5);
//! ```
//!
//! `check_operations_system` refuses an unknown field, a missing one, a number that is not a number or a
//! `"$parameter"`, and a name that points nowhere, in the engine's own words. The engine checks the rest when the scene
//! runs.

use super::Json;

/// Each part's fields: the ones it needs, then the ones it may have. One table, the same in the three SDKs.
pub const PLANT_FIELDS: &[(&str, &[&str], &[&str])] = &[
    ("system", &["id", "kind", "items", "locations", "stations", "routings"], &["seed", "time_scale", "window_s", "shifts", "record", "resources", "orders", "inbound", "outbound", "moves"]),
    ("item", &["id"], &["label", "bom", "body", "ideal_cycle_s"]),
    ("location", &["id", "kind", "capacity"], &["label", "stock", "zone", "slots", "transit_s", "to", "drive"]),
    ("resource", &["id", "kind", "skills"], &["label", "members", "count", "physical"]),
    ("station", &["id", "in", "out"], &["label", "units", "machines", "mtbf_s", "mttr_s", "repair", "maintenance", "tool", "plc", "zone"]),
    ("routing", &["item", "to", "ops"], &[]),
    ("op", &["id", "station", "cycle_s"], &["cycle_cv", "ideal_s", "setup_s", "consumes", "needs", "load_s", "inspect", "rework", "then", "body", "energy_j", "cell"]),
    ("order", &["id", "item", "qty"], &["label", "release_s", "due_s", "priority"]),
    ("inbound", &["id", "item", "qty", "to"], &["label", "at_s", "every_s", "first_s"]),
    ("outbound", &["id", "item", "qty", "from"], &["label", "at_s", "every_s", "first_s"]),
    ("move", &["id", "from", "to", "lot", "reorder_at", "by", "travel_s"], &["label", "item"]),
    ("shifts", &["windows"], &["period_s"]),
];
const NUMBERS: &[(&str, &str)] = &[
    ("capacity", "a whole number from 1 to 100000"), ("machines", "a whole number from 1 to 1000"), ("qty", "a whole number from 1 to 100000"),
    ("cycle_s", "the seconds one entity takes, more than 0"), ("setup_s", "seconds, 0 or more"), ("mtbf_s", "the mean operating seconds between failures, more than 0"),
    ("mttr_s", "the mean seconds a repair takes, more than 0"), ("energy_j", "the joules one entity takes, 0 or more"), ("lot", "a whole number from 1 to 100000"),
    ("reorder_at", "a whole number from 0 to 100000"), ("travel_s", "seconds, 0 or more"), ("transit_s", "the seconds an entity rides it, more than 0"),
];

fn is_number(v: &Json) -> bool {
    match v {
        Json::Num(n) => n.is_finite(),
        Json::Str(s) => {
            let mut c = s.chars();
            c.next() == Some('$') && c.next().is_some_and(|f| f.is_ascii_alphabetic() || f == '_') && s[2..].chars().all(|x| x.is_ascii_alphanumeric() || x == '_')
        }
        _ => false,
    }
}
fn text(v: Option<&Json>) -> String {
    match v {
        Some(Json::Str(s)) => s.clone(),
        Some(other) => other.text(),
        None => "undefined".into(),
    }
}
fn quoted(v: Option<&Json>) -> String {
    match v {
        Some(Json::Str(s)) => format!("\"{s}\""),
        Some(other) => other.text(),
        None => "undefined".into(),
    }
}
fn items_of<'a>(o: &'a Json, key: &str) -> &'a [Json] {
    match o.get(key) {
        Some(Json::Arr(items)) => items,
        _ => &[],
    }
}

/// Refuses what the engine would refuse by shape or by name, with its words.
pub fn check_operations_system(spec: &Json) -> Result<(), String> {
    let no = |why: String| Err(format!("system {}: {why}", text(spec.get("id"))));
    let fields = |o: &Json, part: &str, at: &str| -> Result<(), String> {
        let (_, needs, may) = PLANT_FIELDS.iter().find(|(p, _, _)| *p == part).unwrap();
        let Json::Obj(entries) = o else { return no(format!("{at} is an object")) };
        for (k, _) in entries {
            if !needs.contains(&k.as_str()) && !may.contains(&k.as_str()) {
                return no(format!("{at}: unknown field {k} (it has {})", [*needs, *may].concat().join(", ")));
            }
        }
        for k in needs.iter() {
            if matches!(o.get(k), None | Some(Json::Null)) {
                return no(format!("{at}: {k} is needed"));
            }
        }
        for (k, why) in NUMBERS {
            if o.get(k).is_some_and(|v| !is_number(v)) {
                return no(format!("{at}: {k} is {why}"));
            }
        }
        Ok(())
    };
    fields(spec, "system", "the system")?;
    if spec.get("kind").and_then(Json::as_str) != Some("operations") {
        return no("kind is operations".into());
    }
    for k in ["items", "locations", "resources", "stations", "routings", "orders", "inbound", "outbound", "moves"] {
        if spec.get(k).is_some_and(|v| !matches!(v, Json::Arr(_))) {
            return no(format!("{k} is a list"));
        }
    }
    let (items, locations, resources, stations, routings) = (items_of(spec, "items"), items_of(spec, "locations"), items_of(spec, "resources"), items_of(spec, "stations"), items_of(spec, "routings"));
    let (orders, inbound, outbound, moves) = (items_of(spec, "orders"), items_of(spec, "inbound"), items_of(spec, "outbound"), items_of(spec, "moves"));
    for (part, xs) in [("item", items), ("location", locations), ("resource", resources), ("station", stations), ("order", orders), ("inbound", inbound), ("outbound", outbound), ("move", moves)] {
        for x in xs {
            fields(x, part, &format!("{part} {}", text(x.get("id"))))?;
        }
    }
    if let Some(shifts) = spec.get("shifts") {
        fields(shifts, "shifts", "shifts")?;
    }
    let ids = |xs: &[Json]| xs.iter().map(|x| text(x.get("id"))).collect::<Vec<_>>();
    let (item_ids, loc_ids, station_ids) = (ids(items), ids(locations), ids(stations));
    let skills: Vec<String> = resources.iter().flat_map(|r| items_of(r, "skills").iter().map(|s| text(Some(s)))).collect();
    let item = |v: Option<&Json>, at: &str| if item_ids.contains(&text(v)) { Ok(()) } else { no(format!("{at}: item {} is not one of the items", quoted(v))) };
    let loc = |v: Option<&Json>, at: &str| if loc_ids.contains(&text(v)) { Ok(()) } else { no(format!("{at}: location {} is not one of the locations", quoted(v))) };
    let skill = |v: Option<&Json>, at: &str| if skills.contains(&text(v)) { Ok(()) } else { no(format!("{at}: skill {} is no resource's", quoted(v))) };
    let lines = |o: &Json, key: &str, at: &str| -> Result<(), String> {
        for (i, l) in items_of(o, key).iter().enumerate() {
            item(l.get("item"), &format!("{at} {}", i + 1))?;
        }
        Ok(())
    };
    for x in items {
        lines(x, "bom", &format!("item {}: bom", text(x.get("id"))))?;
    }
    for x in locations {
        lines(x, "stock", &format!("location {}: stock", text(x.get("id"))))?;
        if x.get("to").is_some() {
            loc(x.get("to"), &format!("location {}", text(x.get("id"))))?;
        }
    }
    for x in stations {
        let id = text(x.get("id"));
        loc(x.get("in"), &format!("station {id}: in"))?;
        loc(x.get("out"), &format!("station {id}: out"))?;
        if x.get("repair").is_some() {
            skill(x.get("repair"), &format!("station {id}: repair"))?;
        }
    }
    let mut made = Vec::new();
    for (i, r) in routings.iter().enumerate() {
        let at = format!("routing {}", i + 1);
        fields(r, "routing", &at)?;
        item(r.get("item"), &at)?;
        made.push(text(r.get("item")));
        loc(r.get("to"), &format!("{at}: to"))?;
        for op in items_of(r, "ops") {
            let wh = format!("{at} op {}", text(op.get("id")));
            fields(op, "op", &wh)?;
            if !station_ids.contains(&text(op.get("station"))) {
                return no(format!("{wh}: station {} is not one of the stations", quoted(op.get("station"))));
            }
            lines(op, "consumes", &format!("{wh}: consumes"))?;
            if op.get("needs").is_some() {
                skill(op.get("needs"), &wh)?;
            }
        }
    }
    for x in orders {
        if !made.contains(&text(x.get("item"))) {
            return no(format!("order {}: no routing makes {}", text(x.get("id")), quoted(x.get("item"))));
        }
    }
    for x in inbound {
        let id = text(x.get("id"));
        item(x.get("item"), &format!("inbound {id}"))?;
        loc(x.get("to"), &format!("inbound {id}: to"))?;
    }
    for x in outbound {
        let id = text(x.get("id"));
        item(x.get("item"), &format!("outbound {id}"))?;
        loc(x.get("from"), &format!("outbound {id}: from"))?;
    }
    for x in moves {
        let id = text(x.get("id"));
        if x.get("item").is_some() {
            item(x.get("item"), &format!("move {id}"))?;
        }
        loc(x.get("from"), &format!("move {id}: from"))?;
        loc(x.get("to"), &format!("move {id}: to"))?;
        skill(x.get("by"), &format!("move {id}"))?;
    }
    Ok(())
}

/// The machines a rate needs: CEIL(rate * cycle / (batch * availability * load)); `rate` in entities a second into the
/// station, `load` the planned share of a machine's time (0.85).
pub fn machines_for(rate: f64, cycle_s: f64, batch: f64, availability: f64, load: f64) -> Result<u32, String> {
    if !(rate >= 0.0 && cycle_s > 0.0 && batch > 0.0 && availability > 0.0 && availability <= 1.0 && load > 0.0 && load <= 1.0) {
        return Err("machines_for: rate is 0 or more; cycle_s, batch and load are above 0; availability and load are at most 1".into());
    }
    Ok((rate * cycle_s / (batch * availability * load) - 1e-9).ceil().max(0.0) as u32)
}
