//! A SCHEMATIC IN RUST — the circuit's own schematic sheet, declared as the nodes the CommandAGI circuit editor draws
//! and edits (`sch.symbol.*`, `sch.wire`, `sch.junction`, `sch.label`), and a netlist circuit's parts and nets. The
//! same declarations, node for node, as the TypeScript SDK's JSX schematic (`sheet.ts`) and the Python SDK's
//! (`schematic.py`). Each element is one constructor call, its attributes the methods chained on it, a group's
//! elements an array:
//!
//! ```
//! use commandagi::design::schematic::*;
//!
//! fn document() -> El {
//!     group("Divider", [
//!         voltagesource("V1").voltage("9").sch_x(114.3).sch_y(114.3),
//!         resistor("R1").resistance("3k").sch_x(114.3).sch_y(88.9),
//!         resistor("R2").resistance("1.5k").sch_x(139.7).sch_y(88.9).sch_rotation(90),
//!         ground("#PWR1").sch_x(139.7).sch_y(114.3),
//!         trace(".V1 > .pos", ".R1 > .pin1"),
//!         netlabel("OUT", ".R1 > .pin2"),
//!     ])
//! }
//! # commandagi::design::declare(document()).unwrap();
//! ```
//!
//! The elements:
//!
//! ```text
//! group(name, [ … ])                                                        the schematic (the root)
//! resistor|capacitor|inductor(name).resistance|capacitance|inductance(…)   an ideal two-terminal part
//! voltagesource|currentsource(name).voltage|current(…).excitation(json(…)) an ideal source: pin 1 (`pos`) is +
//! ground(name)                                                              a ground symbol (name "#PWR1"): net GND
//! … .sch_x(…).sch_y(…).sch_rotation(…).sch_mirror(…)                        where its symbol sits on the sheet
//! junction(name).sch_x(…).sch_y(…)                                          a wire vertex
//! trace(from, to)                                                           a wire between two selectors:
//!                                                                           ".R1 > .pin2", ".J1" (a junction),
//!                                                                           "net.GND" (a label naming that net);
//!                                                                           `.path([from, …, to])`: wires through
//!                                                                           more selectors
//! netlabel(net, connection)                                                 a label naming the net of a pin
//! part(name).symbol("Lib:Name").library("x.kicad_sym").value(…)            a library part: its symbol named by its
//!                                                                           library ref in a .kicad_sym, by path
//! unit(part, n).sch_x(…).sch_y(…)                                           where unit n (2 or more) of a part sits;
//!                                                                           the part's own sch_x, sch_y place unit 1
//! code(name).source(path).inputs(json(…))                                   a code part: the parts another file
//!                                                                           declares, run with these inputs
//! part(name).pins(["1", "2"]).value(…).symbol(…)                            a part of a netlist circuit (no sheet):
//!                                                                           its pins by number, or
//!                                                                           json(r#"[{"number": "1", "name": "VBAT"}]"#)
//! net(name).pins([".C1 > .pin1", ".U1 > .pin6"])                           a stored net of a netlist circuit
//! … .spice(json(r#"[{"id", "model", "terminals"}]"#))                       on any part: the vendor SPICE packages it
//!                                                                           binds, each formal by pin NUMBER
//! attachment(name).role("schematic").file(path).mime(…)                     a KiCad file the circuit carries
//! fragment(iter)                                                            several elements as one item (a loop)
//! ```
//!
//! `sch_x` and `sch_y` are the sheet's own coordinates: millimetres, Y DOWN. `sch_rotation` is 0, 90, 180 or 270
//! degrees; `sch_mirror` is "x" or "y". A part with neither is declared and not placed. A library part's pins are the
//! library's: the editor reads them, and binds a pin named by number (".U1 > .pin5") to the unit that has it. A wire
//! is a binding between two pins, never a coincidence of coordinates. Each node carries the call it came from in
//! `meta.source` (`super::source`), so the circuit editor writes its edits back into the file. What the sheet cannot
//! say is refused by name, never guessed.

use super::element::{attributes, elements, rust_name};
use super::ir::{channels, fnv1a, slug, wire, Scope};
use super::source::meta;
use super::{Declared, Family};
pub use super::{fragment, json, El, Json, Value};
use std::collections::HashMap;

