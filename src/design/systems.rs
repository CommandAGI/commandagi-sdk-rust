//! SYSTEMS IN RUST — requirements (`.req.rs`), a system's budgets (`.system.rs`) and an interface between two parts
//! (`.interface.rs`), written as elements. The same elements, and the same documents, as the TypeScript SDK's
//! `systems.ts` and the Python SDK's `systems.py`:
//!
//! ```
//! use commandagi::design::systems::*;
//!
//! fn document() -> El {
//!     requirements([
//!         requirement().id("mass").text("The rover weighs at most 500 g.").method("analysis")
//!             .verified_by("Rover.system.rs#mass").design(["cad/Rover.3d.rs"]),
//!     ])
//!     .name("Rover")
//! }
//! # commandagi::design::declare(document()).unwrap();
//! ```
//!
//! The rule of the ontology's files: a record is an element and its fields are the element's attributes (`verified_by`
//! is `verifiedBy`; `ref_` is `ref`). A requirement is named by a ref, never by an id alone (`Rover.req.rs#mass`). A
//! side's maps are JSON: `.defines(json(r#"{"BOARD": 1}"#))`. Nothing adds a default. A `bom()` gives masses; a cost
//! budget reads the logistics roll-up of the design it names (`.design("cad/Assembly.3d.rs")`: a 3D document, a
//! schematic or a board with a cost check), the one source of cost, so no contribution is a cost. A `param()` states a
//! number with its unit, or a text a channel states (`.param().id("framing").value("8N1")`: a format, a dialect, a
//! transport) and then no unit. A side's `origin`, `rotation` (degrees counter-clockwise) and `face` (`top`, or `bottom`
//! for the side seen from the mating board, whose frame is mirrored) put the interface's frame in the side's. The checks
//! are the host's (`commandagi check`, `commandagi trace`).

use super::documents::{declare_document, DocTree, TagRule, Vocabulary};
use super::element::{attributes, elements};
use super::{Declared, Family};
pub use super::{fragment, json, El, Json, Value};

elements! {
    /// Requirements: each a `requirement()`.
    requirements: children;
    /// One requirement: `.id(…).text(…).method("analysis").verified_by("Cam.system.rs#mass")`.
    requirement: leaf;
    /// A system: its modes, rails, converters, loads, contributions, boms, budgets and checks.
    system: children;
    /// An operating mode: `.id("stream-lte")`.
    mode: leaf;
    /// A supply rail: `.id("+5V").voltage(5)`.
    rail: leaf;
    /// A converter from one rail to another: `.id("U3").input("+5V").output("VBAT").efficiency(0.88)`.
    converter: leaf;
    /// What a part draws from a rail in a mode: `.id(…).part("U4").rail("VBAT").mode("lte").current(0.67).source(…)`.
    load: leaf;
    /// A number a part contributes to a budget that no file states: `.quantity("mass").part("P1").value(15.2).source(…)`.
    contribution: leaf;
    /// A bill of materials the roll-up reads: `.ref_("bom.csv")` (or a 3D document's).
    bom: leaf;
    /// A budget: `.id("mass").quantity("mass").limit(60).unit("g")`.
    budget: leaf;
    /// A check of the system: `.id(…).kind("interface" | "trace").of(…)`.
    check: leaf;
    /// An interface between two parts: its frame, pins, holes, messages, params and sides.
    interface: children;
    /// What positions are in: `.description(…).unit("mm")`.
    frame: leaf;
    /// A pin of the interface: `.number(1).signal("+5V").required(true).at([0.0, 0.0])`.
    pin: leaf;
    /// A mating hole: `.id("a").at([0.0, 0.0]).diameter(2.2)`.
    hole: leaf;
    /// A message across the interface: `.id(…).text("AT+CNMP=38")`, or `.code(3).define("MSG_X")`.
    message: leaf;
    /// A constant of the interface with its unit: `.id("baud").value(115200).unit("Bd")`.
    param: leaf;
    /// A side that implements the interface: `.id(…).ref_("ee/base.circuit.ts").parts(["J1", "J2"])`.
    side: leaf;
}

attributes! {
    /// The fields of the systems' records, chained on their elements.
    pub trait SystemsAttrs {
        name; description; id; text; rationale; method; verified_by; design; parent;
        voltage; source; input; output; efficiency; part; rail; mode; current; power; peak_current; peak_power; peak; inrush_current; duty;
        quantity; value; unit; qty; currency; per; ref_; limit; resistance; margin; kind; of;
        number; signal; dir; gpio; at; required; diameter; code; define;
        parts; channel; origin; rotation; face; defines; pins; params; files;
    }
}

