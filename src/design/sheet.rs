//! A SCHEMATIC IN RUST — the circuit's own schematic sheet, declared as the nodes the CommandAGI circuit editor draws
//! and edits (`sch.symbol.*`, `sch.wire`, `sch.junction`, `sch.label`). The same declarations, node for node, as the
//! TypeScript SDK's JSX schematic (`sheet.ts`) and the Python SDK's (`sheet.py`). Each element is one constructor call,
//! its attributes the methods chained on it, a group's elements an array:
//!
//! ```
//! use commandagi::design::sheet::*;
//!
//! fn document() -> Group {
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
//! resistor|capacitor|inductor(name).resistance|capacitance|inductance(…)   an ideal two-terminal part
//! voltagesource|currentsource(name).voltage|current(…).excitation(json)    an ideal source: pin 1 (`pos`) is +
//! ground(name)                                                              a ground symbol (name "#PWR1"): net GND
//! … .sch_x(…).sch_y(…).sch_rotation(…)                                      where its symbol sits on the sheet
//! junction(name).sch_x(…).sch_y(…)                                          a wire vertex
//! trace(from, to)                                                           a wire between two selectors:
//!                                                                           ".R1 > .pin2", ".J1" (a junction),
//!                                                                           "net.GND" (a label naming that net)
//! netlabel(net, connection)                                                 a label naming the net of a pin
//! part(name).symbol("Lib:Name").library("x.kicad_sym").value(…)            a library part: its symbol named by its
//!                                                                           library ref in a .kicad_sym, by path
//! unit(part, n).sch_x(…).sch_y(…)                                           where unit n (2 or more) of a part sits;
//!                                                                           the part's own sch_x, sch_y place unit 1
//! code(name).source(path).inputs(json)                                      a code part: the parts another file
//!                                                                           declares, run with these inputs
//! fragment(iter)                                                            several elements as one item (a loop)
//! ```
//!
//! `sch_x` and `sch_y` are the sheet's own coordinates: millimetres, Y DOWN. `sch_rotation` is 0, 90, 180 or 270
//! degrees; `sch_mirror` is "x" or "y". A part with neither is declared and not placed. A library part's pins are the
//! library's: the editor reads them, and binds a pin named by number (".U1 > .pin5") to the unit that has it. A wire is a binding between two pins, never a coincidence
//! of coordinates. Each node carries the call it came from in `meta.source` (`super::source`), so the circuit editor
//! writes its edits back into the file. What the sheet cannot say is refused by name, never guessed.

use super::ir::{channels, fnv1a, slug, wire, Scope};
use super::json::Json;
use super::source::{here, meta, Site};
use super::Document;
use std::collections::HashMap;

pub const SCH_WIRE: &str = "sch.wire";
pub const SCH_JUNCTION: &str = "sch.junction";
pub const SCH_LABEL: &str = "sch.label";
const PART_PORT: &str = "part";
const PART_BODY_PORT: &str = "@part";
const VERTEX_PORT: &str = "v";
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

/// The parts a schematic declares, by constructor: the circuit editor's ideal symbols.
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

/// A part's value: text (`"3k"`) or a number (`1000`).
pub struct PartValue(Json);
impl From<&str> for PartValue {
    fn from(s: &str) -> Self {
        PartValue(Json::from(s))
    }
}
impl From<String> for PartValue {
    fn from(s: String) -> Self {
        PartValue(Json::from(s))
    }
}
impl From<i32> for PartValue {
    fn from(n: i32) -> Self {
        PartValue(Json::from(n as f64))
    }
}
impl From<f64> for PartValue {
    fn from(n: f64) -> Self {
        PartValue(Json::from(n))
    }
}

/// One element of a sheet: a part, a junction, a trace or a net label. Its attributes are the methods chained on it.
#[derive(Clone, Debug)]
pub struct El {
    tag: &'static str,
    name: Option<String>,
    props: Vec<(&'static str, Json)>,
    path: Vec<String>,
    site: Option<Site>,
    /// A fragment's elements.
    children: Vec<El>,
}

impl El {
    fn new(tag: &'static str, name: Option<String>, site: Site) -> El {
        El { tag, name, props: Vec::new(), path: Vec::new(), site: Some(site), children: Vec::new() }
    }

    fn prop(mut self, name: &'static str, value: Json) -> El {
        self.props.retain(|(k, _)| *k != name);
        self.props.push((name, value));
        self
    }

    fn get(&self, name: &str) -> Option<&Json> {
        self.props.iter().find(|(k, _)| *k == name).map(|(_, v)| v)
    }