elements! {
    /// The schematic: its name and its elements.
    group(name): children;
    /// An ideal resistor: `resistor("R1").resistance("3k")`.
    resistor(name): leaf;
    /// An ideal capacitor: `capacitor("C1").capacitance("100n")`.
    capacitor(name): leaf;
    /// An ideal inductor: `inductor("L1").inductance("10u")`.
    inductor(name): leaf;
    /// An ideal voltage source: `voltagesource("V1").voltage("9")`; pin 1 (`pos`) is +.
    voltagesource(name): leaf;
    /// An ideal current source: `currentsource("I1").current("1m")`; pin 1 (`pos`) is +.
    currentsource(name): leaf;
    /// A ground symbol (its name starts with #, as KiCad names power symbols): its net is GND.
    ground(name): leaf;
    /// A wire vertex at (`sch_x`, `sch_y`).
    junction(name): leaf;
    /// A wire between two selectors: `trace(".V1 > .pos", ".R1 > .pin1")`.
    trace(from, to): leaf;
    /// A label naming the net of a pin: `netlabel("OUT", ".R1 > .pin2")`.
    netlabel(net, connection): leaf;
    /// A library part (`.symbol("Lib:Name").library("x.kicad_sym")`), or a netlist circuit's part (`.pins([…])`).
    part(name): leaf;
    /// Where unit `unit` (2 or more) of a part sits: `unit("U1", 2).sch_x(101.6).sch_y(25.4)`.
    unit(part, unit): leaf;
    /// A code part: `code("blinker").source("blinker.circuit.ts").inputs(json(r#"{"resistor": "330"}"#))`.
    code(name): leaf;
    /// A stored net of a netlist circuit: `net("VCC").pins([".C1 > .pin1", ".U1 > .pin8"])`.
    net(name): leaf;
    /// A KiCad file the circuit carries for exchange: `attachment("Divider.kicad_sch").role("schematic").file(…)`.
    attachment(name): leaf;
}

attributes! {
    /// The schematic's attributes, chained on its elements.
    pub trait SchematicAttrs {
        /// A resistor's value: `"3k"` or `3000`.
        resistance;
        /// A capacitor's value: `"100n"`.
        capacitance;
        /// An inductor's value: `"10u"`.
        inductance;
        /// A voltage source's value: `"9"`.
        voltage;
        /// A current source's value: `"1m"`.
        current;
        /// A source's excitation for the simulator: `.excitation(json(r#"{"acMagnitude": 1}"#))`.
        excitation;
        /// Where the symbol sits, across: millimetres.
        sch_x;
        /// Where the symbol sits, down: millimetres (Y down).
        sch_y;
        /// The symbol's rotation: 0, 90, 180 or 270 degrees.
        sch_rotation;
        /// The symbol's mirror: "x" or "y".
        sch_mirror;
        /// A part's symbol, by its library ref: `"Device:R_Small"`.
        symbol;
        /// The `.kicad_sym` that holds a library part's symbol, by its path relative to this file.
        library;
        /// A library or netlist part's value: `"LM358"`.
        value;
        /// The file a code part runs, by its path relative to this file.
        source;
        /// A code part's inputs (its file's parameters): `.inputs(json(r#"{"resistor": "330"}"#))`.
        inputs;
        /// A netlist part's pins (`["1", "2"]`), or a net's (`[".C1 > .pin1"]`).
        pins;
        /// The SPICE packages a part binds: `json(r#"[{"id": "q", "model": "2N3904", "terminals": {"C": "1"}}]"#)`.
        spice;
        /// A trace's whole path of selectors, from `from` to `to`.
        path;
        /// What an attachment is: "schematic", "project", "library" or "other".
        role;
        /// The file an attachment carries, by its path relative to this file.
        file;
        /// An attachment's media type.
        mime;
    }
}

pub(crate) const FAMILY: Family = Family { module: "schematic", roots: &["group"], declare };

pub const SCH_WIRE: &str = "sch.wire";
pub const SCH_JUNCTION: &str = "sch.junction";
pub const SCH_LABEL: &str = "sch.label";
/// A stored net (a netlist circuit's): its pins on channel `pins`.
pub const NET: &str = "eda.net";
/// A file the circuit carries for exchange (the circuit editor reads its text from `file`).
pub const ATTACHMENT: &str = "eda.source";
const ROLES: &[&str] = &["schematic", "project", "library", "other"];
const PART_PORT: &str = "part";
const PART_BODY_PORT: &str = "@part";
const VERTEX_PORT: &str = "v";
const PLACE: &[&str] = &["schX", "schY", "schRotation", "schMirror"];
/// A library part's wire end before the editor reads its library: the part node, and the pin by number.
pub const LIBRARY_PIN_PORT: &str = "pin:";
/// The node a code part declares (the graph's own `code` op): it runs another file.
const CODE_OP: &str = "code";

