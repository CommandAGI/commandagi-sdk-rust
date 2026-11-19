//! A FACTORY IN A SCENE — the `flow` system a design's `slot().name("scene")` holds (docs/formats.md § simulations,
//! "A factory", in the CommandAGI repository): products on routes through stations of machines with finite buffers,
//! from sources to sinks, run by the simulation engine as discrete events. The same factory as the TypeScript SDK's
//! (`factory-flow.ts`) and the Python SDK's (`commandagi.design.factory`):
//!
//! ```
//! use commandagi::design::factory::*;
//! use commandagi::design::Json;
//!
//! let line = Json::parse(r#"{ "id": "line", "kind": "flow",
//!   "products": [{ "id": "motor", "route": ["wind"], "sink": "dock" }],
//!   "sources": [{ "id": "kits", "product": "motor" }], "sinks": [{ "id": "dock" }],
//!   "stations": [{ "id": "wind", "machines": 4, "cycle_s": 75, "buffer": 12 }] }"#).unwrap();
//! check_flow_system(&line).unwrap();
//! assert_eq!(machines_for(2000.0 / 57600.0, 90.0, 1.0, 0.9, 0.85).unwrap(), 5);
//! ```
//!
//! `check_flow_system` refuses an unknown field, a missing one, a number that is not a number or a `"$parameter"`, and
//! a name that points nowhere, in the engine's own words. The engine checks the rest when the scene runs.

use super::Json;

/// Each part's fields: the ones it needs, then the ones it may have. One table, the same in the three SDKs.
pub const FACTORY_FIELDS: &[(&str, &[&str], &[&str])] = &[
    ("system", &["id", "kind", "products", "stations", "sources", "sinks"], &["seed", "time_scale", "window_s", "shifts", "record"]),
    ("station", &["id", "machines", "cycle_s", "buffer"], &["label", "at", "cycle_cv", "batch", "setup_s", "availability", "mtbf_s", "mttr_s", "yield", "energy_j", "cell"]),
    ("product", &["id", "route", "sink"], &["label", "demand_per_period", "color"]),
    ("source", &["id", "product"], &["label", "at", "per_hour", "arrivals"]),
    ("sink", &["id"], &["label", "at"]),
    ("shifts", &["windows"], &["period_s"]),
];
const NUMBERS: &[(&str, &str)] = &[
    ("machines", "a whole number from 1 to 256"), ("buffer", "a whole number from 1 to 100000"), ("batch", "a whole number from 1 to 1024"),
    ("cycle_s", "the seconds one batch takes, more than 0"), ("cycle_cv", "the spread of the cycle time over its mean, 0 to 4"),
    ("setup_s", "the seconds a change of product takes, 0 or more"), ("yield", "the share of parts that survive, above 0 and at most 1"),
    ("availability", "the share of time it is not under repair, above 0 and at most 1"), ("mtbf_s", "the mean operating seconds between failures, more than 0"),
    ("mttr_s", "the mean seconds a repair takes, more than 0"), ("per_hour", "the parts it releases in a working hour, more than 0"),
    ("demand_per_period", "the parts wanted in one shift period, more than 0"),
    ("energy_j", "the joules one batch takes, 0 or more"),
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
fn list<'a>(spec: &'a Json, key: &str) -> Option<&'a Vec<Json>> {
    match spec.get(key) {
        Some(Json::Arr(items)) => Some(items),
        _ => None,
    }
}

/// Refuses what the engine would refuse by shape or by name, with its words.
pub fn check_flow_system(spec: &Json) -> Result<(), String> {
    let no = |why: String| Err(format!("system {}: {why}", text(spec.get("id"))));
    let fields = |o: &Json, part: &str, at: &str| -> Result<(), String> {
        let (_, needs, may) = FACTORY_FIELDS.iter().find(|(p, _, _)| *p == part).unwrap();
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
    if spec.get("kind").and_then(Json::as_str) != Some("flow") {
        return no("kind is flow".into());
    }
    let mut lists = Vec::new();
    for k in ["stations", "products", "sources", "sinks"] {
        match list(spec, k) {
            Some(items) => lists.push(items),
            None => return no(format!("{k} is a list")),
        }
    }
    let (stations, products, sources, sinks) = (lists[0], lists[1], lists[2], lists[3]);
    for (part, items) in [("station", stations), ("product", products), ("source", sources), ("sink", sinks)] {
        for x in items {
            fields(x, part, &format!("{part} {}", text(x.get("id"))))?;
        }
    }
    if let Some(shifts) = spec.get("shifts") {
        fields(shifts, "shifts", "shifts")?;
    }
    let ids = |items: &Vec<Json>| items.iter().map(|x| text(x.get("id"))).collect::<Vec<_>>();
    let (station_ids, sink_ids, product_ids) = (ids(stations), ids(sinks), ids(products));
    for p in products {
        for step in list(p, "route").map(|v| v.as_slice()).unwrap_or(&[]) {
            let id = match step {
                Json::Str(s) => s.clone(),
                other => text(other.get("station")),
            };
            if !station_ids.contains(&id) {
                return no(format!("product {}: route names \"{id}\", which is not a station", text(p.get("id"))));
            }
        }
        let sink = text(p.get("sink"));
        if !sink_ids.contains(&sink) {
            return no(format!("product {}: sink \"{sink}\" is not one of the sinks", text(p.get("id"))));
        }
    }
    for x in sources {
        let product = text(x.get("product"));
        if !product_ids.contains(&product) {
            return no(format!("source {}: product \"{product}\" is not one of the products", text(x.get("id"))));
        }
    }
    Ok(())
}

/// The machines a rate needs: CEIL(rate * cycle / (batch * availability * load)); `rate` in parts a second into the
/// station, `load` the planned share of a machine's time (0.85).
pub fn machines_for(rate: f64, cycle_s: f64, batch: f64, availability: f64, load: f64) -> Result<u32, String> {
    if !(rate >= 0.0 && cycle_s > 0.0 && batch > 0.0 && availability > 0.0 && availability <= 1.0 && load > 0.0 && load <= 1.0) {
        return Err("machines_for: rate is 0 or more; cycle_s, batch and load are above 0; availability and load are at most 1".into());
    }
    Ok((rate * cycle_s / (batch * availability * load) - 1e-9).ceil().max(0.0) as u32)
}