pub(crate) const FAMILY: Family = Family { module: "systems", roots: &["requirements", "system", "interface"], declare };

pub const VERIFICATION_METHODS: &[&str] = &["test", "analysis", "inspection", "demonstration"];
pub const BUDGET_QUANTITIES: &[&str] = &["mass", "power", "cost", "thermal"];
pub const SYSTEM_CHECK_KINDS: &[&str] = &["interface", "trace"];
pub const SIDE_KINDS: &[&str] = &["circuit", "3d", "firmware", "definition"];
/// Which face of a side the interface's frame is on: `top` as the side's own frame has it, `bottom` from the mating board.
pub const SIDE_FACES: &[&str] = &["top", "bottom"];

pub const REQUIREMENT_FIELDS: &[&str] = &["id", "text", "rationale", "method", "verifiedBy", "design", "parent"];
pub const MODE_FIELDS: &[&str] = &["id", "description"];
pub const RAIL_FIELDS: &[&str] = &["id", "voltage", "description", "source"];
pub const CONVERTER_FIELDS: &[&str] = &["id", "input", "output", "efficiency", "source"];
pub const LOAD_FIELDS: &[&str] = &["id", "part", "rail", "mode", "current", "power", "voltage", "peakCurrent", "peakPower", "inrushCurrent", "duty", "source"];
pub const CONTRIBUTION_FIELDS: &[&str] = &["id", "quantity", "part", "value", "unit", "qty", "mode", "rail", "source"];
pub const BUDGET_FIELDS: &[&str] = &["id", "quantity", "limit", "unit", "mode", "rail", "peak", "per", "currency", "design", "resistance", "margin", "source", "description"];
pub const PIN_FIELDS: &[&str] = &["number", "signal", "dir", "voltage", "gpio", "at", "required", "description"];
pub const HOLE_FIELDS: &[&str] = &["id", "at", "diameter", "description"];
pub const MESSAGE_FIELDS: &[&str] = &["id", "text", "code", "define", "dir", "description"];
pub const PARAM_FIELDS: &[&str] = &["id", "value", "unit", "description"];
pub const SIDE_FIELDS: &[&str] = &["id", "ref", "kind", "part", "parts", "channel", "origin", "rotation", "face", "defines", "pins", "params", "files", "description"];
const ROOT_FIELDS: &[&str] = &["name", "description"];

const REQUIREMENTS: Vocabulary = Vocabulary {
    format: "requirements",
    noun: "a requirements document",
    root: "requirements",
    tags: &[
        TagRule::new("requirements", &[]).attrs(ROOT_FIELDS),
        TagRule::new("requirement", &["requirements"]).key("id").required(&["id", "text"]).attrs(REQUIREMENT_FIELDS),
    ],
    from_tree: requirements_of,
};

const SYSTEM: Vocabulary = Vocabulary {
    format: "system",
    noun: "a system",
    root: "system",
    tags: &[
        TagRule::new("system", &[]).attrs(ROOT_FIELDS),
        TagRule::new("mode", &["system"]).key("id").required(&["id"]).attrs(MODE_FIELDS),
        TagRule::new("rail", &["system"]).key("id").required(&["id", "voltage"]).attrs(RAIL_FIELDS),
        TagRule::new("converter", &["system"]).key("id").required(&["id", "input", "output", "efficiency"]).attrs(CONVERTER_FIELDS),
        TagRule::new("load", &["system"]).key("id").required(&["id", "part", "rail", "mode"]).attrs(LOAD_FIELDS),
        TagRule::new("contribution", &["system"]).key("id").required(&["id", "quantity", "part", "value"]).attrs(CONTRIBUTION_FIELDS),
        TagRule::new("bom", &["system"]).key("ref").required(&["ref"]).attrs(&["ref"]),
        TagRule::new("budget", &["system"]).key("id").required(&["id", "quantity", "limit"]).attrs(BUDGET_FIELDS),
        TagRule::new("check", &["system"]).key("id").required(&["id", "kind", "of"]).attrs(&["id", "kind", "of"]),
    ],
    from_tree: system_of,
};