/// A placement's node type: `sch.symbol.` + FNV-1a over its pin sockets joined by NUL (part of the format).
pub fn sch_symbol_type_for(sockets: &[&str]) -> String {
    format!("sch.symbol.{}", fnv1a(&sockets.join("\u{0}")))
}

/// A part's node type: `eda.part.` + FNV-1a over each pin's `id\0number`, pins joined by `\x01` (the engine's rule).
pub fn part_type_for(pins: &[(String, String)]) -> String {
    let signature: Vec<String> = pins.iter().map(|(id, number)| format!("{id}\u{0}{number}")).collect();
    format!("eda.part.{}", fnv1a(&signature.join("\u{1}")))
}

struct Ideal {
    symbol: &'static str,
    /// The attribute that holds the value, or none (a ground symbol's value is its net).
    value: Option<&'static str>,
    pins: usize,
    aliases: &'static [(&'static str, &'static str)],
    power: Option<&'static str>,
    excitation: bool,
}

const TWO: &[(&str, &str)] = &[("pin1", "1"), ("pin2", "2"), ("left", "1"), ("right", "2")];
const SOURCE: &[(&str, &str)] = &[("pin1", "1"), ("pin2", "2"), ("left", "1"), ("right", "2"), ("pos", "1"), ("neg", "2"), ("+", "1"), ("-", "2")];

/// The parts a schematic declares, by tag: the circuit editor's ideal symbols.
fn ideal(tag: &str) -> Option<Ideal> {
    let two = |symbol, value| Ideal { symbol, value: Some(value), pins: 2, aliases: TWO, power: None, excitation: false };
    let source = |symbol, value| Ideal { symbol, value: Some(value), pins: 2, aliases: SOURCE, power: None, excitation: true };
    Some(match tag {
        "resistor" => two("Ideal:R", "resistance"),
        "capacitor" => two("Ideal:C", "capacitance"),
        "inductor" => two("Ideal:L", "inductance"),
        "voltagesource" => source("Ideal:V", "voltage"),
        "currentsource" => source("Ideal:I", "current"),
        "ground" => Ideal { symbol: "Ideal:GND", value: None, pins: 1, aliases: &[("pin1", "1"), ("gnd", "1")], power: Some("GND"), excitation: false },
        _ => return None,
    })
}

/// Declare a schematic: `group(name, [ … ])` is the circuit `eda:<name>`.
fn declare(root: El) -> Result<Declared, String> {
    refuse_unknown(&root, &["name"])?;
    let name = match root.attr("name") {
        Some(Json::Str(n)) if !n.is_empty() => n.clone(),
        _ => return Err("group() needs a name".into()),
    };
    let mut s = Scope::new(&format!("eda:{}", slug(&name)), Json::obj().with("domain", "eda").with("rung", "board").with("name", name.as_str()));
    declare_sheet(&mut s, &root.child_elements().collect::<Vec<_>>())?;
    Ok(Declared::Graph(s.build()))
}