    fn at(&self) -> String {
        match &self.name {
            Some(n) => format!("{}({n:?})", self.tag),
            None => format!("{}()", self.tag),
        }
    }

    pub fn resistance(self, value: impl Into<PartValue>) -> El {
        self.prop("resistance", value.into().0)
    }
    pub fn capacitance(self, value: impl Into<PartValue>) -> El {
        self.prop("capacitance", value.into().0)
    }
    pub fn inductance(self, value: impl Into<PartValue>) -> El {
        self.prop("inductance", value.into().0)
    }
    pub fn voltage(self, value: impl Into<PartValue>) -> El {
        self.prop("voltage", value.into().0)
    }
    pub fn current(self, value: impl Into<PartValue>) -> El {
        self.prop("current", value.into().0)
    }
    /// A source's excitation for the simulator, as JSON text: `.excitation(r#"{"acMagnitude": 1}"#)`.
    pub fn excitation(self, json: &str) -> El {
        let value = Json::parse(json).unwrap_or_else(|e| Json::Str(format!("\u{0}{e}")));
        self.prop("excitation", value)
    }
    /// Where the symbol sits, across: millimetres.
    pub fn sch_x(self, mm: impl Into<f64>) -> El {
        self.prop("sch_x", Json::Num(mm.into()))
    }
    /// Where the symbol sits, down: millimetres (Y down).
    pub fn sch_y(self, mm: impl Into<f64>) -> El {
        self.prop("sch_y", Json::Num(mm.into()))
    }
    /// The symbol's rotation: 0, 90, 180 or 270 degrees.
    pub fn sch_rotation(self, degrees: impl Into<f64>) -> El {
        self.prop("sch_rotation", Json::Num(degrees.into()))
    }
    /// The symbol's mirror: "x" or "y".
    pub fn sch_mirror(self, axis: &str) -> El {
        self.prop("sch_mirror", Json::from(axis))
    }
    /// A library part's symbol, by its library ref: `"Device:R_Small"`.
    pub fn symbol(self, library_ref: &str) -> El {
        self.prop("symbol", Json::from(library_ref))
    }
    /// The `.kicad_sym` that holds a library part's symbol, by its path relative to this file.
    pub fn library(self, path: &str) -> El {
        self.prop("library", Json::from(path))
    }
    /// A library part's value: `"LM358"`.
    pub fn value(self, value: impl Into<PartValue>) -> El {
        self.prop("value", value.into().0)
    }
    /// The file a code part runs, by its path relative to this file.
    pub fn source(self, path: &str) -> El {
        self.prop("source", Json::from(path))
    }
    /// A code part's inputs (its file's parameters), as JSON text of an object: `.inputs(r#"{"resistor": "330"}"#)`.
    pub fn inputs(self, json: &str) -> El {
        let value = Json::parse(json).unwrap_or_else(|e| Json::Str(format!("\u{0}{e}")));
        self.prop("inputs", value)
    }
    /// A trace's path goes on to one more selector (each step is one wire).
    pub fn then(mut self, selector: impl Into<String>) -> El {
        self.path.push(selector.into());
        self
    }
}

macro_rules! parts {
    ($($tag:ident: $doc:literal),*) => {$(
        #[doc = $doc]
        #[track_caller]
        pub fn $tag(name: impl Into<String>) -> El {
            El::new(stringify!($tag), Some(name.into()), here())
        }
    )*};
}
parts! {
    resistor: "An ideal resistor: `resistor(\"R1\").resistance(\"3k\")`.",
    capacitor: "An ideal capacitor: `capacitor(\"C1\").capacitance(\"100n\")`.",
    inductor: "An ideal inductor: `inductor(\"L1\").inductance(\"10u\")`.",
    voltagesource: "An ideal voltage source: `voltagesource(\"V1\").voltage(\"9\")`; pin 1 (`pos`) is +.",
    currentsource: "An ideal current source: `currentsource(\"I1\").current(\"1m\")`; pin 1 (`pos`) is +.",
    ground: "A ground symbol (its name starts with #, as KiCad names power symbols): its net is GND.",
    junction: "A wire vertex at (`sch_x`, `sch_y`).",
    part: "A library part: `part(\"U1\").symbol(\"Amplifier_Operational:LM358\").library(\"opamps.kicad_sym\")`.",
    code: "A code part: `code(\"blinker\").source(\"blinker.circuit.ts\").inputs(r#\"{\"resistor\": \"330\"}\"#)`."
}

/// Where unit `n` (2 or more) of a part sits: `unit("U1", 2).sch_x(101.6).sch_y(25.4)`.
#[track_caller]
pub fn unit(part: impl Into<String>, n: u32) -> El {
    El::new("unit", Some(part.into()), here()).prop("unit", Json::Num(n as f64))
}

/// A wire between two selectors: `trace(".V1 > .pos", ".R1 > .pin1")`.
#[track_caller]
pub fn trace(from: impl Into<String>, to: impl Into<String>) -> El {
    let mut el = El::new("trace", None, here());
    el.path = vec![from.into(), to.into()];
    el
}

/// A label naming the net of a pin: `netlabel("OUT", ".R1 > .pin2")`.
#[track_caller]
pub fn netlabel(net: impl Into<String>, connection: impl Into<String>) -> El {
    El::new("netlabel", None, here()).prop("net", Json::Str(net.into())).prop("connection", Json::Str(connection.into()))
}

/// Several elements as one item of a group's array, React's fragment: `fragment((1..=4).map(|i| resistor(…)))`.
/// The elements a closure or a loop makes are the code's to edit; the array around them stays the editor's.
pub fn fragment(children: impl IntoIterator<Item = El>) -> El {
    El { tag: "fragment", name: None, props: Vec::new(), path: Vec::new(), site: None, children: children.into_iter().collect() }
}

/// The schematic: its name and its elements (an array, or anything that iterates over elements).
pub struct Group {
    name: String,
    children: Vec<El>,
}

#[track_caller]
pub fn group(name: impl Into<String>, children: impl IntoIterator<Item = El>) -> Group {
    here();
    fn flat(els: impl IntoIterator<Item = El>, out: &mut Vec<El>) {
        for el in els {
            if el.tag == "fragment" {
                flat(el.children, out);
            } else {
                out.push(el);
            }
        }
    }
    let mut flattened = Vec::new();
    flat(children, &mut flattened);
    Group { name: name.into(), children: flattened }
}

impl Document for Group {
    fn declare(self) -> Result<Json, String> {
        if self.name.is_empty() {
            return Err("group() needs a name".into());
        }
        let mut s = Scope::new(&format!("eda:{}", slug(&self.name)), Json::obj().with("domain", "eda").with("rung", "board").with("name", self.name.as_str()));
        declare_sheet(&mut s, &self.children)?;
        Ok(s.build())
    }
}

struct Placed {
    reference: String,
    /// The part node's id.
    id: String,
    /// Its placements by unit.
    units: Vec<(u32, String)>,
    /// An ideal part's pin count and aliases; none for a library part (its pins are the library's).
    ideal: Option<(usize, &'static [(&'static str, &'static str)])>,
}

/// Add the placement of `unit` of `part` that `el` declares (its sch_x, sch_y, sch_rotation, sch_mirror).
fn place(s: &mut Scope, part: &mut Placed, unit: u32, el: &El, sockets: &[&str]) -> Result<(), String> {
    let (x, y, rot) = (num(el, "sch_x"), num(el, "sch_y"), num(el, "sch_rotation"));
    let mirror = el.get("sch_mirror").map(|m| m.as_str().unwrap_or("").to_string());
    if x.is_none() && y.is_none() {
        if rot.is_some() || mirror.is_some() {
            let what = if rot.is_some() { "sch_rotation" } else { "sch_mirror" };
            return Err(format!("{}: {what} needs sch_x and sch_y", el.at()));
        }
        if el.tag == "unit" {
            return Err(format!("{}: a unit is placed: give it sch_x and sch_y", el.at()));
        }
        return Ok(());
    }
    if let Some(r) = rot {
        if ![0.0, 90.0, 180.0, 270.0].contains(&r) {
            return Err(format!("{}: sch_rotation is 0, 90, 180 or 270", el.at()));
        }
    }
    if let Some(m) = &mirror {
        if m != "x" && m != "y" {
            return Err(format!("{}: sch_mirror is \"x\" or \"y\"", el.at()));
        }
    }
    let inputs = vec![
        ("unit".to_string(), Json::Num(unit as f64)),
        ("style".into(), Json::Num(1.0)),
        ("at".into(), Json::obj().with("x", x.unwrap_or(0.0)).with("y", y.unwrap_or(0.0))),
        ("rot".into(), Json::Num(rot.unwrap_or(0.0))),
        ("mirror".into(), Json::from(mirror.unwrap_or_default().as_str())),
        (PART_PORT.into(), wire(&part.id, PART_BODY_PORT)),
    ];
    let reference = part.reference.clone();
    let id = s.add(&sch_symbol_type_for(sockets), inputs, Some(&format!("sym_{reference}_{unit}")), Some(&reference), meta(el.site))?;
    part.units.push((unit, id));
    Ok(())
}

/// A text attribute that must be there.
fn text_prop(el: &El, prop: &str, what: &str) -> Result<String, String> {
    match el.get(prop).and_then(Json::as_str) {
        Some(v) if !v.trim().is_empty() => Ok(v.to_string()),
        _ => Err(format!("{}: {prop} is {what}", el.at())),
    }
}

fn refuse_unknown(el: &El, allowed: &[&str]) -> Result<(), String> {
    match el.props.iter().find(|(k, _)| !allowed.contains(k)) {
        Some((k, _)) => Err(format!("{}: {k} is not read on a schematic", el.at())),
        None => Ok(()),
    }
}

fn num(el: &El, prop: &str) -> Option<f64> {
    el.get(prop).and_then(Json::as_f64)
}

fn declare_sheet(s: &mut Scope, children: &[El]) -> Result<(), String> {
    let mut parts: HashMap<String, Placed> = HashMap::new();
    let mut junctions: HashMap<String, String> = HashMap::new();
    let mut later: Vec<&El> = Vec::new();
    for el in children {
        if let Some(ideal) = ideal(el.tag) {
            let mut allowed = vec!["sch_x", "sch_y", "sch_rotation", "sch_mirror"];
            allowed.extend(ideal.value);
            if ideal.excitation {
                allowed.push("excitation");
            }
            refuse_unknown(el, &allowed)?;
            let reference = el.name.clone().filter(|n| !n.is_empty()).ok_or_else(|| format!("{}() needs a name", el.tag))?;
            // The netlist leaves power symbols out by their reference (KiCad's rule), so a ground's says it is one.
            if ideal.power.is_some() && !reference.starts_with('#') {
                return Err(format!("{}: a ground symbol's name starts with # (#PWR1), as KiCad names power symbols", el.at()));
            }
            if parts.contains_key(&reference) {
                return Err(format!("two parts are called {reference}"));
            }
            let pins: Vec<(String, String)> = (1..=ideal.pins).map(|i| (format!("p{i}"), i.to_string())).collect();
            let value = match ideal.power {
                Some(net) => Some(net.to_string()),
                None => ideal.value.and_then(|v| el.get(v)).map(value_text),
            };
            let mut inputs = vec![("ref".to_string(), Json::from(reference.as_str()))];
            if let Some(value) = value {
                inputs.push(("value".into(), value.into()));
            }
            inputs.push(("symbol".into(), ideal.symbol.into()));
            inputs.push(("pins".into(), Json::Arr(pins.iter().map(|(id, n)| Json::obj().with("id", id.as_str()).with("number", n.as_str())).collect())));
            inputs.push(("units".into(), Json::Num(1.0)));
            if ideal.power.is_some() {
                inputs.push(("powerSymbol".into(), true.into()));
            }
            if let Some(excitation) = el.get("excitation") {
                if let Json::Str(e) = excitation {
                    if let Some(why) = e.strip_prefix('\u{0}') {
                        return Err(format!("{}: excitation is JSON text ({why})", el.at()));
                    }
                }
                inputs.push(("excitation".into(), excitation.clone()));
            }
            let part = s.add(&part_type_for(&pins), inputs, Some(&reference), Some(&reference), meta(el.site))?;
            let mut placed = Placed { reference: reference.clone(), id: part, units: Vec::new(), ideal: Some((ideal.pins, ideal.aliases)) };
            let sockets: Vec<&str> = pins.iter().map(|(id, _)| id.as_str()).collect();
            place(s, &mut placed, 1, el, &sockets)?;
            parts.insert(reference, placed);
            continue;
        }
        match el.tag {
            "junction" => {
                refuse_unknown(el, &["sch_x", "sch_y"])?;
                let name = el.name.clone().filter(|n| !n.is_empty()).ok_or("junction() needs a name")?;
                if junctions.contains_key(&name) {
                    return Err(format!("two junctions are called {name}"));
                }
                let at = Json::obj().with("x", num(el, "sch_x").unwrap_or(0.0)).with("y", num(el, "sch_y").unwrap_or(0.0));
                let id = s.add(SCH_JUNCTION, vec![("at".into(), at)], Some(&name), Some("Junction"), meta(el.site))?;
                junctions.insert(name, id);
            }
            "part" => {
                refuse_unknown(el, &["symbol", "library", "value", "sch_x", "sch_y", "sch_rotation", "sch_mirror"])?;
                let reference = el.name.clone().filter(|n| !n.trim().is_empty()).ok_or_else(|| format!("{}: name is the part's reference (U1)", el.at()))?;
                if parts.contains_key(&reference) {
                    return Err(format!("two parts are called {reference}"));
                }
                let symbol = text_prop(el, "symbol", "its library ref (\"Device:R_Small\")")?;
                let mut halves = symbol.split(':');
                if !(halves.next().is_some_and(|l| !l.is_empty()) && halves.next().is_some_and(|n| !n.is_empty()) && halves.next().is_none()) {
                    return Err(format!("{}: symbol is a library ref, \"Library:Symbol\" (\"Device:R_Small\"), not {symbol:?}", el.at()));
                }
                let library = text_prop(el, "library", "the path of the .kicad_sym that holds the symbol")?;
                if !library.to_lowercase().ends_with(".kicad_sym") {
                    return Err(format!("{}: library names a .kicad_sym file, not {library:?}", el.at()));
                }
                // The pins are the library's: the editor reads them (and the part's type) from the library file.
                let mut inputs = vec![("ref".to_string(), Json::from(reference.as_str()))];
                if let Some(v) = el.get("value") {
                    inputs.push(("value".into(), value_text(v).into()));
                }
                inputs.push(("symbol".into(), symbol.as_str().into()));
                inputs.push(("library".into(), library.as_str().into()));
                inputs.push(("pins".into(), Json::Arr(Vec::new())));
                let part = s.add(&part_type_for(&[]), inputs, Some(&reference), Some(&reference), meta(el.site))?;
                let mut placed = Placed { reference: reference.clone(), id: part, units: Vec::new(), ideal: None };
                place(s, &mut placed, 1, el, &[])?;
                parts.insert(reference, placed);
            }
            "code" => {
                refuse_unknown(el, &["source", "inputs"])?;
                let name = el.name.clone().filter(|n| !n.trim().is_empty()).ok_or_else(|| format!("{}: name is the code part's id", el.at()))?;
                let source = text_prop(el, "source", "the path of the file it runs, relative to this one")?;
                let mut inputs = vec![("source".to_string(), Json::from(source.as_str()))];
                match el.get("inputs") {
                    None => {}
                    Some(Json::Obj(entries)) => {
                        if entries.iter().any(|(k, _)| k == "source") {
                            return Err(format!("{}: source is the file, not one of its inputs", el.at()));
                        }
                        inputs.extend(entries.iter().cloned());
                    }
                    Some(Json::Str(e)) if e.starts_with('\u{0}') => return Err(format!("{}: inputs is JSON text ({})", el.at(), &e[1..])),
                    Some(_) => return Err(format!("{}: inputs is an object of the file's parameters", el.at())),
                }
                if s.has(&slug(&name)) {
                    return Err(format!("two nodes are called {name}"));
                }
                let label = source.rsplit('/').next().unwrap_or(&source).to_string();
                s.add(CODE_OP, inputs, Some(&name), Some(&label), meta(el.site))?;
            }
            "unit" | "trace" | "netlabel" => later.push(el),
            other => return Err(format!("{other}() is not read on a schematic (see commandagi::design::sheet)")),
        }
    }

    // Units first: a wire may land on any unit's pin.
    for el in later.iter().filter(|el| el.tag == "unit") {
        refuse_unknown(el, &["unit", "sch_x", "sch_y", "sch_rotation", "sch_mirror"])?;
        let reference = el.name.clone().unwrap_or_default();
        let part = parts.get_mut(&reference).ok_or_else(|| format!("{}: there is no part {reference}", el.at()))?;
        let n = num(el, "unit").filter(|n| n.fract() == 0.0 && *n >= 2.0).ok_or_else(|| format!("{}: unit is 2 or more (the part's own sch_x and sch_y place unit 1)", el.at()))? as u32;
        if part.ideal.is_some() {
            return Err(format!("{}: {reference} is an ideal part, which has one unit", el.at()));
        }
        if part.units.iter().any(|(u, _)| *u == n) {
            return Err(format!("{}: unit {n} of {reference} is placed twice", el.at()));
        }
        place(s, part, n, el, &[])?;
    }
    let later: Vec<&El> = later.into_iter().filter(|el| el.tag != "unit").collect();

    let end = |sel: &str, el: &El| -> Result<End, String> {
        let t = sel.trim();
        if let Some(net) = t.strip_prefix("net.") {
            if !net.is_empty() && net.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '+' | '-')) {
                return Ok(End::Net(net.to_string()));
            }
        }
        let name_ok = |n: &str| !n.is_empty() && n.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '#' | '-'));
        let pin_ok = |n: &str| !n.is_empty() && n.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '+' | '-'));
        let refused = || format!("{}: {sel:?} is not \".REF > .pin\", \".JUNCTION\" or \"net.NAME\"", el.at());
        let body = t.strip_prefix('.').ok_or_else(refused)?;
        match body.split_once('>') {
            None if name_ok(body) => {
                let node = junctions.get(body).ok_or_else(|| format!("{}: there is no junction {body}", el.at()))?;
                Ok(End::Pin(node.clone(), VERTEX_PORT.to_string()))
            }
            Some((part, pin)) => {
                let (part, pin) = (part.trim_end(), pin.trim_start().strip_prefix('.').ok_or_else(refused)?);
                if !name_ok(part) || !pin_ok(pin) {
                    return Err(refused());
                }
                let p = parts.get(part).ok_or_else(|| format!("{}: there is no part {part}", el.at()))?;
                if p.units.is_empty() {
                    return Err(format!("{}: {} is not on the sheet (give it sch_x and sch_y)", el.at(), p.reference));
                }
                let Some((count, aliases)) = p.ideal else {
                    // A library part's pin by number (".pin5" or ".5"); the editor binds it to the unit that has it.
                    let number = match pin.get(..3) {
                        Some(head) if head.eq_ignore_ascii_case("pin") && pin.len() > 3 => &pin[3..],
                        _ => pin,
                    };
                    return Ok(End::Pin(p.id.clone(), format!("{LIBRARY_PIN_PORT}{number}")));
                };
                let placement = &p.units.iter().find(|(u, _)| *u == 1).ok_or_else(|| format!("{}: {} is not on the sheet (give it sch_x and sch_y)", el.at(), p.reference))?.1;
                let key = pin.to_lowercase();
                let number = aliases.iter().find(|(a, _)| *a == key).map(|(_, n)| n.to_string()).or_else(|| key.chars().all(|c| c.is_ascii_digit()).then(|| key.clone()));
                match number.and_then(|n| n.parse::<usize>().ok()) {
                    Some(n) if n >= 1 && n <= count => Ok(End::Pin(placement.clone(), format!("p{n}"))),
                    _ => Err(format!("{}: {} has no pin {pin}", el.at(), p.reference)),
                }
            }
            None => Err(refused()),
        }
    };

    let mut wires = 0;
    for el in later {
        if el.tag == "netlabel" {
            refuse_unknown(el, &["net", "connection"])?;
            let text = el.get("net").and_then(Json::as_str).map(str::trim).filter(|t| !t.is_empty()).ok_or("netlabel() needs a net name")?.to_string();
            match end(el.get("connection").and_then(Json::as_str).unwrap_or(""), el)? {
                End::Net(_) => return Err(format!("{}: a label's connection is a pin or a junction", el.at())),
                End::Pin(node, port) => label(s, &text, &node, &port, el)?,
            }
            continue;
        }
        refuse_unknown(el, &[])?;
        let ends = el.path.iter().map(|p| end(p, el)).collect::<Result<Vec<_>, _>>()?;
        for pair in ends.windows(2) {
            match (&pair[0], &pair[1]) {
                (End::Net(a), End::Net(b)) => return Err(format!("trace() joins two nets ({a}, {b})")),
                (End::Net(net), End::Pin(node, port)) | (End::Pin(node, port), End::Net(net)) => label(s, net, node, port, el)?,
                (End::Pin(an, ap), End::Pin(bn, bp)) => {
                    wires += 1;
                    let id = free(s, &format!("w_{wires}"));
                    s.add(SCH_WIRE, channels("ends", vec![wire(an, ap), wire(bn, bp)]), Some(&id), Some("Wire"), meta(el.site))?;
                }
            }
        }
    }
    Ok(())
}

enum End {
    Net(String),
    Pin(String, String),
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
    s.add(SCH_LABEL, inputs, Some(&id), Some(text), meta(el.site)).map(|_| ())
}

/// A value as the sheet holds it: text as written, a number as JavaScript's `String(n)` writes it (1000, 1.5).
fn value_text(raw: &Json) -> String {
    match raw {
        Json::Str(s) => s.clone(),
        other => other.text(),
    }
}