const INTERFACE: Vocabulary = Vocabulary {
    format: "interface",
    noun: "an interface",
    root: "interface",
    tags: &[
        TagRule::new("interface", &[]).attrs(ROOT_FIELDS),
        TagRule::new("frame", &["interface"]).single().attrs(&["description", "unit"]),
        TagRule::new("pin", &["interface"]).key("number").required(&["number", "signal"]).attrs(PIN_FIELDS),
        TagRule::new("hole", &["interface"]).key("id").required(&["id", "at"]).attrs(HOLE_FIELDS),
        TagRule::new("message", &["interface"]).key("id").required(&["id"]).attrs(MESSAGE_FIELDS),
        TagRule::new("param", &["interface"]).key("id").required(&["id", "value"]).attrs(PARAM_FIELDS),
        TagRule::new("side", &["interface"]).key("id").required(&["id", "ref"]).attrs(SIDE_FIELDS),
    ],
    from_tree: interface_of,
};

fn declare(root: El) -> Result<Declared, String> {
    let v = match root.tag {
        "requirements" => &REQUIREMENTS,
        "system" => &SYSTEM,
        _ => &INTERFACE,
    };
    declare_document(&root, v)
}

/// How a message names a record: its call with the attribute that names it (`requirement().id("x")`).
fn at(t: &DocTree, key: &str) -> String {
    let call = super::element::fn_name(t.tag);
    match t.attr_of(key) {
        Some(v) => format!("{call}().{}({})", super::element::rust_name(key), v.text()),
        None => format!("{call}()"),
    }
}
fn num(v: Option<&Json>) -> bool {
    matches!(v, Some(Json::Num(n)) if n.is_finite())
}
fn int(v: Option<&Json>) -> bool {
    matches!(v, Some(Json::Num(n)) if n.is_finite() && n.fract() == 0.0)
}
fn string(v: Option<&Json>) -> bool {
    matches!(v, Some(Json::Str(_)))
}
fn refs(v: &Json) -> bool {
    match v {
        Json::Str(_) => true,
        Json::Arr(items) => items.iter().all(|x| matches!(x, Json::Str(_))),
        _ => false,
    }
}
fn pair(v: Option<&Json>, n: usize) -> bool {
    matches!(v, Some(Json::Arr(items)) if items.len() == n && items.iter().all(|x| num(Some(x))))
}
fn one_of(v: Option<&Json>, allowed: &[&str]) -> bool {
    matches!(v, Some(Json::Str(s)) if allowed.contains(&s.as_str()))
}
fn said(v: Option<&Json>) -> String {
    v.map(Json::text).unwrap_or_else(|| "undefined".into())
}
fn need(cond: bool, message: impl FnOnce() -> String) -> Result<(), String> {
    if cond {
        Ok(())
    } else {
        Err(message())
    }
}

/// The root's own fields, then the children grouped by tag into the document's lists, in the order written.
fn grouped(t: &DocTree, lists: &[(&str, &str)]) -> Json {
    let mut out = Json::obj();
    for k in ROOT_FIELDS {
        if let Some(v) = t.attr_of(k) {
            out.set(k, v.clone());
        }
    }
    for (tag, field) in lists {
        let records: Vec<Json> = t.children.iter().filter(|c| c.tag == *tag).map(DocTree::record).collect();
        if !records.is_empty() {
            out.set(field, records);
        }
    }
    out
}