struct Placed {
    reference: String,
    /// The part node's id.
    id: String,
    /// Its placements by unit.
    units: Vec<(u32, String)>,
    /// An ideal part's pin count and aliases; none for a library part or a netlist part.
    ideal: Option<(usize, &'static [(&'static str, &'static str)])>,
    /// A netlist part's pins: (id, number).
    pins: Option<Vec<(String, String)>>,
}

fn refuse_unknown(el: &El, allowed: &[&str]) -> Result<(), String> {
    match el.attrs.iter().find(|(k, _)| !allowed.contains(&k.as_str())) {
        Some((k, _)) => Err(format!("{}: {} is not read on a schematic", el.call_text(), rust_name(k))),
        None => Ok(()),
    }
}

/// A number attribute: mm or degrees.
fn num(el: &El, prop: &str) -> Result<Option<f64>, String> {
    match el.attr(prop) {
        None => Ok(None),
        Some(Json::Num(n)) => Ok(Some(*n)),
        Some(other) => Err(format!("{}: {} is a number (mm or degrees), not {}", el.call_text(), rust_name(prop), other.text())),
    }
}

/// A text attribute that must be there.
fn text(el: &El, prop: &str, what: &str) -> Result<String, String> {
    match el.attr(prop).and_then(Json::as_str) {
        Some(v) if !v.trim().is_empty() => Ok(v.to_string()),
        _ => Err(format!("{}: {} is {what}", el.call_text(), rust_name(prop))),
    }
}

/// A part's value as the sheet holds it: text as written, a number as JavaScript's `String(n)` writes it.
fn value_of(el: &El, prop: &str, example: &str) -> Result<Option<String>, String> {
    match el.attr(prop) {
        None => Ok(None),
        Some(Json::Str(s)) => Ok(Some(s.clone())),
        Some(n @ Json::Num(_)) => Ok(Some(n.text())),
        Some(other) => Err(format!("{}: {} is a value ({example}), not {}", el.call_text(), rust_name(prop), other.text())),
    }
}

/// A part's SPICE package bindings: a list of `{id, model, terminals: {formal: "pin number"}}`.
fn spice_of(el: &El) -> Result<Option<Json>, String> {
    let Some(v) = el.attr("spice") else { return Ok(None) };
    let ok = match v {
        Json::Arr(list) => list.iter().all(|b| {
            b.get("id").and_then(Json::as_str).is_some()
                && b.get("model").and_then(Json::as_str).is_some()
                && matches!(b.get("terminals"), Some(Json::Obj(t)) if t.iter().all(|(_, n)| n.as_str().is_some()))
        }),
        _ => false,
    };
    if !ok {
        return Err(format!("{}: spice is a list of {{ id, model, terminals: {{ formal: \"pin number\" }} }}", el.call_text()));
    }
    Ok(Some(v.clone()))
}

/// Add the placement of `unit` of `part` that `el` declares (its sch_x, sch_y, sch_rotation, sch_mirror).
fn place(s: &mut Scope, part: &mut Placed, unit: u32, el: &El, sockets: &[&str]) -> Result<(), String> {
    let (x, y, rot) = (num(el, "schX")?, num(el, "schY")?, num(el, "schRotation")?);
    let mirror = el.attr("schMirror");
    if x.is_none() && y.is_none() {
        if rot.is_some() || mirror.is_some() {
            let what = if rot.is_some() { "sch_rotation" } else { "sch_mirror" };
            return Err(format!("{}: {what} needs sch_x and sch_y", el.call_text()));
        }
        if el.tag == "unit" {
            return Err(format!("{}: a unit is placed: give it sch_x and sch_y", el.call_text()));
        }
        return Ok(());
    }
    if let Some(r) = rot {
        if ![0.0, 90.0, 180.0, 270.0].contains(&r) {
            return Err(format!("{}: sch_rotation is 0, 90, 180 or 270", el.call_text()));
        }
    }
    let mirror = match mirror {
        None => String::new(),
        Some(Json::Str(m)) if m == "x" || m == "y" => m.clone(),
        Some(_) => return Err(format!("{}: sch_mirror is \"x\" or \"y\"", el.call_text())),
    };
    let inputs = vec![
        ("unit".to_string(), Json::Num(unit as f64)),
        ("style".into(), Json::Num(1.0)),
        ("at".into(), Json::obj().with("x", x.unwrap_or(0.0)).with("y", y.unwrap_or(0.0))),
        ("rot".into(), Json::Num(rot.unwrap_or(0.0))),
        ("mirror".into(), Json::from(mirror)),
        (PART_PORT.into(), wire(&part.id, PART_BODY_PORT)),
    ];
    let reference = part.reference.clone();
    let id = s.add(&sch_symbol_type_for(sockets), inputs, Some(&format!("sym_{reference}_{unit}")), Some(&reference), meta(el))?;
    part.units.push((unit, id));
    Ok(())
}

fn pins_json(pins: &[(String, String, Option<String>)]) -> Json {
    Json::Arr(
        pins.iter()
            .map(|(id, n, name)| {
                let p = Json::obj().with("id", id.as_str()).with("number", n.as_str());
                match name {
                    Some(name) => p.with("name", name.as_str()),
                    None => p,
                }
            })
            .collect(),
    )
}

fn declare_sheet(s: &mut Scope, children: &[&El]) -> Result<(), String> {
    let mut parts: HashMap<String, Placed> = HashMap::new();
    let mut junctions: HashMap<String, String> = HashMap::new();
    let mut later: Vec<&El> = Vec::new();
    for &el in children {
        if let Some(ideal) = ideal(el.tag) {
            let mut allowed = [&["name", "spice"], PLACE].concat();
            allowed.extend(ideal.value);
            if ideal.excitation {
                allowed.push("excitation");
            }
            refuse_unknown(el, &allowed)?;
            let reference = match el.attr("name") {
                Some(Json::Str(n)) if !n.is_empty() => n.clone(),
                _ => return Err(format!("{}() needs a name", el.tag)),
            };
            // The netlist leaves power symbols out by their reference (KiCad's rule), so a ground's says it is one.
            if ideal.power.is_some() && !reference.starts_with('#') {
                return Err(format!("{}: a ground symbol's name starts with # (#PWR1), as KiCad names power symbols", el.call_text()));
            }
            if parts.contains_key(&reference) {
                return Err(format!("two parts are called {reference}"));
            }
            let pins: Vec<(String, String)> = (1..=ideal.pins).map(|i| (format!("p{i}"), i.to_string())).collect();
            let raw = match ideal.value {
                Some(v) => value_of(el, v, "\"1k\", 1000")?,
                None => None,
            };
            let value = ideal.power.map(str::to_string).or(raw);
            let mut inputs = vec![("ref".to_string(), Json::from(reference.as_str()))];
            if let Some(value) = value {
                inputs.push(("value".into(), value.into()));
            }
            inputs.push(("symbol".into(), ideal.symbol.into()));
            inputs.push(("pins".into(), pins_json(&pins.iter().map(|(i, n)| (i.clone(), n.clone(), None)).collect::<Vec<_>>())));
            inputs.push(("units".into(), Json::Num(1.0)));
            if ideal.power.is_some() {
                inputs.push(("powerSymbol".into(), true.into()));
            }
            if let Some(excitation) = el.attr("excitation") {
                inputs.push(("excitation".into(), excitation.clone()));
            }
            if let Some(spice) = spice_of(el)? {
                inputs.push(("spice".into(), spice));
            }
            let part = s.add(&part_type_for(&pins), inputs, Some(&reference), Some(&reference), meta(el))?;
            let mut placed = Placed { reference: reference.clone(), id: part, units: Vec::new(), ideal: Some((ideal.pins, ideal.aliases)), pins: None };
            let sockets: Vec<&str> = pins.iter().map(|(id, _)| id.as_str()).collect();
            place(s, &mut placed, 1, el, &sockets)?;
            parts.insert(reference, placed);
            continue;
        }
        match el.tag {
            "part" => {
                refuse_unknown(el, &[&["name", "symbol", "library", "value", "pins", "spice"], PLACE].concat())?;
                let reference = text(el, "name", "the part's reference (U1)")?;
                if parts.contains_key(&reference) {
                    return Err(format!("two parts are called {reference}"));
                }
                if el.attr("pins").is_some() {
                    let placed = netlist_part(s, el, &reference)?;
                    parts.insert(reference, placed);
                    continue;
                }
                let symbol = text(el, "symbol", "its library ref (\"Device:R_Small\")")?;
                let mut halves = symbol.split(':');
                if !(halves.next().is_some_and(|l| !l.is_empty()) && halves.next().is_some_and(|n| !n.is_empty()) && halves.next().is_none()) {
                    return Err(format!("{}: symbol is a library ref, \"Library:Symbol\" (\"Device:R_Small\"), not {symbol:?}", el.call_text()));
                }
                let library = text(el, "library", "the path of the .kicad_sym that holds the symbol")?;
                if !library.to_lowercase().ends_with(".kicad_sym") {
                    return Err(format!("{}: library names a .kicad_sym file, not {library:?}", el.call_text()));
                }
                let value = value_of(el, "value", "\"LM358\", 1000")?;
                // The pins are the library's: the editor reads them (and the part's type) from the library file.
                let mut inputs = vec![("ref".to_string(), Json::from(reference.as_str()))];
                if let Some(v) = value {
                    inputs.push(("value".into(), v.into()));
                }
                inputs.push(("symbol".into(), symbol.as_str().into()));
                inputs.push(("library".into(), library.as_str().into()));
                inputs.push(("pins".into(), Json::Arr(Vec::new())));
                if let Some(spice) = spice_of(el)? {
                    inputs.push(("spice".into(), spice));
                }
                let part = s.add(&part_type_for(&[]), inputs, Some(&reference), Some(&reference), meta(el))?;
                let mut placed = Placed { reference: reference.clone(), id: part, units: Vec::new(), ideal: None, pins: None };
                place(s, &mut placed, 1, el, &[])?;
                parts.insert(reference, placed);
            }
            "code" => {
                refuse_unknown(el, &["name", "source", "inputs"])?;
                let name = text(el, "name", "the code part's id")?;
                let source = text(el, "source", "the path of the file it runs, relative to this one")?;
                let mut inputs = vec![("source".to_string(), Json::from(source.as_str()))];
                match el.attr("inputs") {
                    None => {}
                    Some(Json::Obj(entries)) => {
                        if entries.iter().any(|(k, _)| k == "source") {
                            return Err(format!("{}: source is the file, not one of its inputs", el.call_text()));
                        }
                        inputs.extend(entries.iter().cloned());
                    }
                    Some(_) => return Err(format!("{}: inputs is an object of the file's parameters", el.call_text())),
                }
                if s.has(&slug(&name)) {
                    return Err(format!("two nodes are called {name}"));
                }
                let label = source.rsplit('/').next().unwrap_or(&source).to_string();
                s.add(CODE_OP, inputs, Some(&name), Some(&label), meta(el))?;
            }
            "junction" => {
                refuse_unknown(el, &["name", "schX", "schY"])?;
                let name = match el.attr("name") {
                    Some(Json::Str(n)) if !n.is_empty() => n.clone(),
                    _ => return Err("junction() needs a name".into()),
                };
                if junctions.contains_key(&name) {
                    return Err(format!("two junctions are called {name}"));
                }
                let at = Json::obj().with("x", num(el, "schX")?.unwrap_or(0.0)).with("y", num(el, "schY")?.unwrap_or(0.0));
                let id = s.add(SCH_JUNCTION, vec![("at".into(), at)], Some(&name), Some("Junction"), meta(el))?;
                junctions.insert(name, id);
            }
            "unit" | "trace" | "netlabel" | "net" | "attachment" => later.push(el),
            _ => return Err(format!("{} is not read on a schematic (see commandagi::design::schematic)", el.call_text())),
        }
    }

    // Units first: a wire may land on any unit's pin.
    for el in later.iter().filter(|el| el.tag == "unit") {
        refuse_unknown(el, &[&["part", "unit"], PLACE].concat())?;
        let reference = text(el, "part", "the reference of the part whose unit it places")?;
        let part = parts.get_mut(&reference).ok_or_else(|| format!("{}: there is no part {reference}", el.call_text()))?;
        let n = match el.attr("unit") {
            Some(Json::Num(n)) if n.fract() == 0.0 && *n >= 2.0 => *n as u32,
            _ => return Err(format!("{}: unit is 2 or more (the part's own sch_x and sch_y place unit 1)", el.call_text())),
        };
        if part.ideal.is_some() {
            return Err(format!("{}: {reference} is an ideal part, which has one unit", el.call_text()));
        }
        if part.units.iter().any(|(u, _)| *u == n) {
            return Err(format!("{}: unit {n} of {reference} is placed twice", el.call_text()));
        }
        place(s, part, n, el, &[])?;
    }

    let end = |sel: &Json, el: &El| -> Result<End, String> {
        let Some(sel) = sel.as_str() else {
            return Err(format!("{}: an end is a selector (\".R1 > .pin1\", \".J1\", \"net.GND\")", el.call_text()));
        };
        let t = sel.trim();
        if let Some(net) = t.strip_prefix("net.") {
            if !net.is_empty() && net.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '+' | '-')) {
                return Ok(End::Net(net.to_string()));
            }
        }
        let refused = || format!("{}: {sel:?} is not \".REF > .pin\", \".JUNCTION\" or \"net.NAME\"", el.call_text());
        let body = t.strip_prefix('.').ok_or_else(refused)?;
        match body.split_once('>') {
            None if name_ok(body) => {
                let node = junctions.get(body).ok_or_else(|| format!("{}: there is no junction {body}", el.call_text()))?;
                Ok(End::Pin(node.clone(), VERTEX_PORT.to_string()))
            }
            Some((part, pin)) => {
                let (part, pin) = (part.trim_end(), pin.trim_start().strip_prefix('.').ok_or_else(refused)?);
                if !name_ok(part) || !pin_ok(pin) {
                    return Err(refused());
                }
                let p = parts.get(part).ok_or_else(|| format!("{}: there is no part {part}", el.call_text()))?;
                if p.units.is_empty() {
                    return Err(format!("{}: {} is not on the sheet (give it sch_x and sch_y)", el.call_text(), p.reference));
                }
                let Some((count, aliases)) = p.ideal else {
                    // A library part's pin by number (".pin5" or ".5"); the editor binds it to the unit that has it.
                    return Ok(End::Pin(p.id.clone(), format!("{LIBRARY_PIN_PORT}{}", pin_number(pin))));
                };
                let placement = &p.units.iter().find(|(u, _)| *u == 1).expect("an ideal part on the sheet has unit 1").1;
                let key = pin.to_lowercase();
                let number = aliases.iter().find(|(a, _)| *a == key).map(|(_, n)| n.to_string()).or_else(|| key.chars().all(|c| c.is_ascii_digit()).then(|| key.clone()));
                match number.and_then(|n| n.parse::<usize>().ok()) {
                    Some(n) if n >= 1 && n <= count => Ok(End::Pin(placement.clone(), format!("p{n}"))),
                    _ => Err(format!("{}: {} has no pin {pin}", el.call_text(), p.reference)),
                }
            }
            None => Err(refused()),
        }
    };

    let mut wires = 0;
    for el in later.into_iter().filter(|el| el.tag != "unit") {
        match el.tag {
            "attachment" => {
                refuse_unknown(el, &["name", "role", "file", "mime"])?;
                let name = text(el, "name", "the attachment's name (schematic.kicad_sch)")?;
                let role = text(el, "role", &format!("one of {}", ROLES.join(", ")))?;
                if !ROLES.contains(&role.as_str()) {
                    return Err(format!("{}: role is one of {}", el.call_text(), ROLES.join(", ")));
                }
                let file = text(el, "file", "the path of the file it carries, relative to this one")?;
                if file.starts_with('/') {
                    return Err(format!("{}: file is a path relative to this file", el.call_text()));
                }
                let mut inputs = vec![("name".to_string(), Json::from(name.as_str())), ("role".into(), role.into())];
                match el.attr("mime") {
                    None => {}
                    Some(m @ Json::Str(_)) => inputs.push(("mime".into(), m.clone())),
                    Some(_) => return Err(format!("{}: mime is a media type", el.call_text())),
                }
                inputs.push(("file".into(), file.into()));
                let id = free(s, &format!("source_{name}"));
                s.add(ATTACHMENT, inputs, Some(&id), Some(&name), meta(el))?;
            }
            "net" => {
                refuse_unknown(el, &["name", "pins"])?;
                let name = text(el, "name", "the net's name")?;
                let pins = match el.attr("pins") {
                    Some(Json::Arr(p)) if !p.is_empty() => p,
                    _ => return Err(format!("{}: pins is a list of pins (\".C1 > .pin1\")", el.call_text())),
                };
                let mut ends = Vec::new();
                for sel in pins {
                    let refused = || format!("{}: {} is not \".REF > .pin1\"", el.call_text(), sel.text());
                    let (part, pin) = sel.as_str().and_then(pin_selector).ok_or_else(refused)?;
                    let p = parts.get(part).ok_or_else(|| format!("{}: there is no part {part}", el.call_text()))?;
                    let number = pin_number(pin);
                    ends.push(match (&p.pins, p.ideal) {
                        // A library part's pin by number: the editor reads its pins from the library.
                        (None, None) => wire(&p.id, &format!("{LIBRARY_PIN_PORT}{number}")),
                        (None, Some(_)) => return Err(format!("{}: {} is an ideal part drawn on the sheet; a net names pins of a netlist circuit's parts", el.call_text(), p.reference)),
                        (Some(list), _) => {
                            let (id, _) = list.iter().find(|(_, n)| n == number).ok_or_else(|| format!("{}: {} has no pin {number}", el.call_text(), p.reference))?;
                            wire(&p.id, id)
                        }
                    });
                }
                let id = free(s, &format!("net_{name}"));
                s.add(NET, channels("pins", ends), Some(&id), Some(&name), meta(el))?;
            }
            "netlabel" => {
                refuse_unknown(el, &["net", "connection"])?;
                let text = el.attr("net").and_then(Json::as_str).map(str::trim).filter(|t| !t.is_empty()).ok_or("netlabel() needs a net name")?.to_string();
                match end(el.attr("connection").unwrap_or(&Json::Null), el)? {
                    End::Net(_) => return Err(format!("{}: a label's connection is a pin or a junction", el.call_text())),
                    End::Pin(node, port) => label(s, &text, &node, &port, el)?,
                }
            }
            _ => {
                refuse_unknown(el, &["from", "to", "path"])?;
                let (from, to) = (el.attr("from").cloned().unwrap_or(Json::Null), el.attr("to").cloned().unwrap_or(Json::Null));
                let path = match el.attr("path") {
                    None => vec![from, to],
                    // The path is the whole path: it starts at `from` and ends at `to`, so no argument is ignored.
                    Some(Json::Arr(p)) if p.len() >= 2 && p.first() == Some(&from) && p.last() == Some(&to) => p.clone(),
                    Some(_) => return Err(format!("{}: path is the list of selectors from {} to {}", el.call_text(), from.text(), to.text())),
                };
                let ends = path.iter().map(|p| end(p, el)).collect::<Result<Vec<_>, _>>()?;
                for pair in ends.windows(2) {
                    match (&pair[0], &pair[1]) {
                        (End::Net(a), End::Net(b)) => return Err(format!("trace() joins two nets ({a}, {b})")),
                        (End::Net(net), End::Pin(node, port)) | (End::Pin(node, port), End::Net(net)) => label(s, net, node, port, el)?,
                        (End::Pin(an, ap), End::Pin(bn, bp)) => {
                            wires += 1;
                            let id = free(s, &format!("w_{wires}"));
                            s.add(SCH_WIRE, channels("ends", vec![wire(an, ap), wire(bn, bp)]), Some(&id), Some("Wire"), meta(el))?;
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

/// A part of a netlist circuit: its pins listed (by number, with a name or not), no symbol drawing, no placement.
fn netlist_part(s: &mut Scope, el: &El, reference: &str) -> Result<Placed, String> {
    for p in PLACE.iter().chain(["library"].iter()) {
        if el.attr(p).is_some() {
            let what = if *p == "library" { "library" } else { "place on the sheet" };
            return Err(format!("{}: a part that lists its pins is a netlist circuit's; it has no {what}", el.call_text()));
        }
    }
    let raw = match el.attr("pins") {
        Some(Json::Arr(p)) if !p.is_empty() => p,
        _ => return Err(format!("{}: pins is a list of pin numbers ([\"1\", \"2\"]) or of {{ number, name }}", el.call_text())),
    };
    // A pin number may repeat (a power pin each unit of a part shares, a mechanical pad with no number).
    let mut pins = Vec::new();
    for (i, p) in raw.iter().enumerate() {
        let refused = || format!("{}: pin {i} is a number (\"1\") or {{ number, name }}, not {}", el.call_text(), p.text());
        let (number, name) = match p {
            Json::Str(n) => (n.clone(), None),
            Json::Obj(entries) if entries.iter().all(|(k, _)| k == "number" || k == "name") => {
                let number = p.get("number").and_then(Json::as_str).ok_or_else(refused)?.to_string();
                let name = match p.get("name") {
                    None => None,
                    Some(Json::Str(n)) => Some(n.clone()),
                    Some(_) => return Err(refused()),
                };
                (number, name)
            }
            _ => return Err(refused()),
        };
        pins.push((format!("p{}", i + 1), number, name));
    }
    let symbol = match el.attr("symbol") {
        None => None,
        Some(Json::Str(sym)) if !sym.is_empty() => Some(sym.clone()),
        Some(_) => return Err(format!("{}: symbol is a library ref (\"Device:R\")", el.call_text())),
    };
    let value = value_of(el, "value", "\"LM358\", 1000")?;
    let mut inputs = vec![("ref".to_string(), Json::from(reference))];
    if let Some(v) = value {
        inputs.push(("value".into(), v.into()));
    }
    if let Some(sym) = symbol {
        inputs.push(("symbol".into(), sym.into()));
    }
    inputs.push(("pins".into(), pins_json(&pins)));
    if let Some(spice) = spice_of(el)? {
        inputs.push(("spice".into(), spice));
    }
    let typed: Vec<(String, String)> = pins.iter().map(|(id, n, _)| (id.clone(), n.clone())).collect();
    let id = s.add(&part_type_for(&typed), inputs, Some(reference), Some(reference), meta(el))?;
    Ok(Placed { reference: reference.to_string(), id, units: Vec::new(), ideal: None, pins: Some(typed) })
}

enum End {
    Net(String),
    Pin(String, String),
}

fn name_ok(n: &str) -> bool {
    !n.is_empty() && n.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '#' | '-'))
}

fn pin_ok(n: &str) -> bool {
    !n.is_empty() && n.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '+' | '-'))
}

/// `.REF > .pin` → (REF, pin).
fn pin_selector(sel: &str) -> Option<(&str, &str)> {
    let (part, pin) = sel.trim().strip_prefix('.')?.split_once('>')?;
    let (part, pin) = (part.trim_end(), pin.trim_start().strip_prefix('.')?);
    (name_ok(part) && pin_ok(pin)).then_some((part, pin))
}

/// A pin by number: `pin5` or `5` → `5`.
fn pin_number(pin: &str) -> &str {
    match pin.get(..3) {
        Some(head) if head.eq_ignore_ascii_case("pin") && pin.len() > 3 => &pin[3..],
        _ => pin,
    }
}

/// A free id `base`, `base_2`, … (the same file declares the same ids).
fn free(s: &Scope, base: &str) -> String {
    let b = slug(base);
    let (mut i, mut n) = (b.clone(), 2);
    while s.has(&i) {
        i = format!("{b}_{n}");
        n += 1;
    }
    i
}

fn label(s: &mut Scope, text: &str, node: &str, port: &str, el: &El) -> Result<(), String> {
    let id = free(s, &format!("lbl_{text}"));
    let inputs = vec![("text".to_string(), Json::from(text)), ("on".into(), wire(node, port))];
    s.add(SCH_LABEL, inputs, Some(&id), Some(text), meta(el)).map(|_| ())
}
