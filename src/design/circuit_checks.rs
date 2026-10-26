//! THE CHECKS A CIRCUIT DECLARES: `check()` and `accept()` in a `.sch.rs` or a `.pcb.rs` (the TypeScript SDK's
//! `design/circuit-checks.ts` and the Python SDK's `circuit_checks.py`, element for element):
//!
//! ```text
//! check().id("erc").kind("erc")                                  the electrical rule check (a schematic, a board)
//! check().id("drc").kind("drc").severity("warning")             the design rule check (a board); its errors as warnings
//! check().id("pinout").kind("pinout")                           every part's pinout checked against its datasheet or
//!                                                                measured (its pinout_verified, `super::part_fields`)
//! check().id("footprint").kind("footprint")                     every part's pads checked, and its footprint's pad
//!                                                                count, pitch and body equal to its package_dimensions
//! check().id("cost").kind("cost").options(json(r#"{"quantities": [10, 100]}"#))
//!                                                                a logistics check (sourcing, process, assembly,
//!                                                                inspection, compliance, shipping, lifecycle, cost:
//!                                                                docs/formats.md § logistics), with its own options
//! accept().id("lbl").check("sourcing").rule("no-supplier").refs(["J9"]).reason("…")   a logistics finding of one part
//! accept().id("tx").check("erc").rule("pin_conflict").refs(["U1", "U3"]).reason("the bus is open-drain")
//! accept().id("edge").check("drc").rule("clearance").region(json(r#"{"min": [0, 0], "max": [3, 3]}"#)).reason("…")
//! ```
//!
//! An accept suppresses the findings of its check with its rule that it matches: every ref it names is a part the
//! finding is about, its net is the finding's, and a DRC finding lies in its region (board millimetres). They are no
//! node of the circuit: the reader keeps them in the graph's `meta` (`checks`, `accepted`), each with the call it came
//! from in `source`, so the circuit the editor draws is the same with them or without them.

use super::element::{fn_name, rust_name, El};
use super::json::Json;

fn at(el: &El) -> String {
    match el.attr_of("id").and_then(Json::as_str) {
        Some(id) => format!("{}().id({id:?})", fn_name(el.tag)),
        None => format!("{}()", fn_name(el.tag)),
    }
}

fn only(el: &El, allowed: &[&str]) -> Result<(), String> {
    match el.attrs.iter().find(|(k, _)| !allowed.contains(&k.as_str())) {
        Some((k, _)) => {
            let takes: Vec<String> = allowed.iter().map(|a| rust_name(a)).collect();
            Err(format!("{}: {} is not read (it takes {})", at(el), rust_name(k), takes.join(", ")))
        }
        None => Ok(()),
    }
}

fn text(el: &El, prop: &str, what: &str, required: bool) -> Result<Option<String>, String> {
    match el.attr_of(prop) {
        None if !required => Ok(None),
        Some(Json::Str(v)) if !v.trim().is_empty() => Ok(Some(v.clone())),
        _ => Err(format!("{}: {} is {what}", at(el), rust_name(prop))),
    }
}

fn id_of(el: &El, what: &str, taken: &[Json]) -> Result<String, String> {
    let id = text(el, "id", what, true)?.unwrap_or_default();
    if !id.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '-')) {
        return Err(format!("{}: id is letters, digits, _ . -", at(el)));
    }
    if taken.iter().any(|c| c.get("id").and_then(Json::as_str) == Some(id.as_str())) {
        return Err(format!("two checks or accepts are called {id}"));
    }
    Ok(id)
}

fn point(v: Option<&Json>) -> bool {
    matches!(v, Some(Json::Arr(p)) if p.len() == 2 && p.iter().all(|n| n.as_f64().is_some_and(f64::is_finite)))
}

/// Read the circuit's `check()` and `accept()` elements into its graph's `meta`; the other elements are returned.
pub(crate) fn declare_circuit_checks<'a>(meta: &mut Json, elements: Vec<&'a El>) -> Result<Vec<&'a El>, String> {
    let mut checks: Vec<Json> = Vec::new();
    let mut accepted: Vec<Json> = Vec::new();
    let mut rest = Vec::new();
    for el in elements {
        let taken = [checks.as_slice(), accepted.as_slice()].concat();
        match el.tag {
            "check" => {
                only(el, &["id", "kind", "severity", "options"])?;
                let id = id_of(el, "the check's name (erc)", &taken)?;
                let kind = text(el, "kind", "what is checked: erc, drc, pinout, footprint, or a logistics kind (sourcing, process, assembly, inspection, compliance, shipping, lifecycle, cost)", true)?
                    .unwrap_or_default();
                let mut c = Json::obj().with("id", id.as_str()).with("kind", kind.as_str());
                match el.attr_of("severity") {
                    None => {}
                    Some(Json::Str(s)) if s == "error" || s == "warning" => {
                        c.set("severity", s.as_str());
                    }
                    Some(_) => return Err(format!("{}: severity is \"error\" or \"warning\"", at(el))),
                }
                match el.attr_of("options") {
                    None => {}
                    Some(o @ Json::Obj(_)) => {
                        c.set("options", o.clone());
                    }
                    Some(_) => return Err(format!("{}: options is an object (the cost check's {{\"quantities\": [10, 100]}})", at(el))),
                }
                if let Some(s) = el.source_json() {
                    c.set("source", s);
                }
                checks.push(c);
            }
            "accept" => {
                only(el, &["id", "check", "rule", "refs", "net", "region", "reason"])?;
                let id = id_of(el, "the accept's name", &taken)?;
                let refs = el.attr_of("refs");
                if let Some(r) = refs {
                    if !matches!(r, Json::Arr(l) if !l.is_empty() && l.iter().all(|x| x.as_str().is_some_and(|s| !s.is_empty()))) {
                        return Err(format!("{}: refs is a list of part references ([\"U1\", \"U3\"])", at(el)));
                    }
                }
                let region = el.attr_of("region");
                if let Some(r) = region {
                    if !(matches!(r, Json::Obj(_)) && point(r.get("min")) && point(r.get("max"))) {
                        return Err(format!("{}: region is json(r#\"{{\"min\": [x, y], \"max\": [x, y]}}\"#) in board millimetres", at(el)));
                    }
                }
                let net = text(el, "net", "a net's name", false)?;
                let check = text(el, "check", "the id of the check whose finding it accepts", true)?.unwrap_or_default();
                let rule = text(el, "rule", "the rule of the finding it accepts (pin_conflict, clearance)", true)?.unwrap_or_default();
                let mut a = Json::obj().with("id", id.as_str()).with("check", check.as_str()).with("rule", rule.as_str());
                if let Some(r) = refs {
                    a.set("refs", r.clone());
                }
                if let Some(n) = net {
                    a.set("net", n.as_str());
                }
                if let Some(r) = region {
                    a.set("region", Json::obj().with("min", r.get("min").cloned().unwrap()).with("max", r.get("max").cloned().unwrap()));
                }
                let reason = text(el, "reason", "why the finding is right on purpose", true)?.unwrap_or_default();
                a.set("reason", reason.as_str());
                if let Some(s) = el.source_json() {
                    a.set("source", s);
                }
                accepted.push(a);
            }
            _ => rest.push(el),
        }
    }
    if !checks.is_empty() {
        meta.set("checks", Json::Arr(checks));
    }
    if !accepted.is_empty() {
        meta.set("accepted", Json::Arr(accepted));
    }
    Ok(rest)
}