fn requirements_of(t: &DocTree) -> Result<Json, String> {
    for c in &t.children {
        let w = at(c, "id");
        let id_ok = matches!(c.attr_of("id"), Some(Json::Str(s)) if !s.is_empty() && s.chars().next().unwrap().is_ascii_alphanumeric() && s.chars().all(|ch| ch.is_ascii_alphanumeric() || "._-".contains(ch)));
        need(id_ok, || format!(r#"{w}: id is letters, digits, ".", "_" and "-" (a ref names it as <file>#<id>)"#))?;
        need(matches!(c.attr_of("text"), Some(Json::Str(s)) if !s.trim().is_empty()), || format!("{w}: text says what is required"))?;
        need(c.attr_of("method").is_none() || one_of(c.attr_of("method"), VERIFICATION_METHODS), || format!("{w}: method must be one of {} (got {})", VERIFICATION_METHODS.join(", "), said(c.attr_of("method"))))?;
        for k in ["verifiedBy", "design"] {
            need(c.attr_of(k).map_or(true, refs), || format!("{w}: {k} is a ref or a list of refs"))?;
        }
        need(c.attr_of("parent").is_none() || string(c.attr_of("parent")), || format!("{w}: parent is a ref to the requirement this one is derived from"))?;
    }
    Ok(grouped(t, &[("requirement", "requirements")]))
}

/// A built-in quantity, or a declared one's name (lower-case letters, digits and "-") with its unit: the unit names the
/// family the host converts within (m3/h, m2, slm).
fn quantity_of(c: &DocTree, w: &str) -> Result<(), String> {
    if one_of(c.attr_of("quantity"), BUDGET_QUANTITIES) {
        return Ok(());
    }
    let q = c.attr_of("quantity").and_then(Json::as_str);
    let named = matches!(q, Some(s) if s.chars().next().is_some_and(|ch| ch.is_ascii_lowercase()) && s.chars().all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-'));
    need(named, || format!(r#"{w}: quantity must be one of {}, or a quantity's name in lower-case letters, digits and "-" ("exhaust-flow") (got {})"#, BUDGET_QUANTITIES.join(", "), said(c.attr_of("quantity"))))?;
    need(matches!(c.attr_of("unit"), Some(Json::Str(s)) if !s.trim().is_empty()), || format!(r#"{w}: a quantity of {} states its unit (unit="m3/h"): the unit names what it measures"#, q.unwrap_or_default()))
}

fn system_of(t: &DocTree) -> Result<Json, String> {
    for c in &t.children {
        let w = at(c, if c.tag == "bom" { "ref" } else { "id" });
        match c.tag {
            "rail" => need(num(c.attr_of("voltage")), || format!("{w}: voltage is a number of volts"))?,
            "converter" => need(matches!(c.attr_of("efficiency"), Some(Json::Num(e)) if *e > 0.0 && *e <= 1.0), || format!("{w}: efficiency is a number in (0, 1] (0.88 for 88 %)"))?,
            "load" => {
                need(c.attr_of("current").is_some() || c.attr_of("power").is_some(), || format!("{w}: a load states its current (A) or its power (W)"))?;
                for k in ["current", "power", "voltage", "peakCurrent", "peakPower", "inrushCurrent"] {
                    need(c.attr_of(k).is_none() || num(c.attr_of(k)), || format!("{w}: {k} is a number"))?;
                }
                need(c.attr_of("duty").is_none() || matches!(c.attr_of("duty"), Some(Json::Num(d)) if *d > 0.0 && *d <= 1.0), || format!("{w}: duty is the share of time the load draws, a number in (0, 1] (0.25 for 25 %)"))?;
            }
            "contribution" => {
                quantity_of(c, &w)?;
                need(c.attr_of("quantity").and_then(Json::as_str) != Some("cost"), || {
                    format!("{w}: a cost is stated in the design's logistics (a part's suppliers, a made part's rate), which a cost budget's design rolls up, not as a contribution")
                })?;
                need(num(c.attr_of("value")), || format!("{w}: value is a number"))?;
                need(matches!(c.attr_of("source"), Some(Json::Str(s)) if !s.trim().is_empty()), || format!(r#"{w}: source says where the number was read (a datasheet, a measurement, "estimate: …")"#))?;
            }
            "budget" => {
                quantity_of(c, &w)?;
                need(num(c.attr_of("limit")), || format!("{w}: limit is a number"))?;
                need(c.attr_of("margin").is_none() || matches!(c.attr_of("margin"), Some(Json::Num(m)) if *m >= 0.0 && *m < 1.0), || format!("{w}: margin is a fraction of the limit kept free, in [0, 1)"))?;
                let q = c.attr_of("quantity").and_then(Json::as_str);
                if q == Some("power") {
                    need(string(c.attr_of("rail")) && string(c.attr_of("mode")), || format!("{w}: a power budget names its rail and its mode"))?;
                }
                need(c.attr_of("peak").is_none() || (matches!(c.attr_of("peak"), Some(Json::Bool(_))) && q == Some("power")), || {
                    format!("{w}: peak is true or false, on a power budget (each load at its peakCurrent or peakPower)")
                })?;
                if q == Some("cost") {
                    need(string(c.attr_of("design")), || format!(r#"{w}: a cost budget names the design whose logistics roll-up it reads (design="cad/Assembly.3d.rs")"#))?;
                }
                if q == Some("thermal") {
                    need(string(c.attr_of("mode")) && num(c.attr_of("resistance")), || format!("{w}: a thermal budget names its mode and the thermal resistance to the ambient (resistance, K/W)"))?;
                }
            }
            "check" => need(one_of(c.attr_of("kind"), SYSTEM_CHECK_KINDS), || format!("{w}: kind must be one of {} (got {})", SYSTEM_CHECK_KINDS.join(", "), said(c.attr_of("kind"))))?,
            _ => {}
        }
    }
    Ok(grouped(
        t,
        &[("mode", "modes"), ("rail", "rails"), ("converter", "converters"), ("load", "loads"), ("contribution", "contributions"), ("bom", "boms"), ("budget", "budgets"), ("check", "checks")],
    ))
}

fn interface_of(t: &DocTree) -> Result<Json, String> {
    for c in &t.children {
        let w = at(c, if c.tag == "pin" { "number" } else { "id" });
        match c.tag {
            "frame" => need(c.attr_of("unit").is_none() || c.attr_of("unit") == Some(&Json::from("mm")), || format!(r#"{w}: positions are in mm (unit "mm")"#))?,
            "pin" => {
                need(int(c.attr_of("number")) && c.attr_of("number").and_then(Json::as_f64).unwrap_or(0.0) > 0.0, || format!("{w}: number is the pin's number, 1, 2, 3 …"))?;
                need(c.attr_of("at").is_none() || pair(c.attr_of("at"), 2), || format!("{w}: at is [x, y] mm in the interface's frame"))?;
                need(c.attr_of("gpio").is_none() || int(c.attr_of("gpio")), || format!("{w}: gpio is the GPIO number of the pin on the side that has one"))?;
                need(c.attr_of("required").map_or(true, |v| matches!(v, Json::Bool(_))), || format!("{w}: required is true or false"))?;
            }
            "hole" => {
                need(pair(c.attr_of("at"), 2), || format!("{w}: at is [x, y] mm in the interface's frame"))?;
                need(c.attr_of("diameter").is_none() || num(c.attr_of("diameter")), || format!("{w}: diameter is mm"))?;
            }
            "message" => need(c.attr_of("text").is_some() || c.attr_of("code").is_some(), || format!("{w}: a message has its text, its code, or both"))?,
            "param" => {
                need(num(c.attr_of("value")) || matches!(c.attr_of("value"), Some(Json::Str(s)) if !s.trim().is_empty()), || {
                    format!(r#"{w}: value is a number with its unit in unit (115200), or a text a channel states (a format, a dialect, a transport: "8N1")"#)
                })?;
                need(c.attr_of("unit").is_none() || num(c.attr_of("value")), || format!("{w}: a parameter stated as a text has no unit (unit is for a number)"))?;
            }
            "side" => {
                need(c.attr_of("kind").is_none() || one_of(c.attr_of("kind"), SIDE_KINDS), || format!("{w}: kind must be one of {} (got {})", SIDE_KINDS.join(", "), said(c.attr_of("kind"))))?;
                need(c.attr_of("origin").is_none() || pair(c.attr_of("origin"), 2) || pair(c.attr_of("origin"), 3), || format!("{w}: origin is where the interface's frame sits in this side's frame, [x, y] or [x, y, z] mm"))?;
                need(c.attr_of("rotation").is_none() || num(c.attr_of("rotation")), || format!("{w}: rotation is the angle the interface's frame is turned on this side, in degrees counter-clockwise"))?;
                need(c.attr_of("face").is_none() || one_of(c.attr_of("face"), SIDE_FACES), || format!("{w}: face must be one of {} (got {})", SIDE_FACES.join(", "), said(c.attr_of("face"))))?;
                need(c.attr_of("parts").map_or(true, |v| matches!(v, Json::Arr(items) if items.iter().all(|x| matches!(x, Json::Str(_))))), || format!("{w}: parts is a list of part references, whose pins are the interface's in turn"))?;
                need(c.attr_of("channel").is_none() || string(c.attr_of("channel")), || format!("{w}: channel is the id of a channel of the device definition"))?;
            }
            _ => {}
        }
    }
    let mut out = grouped(t, &[("pin", "pins"), ("hole", "holes"), ("message", "messages"), ("param", "params"), ("side", "sides")]);
    if let Some(frame) = t.single("frame") {
        // The frame comes after the root's fields, before the lists, as the TypeScript document has it.
        let Json::Obj(entries) = &mut out else { unreachable!() };
        let at = entries.iter().position(|(k, _)| !ROOT_FIELDS.contains(&k.as_str())).unwrap_or(entries.len());
        entries.insert(at, ("frame".into(), frame.record()));
    }
    Ok(out)
}
