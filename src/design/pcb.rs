//! A BOARD IN RUST — the board half of a circuit, declared as the nodes the CommandAGI circuit editor's board holds
//! (`eda.board`, `eda.stack`, `eda.conductor`) plus, for each part, the board facts that sit on the schematic's part
//! (`eda.footprint`: which footprint, where it sits). The same nodes as the TypeScript SDK's JSX board (`pcb.ts`) and
//! the Python SDK's (`pcb.py`). A board names its schematic; the schematic says what the parts are and what is
//! connected, the board says where they sit and where the copper runs:
//!
//! ```
//! use commandagi::design::pcb::*;
//!
//! fn document() -> El {
//!     board([
//!         component().name("R1").footprint("smd-0805").pcb_x(10).pcb_y(10),
//!         component().name("R2").footprint("smd-0805").pcb_x(25).pcb_y(10).pcb_rotation(90).layer("bottom"),
//!         trace().layer("F.Cu").width(0.2).points([[11, 10], [24, 10]]).from(".R1 > .pin2").to(".R2 > .pin1"),
//!         via().name("V1").pcb_x(18).pcb_y(14).drill(0.4).diameter(0.8),
//!     ])
//!     .schematic("Divider.sch.rs")
//!     .width(40)
//!     .height(30)
//!     .core(1.5)
//!     .copper(0.035)
//! }
//! # commandagi::design::declare(document()).unwrap();
//! ```
//!
//! The elements (every attribute a chained method):
//!
//! ```text
//! board([…]).schematic().width().height().core().copper().thickness().layers().surface()
//!                                   the root. `schematic` is the schematic file, relative to this one. A width x
//!                                   height rectangle outline from (0, 0); core and copper make a two-layer
//!                                   cross-section (mm); thickness is the finished thickness when no cross-section
//!                                   says it; layers is the layer table (json: [{ordinal, name, type, userName}], the
//!                                   editor's blank board's when absent). `surface` puts the board on the faces of a
//!                                   CAD part: json({cadRef, domain, trims, curvedTrims}).
//! stack().name().label().domain().units().process().layers().overall_thickness()
//!                                   the cross-section, layer by layer (top first), when it is more than core and
//!                                   copper.
//! masks().name().label().lib_name().unit_um_per_db().polygons()
//!                                   an IC mask layout, flattened: each polygon {layer, datatype, points}, its
//!                                   points in micrometres, the stream's units and the library it names.
//! graphic().kind().layer().points().width().filled().arcs().id().kicad()
//!                                   a board drawing (line, arc, circle, rect, poly) on a layer.
//! text().text()|.field().at().rotation().layer().size().size_x().thickness().kind().id().kicad()
//! dimension().points().height().layer().width().text_size().id().kicad()
//!                                   board text, and an aligned dimension (its text is derived, never stored).
//! kicad().version().generator().forms()
//!                                   the KiCad forms the board carries for exchange and nothing else reads.
//! net().name().code()               the KiCad net code of the schematic's net `name`.
//! component().name().footprint().library().pcb_x().pcb_y().pcb_rotation().layer().locked().ref_at().ref_rot().value_at().chart()
//!                                   .uuid().kicad()
//!                                   the schematic's part `name` on this board: one of the editor's own land patterns
//!                                   (BOARD_FOOTPRINTS), or a library footprint by its ref ("Package_SO:SOIC-8") in
//!                                   the `.pretty` folder `library` names. `ref_at` and `value_at` [x, y] are where
//!                                   its reference and value texts sit (mm, in the footprint's own frame), and
//!                                   `ref_rot` how far its reference text is turned there (degrees).
//! trace().name().layer().width().points().rule().from().to().net().surface().uuid().segment_uuids().kicad()
//!                                   a copper run: its points ([[x, y], …]) on one copper layer, a finished width.
//!                                   `rule` is its routing rule, "45", "90" or "any" (the default): the angles its
//!                                   legs take between its points, so a moved point re-routes the legs.
//! arc().name().layer().width().points().from().to().net().uuid().kicad()
//!                                   a circular copper arc: start, a point on it, end.
//! via().name().pcb_x().pcb_y().drill().diameter().layers().pads().net().uuid().kicad()
//!                                   a plated barrel; `layers` defaults to ["F.Cu", "B.Cu"].
//! pour().name().layers().points().terminals(json(r#"[[0, ".J1 > .pin2"]]"#)).net().uuid().kicad()
//!                                   a copper pour: its boundary on its layers.
//! ```
//!
//! The layout's constraints (the board's design intent: the placer, the router, the layout score and the checks read
//! them; the editor and the layout operations write each one back):
//!
//! ```text
//! netclass().name().nets().kind().width().min_width().clearance().edge_clearance().hole_clearance().inner()
//!          .min_spokes().via_diameter().via_drill().current().criticality().max_length().max_vias().impedance()
//!                                   the rules of the nets named ("FIN*" matches by prefix): the width the router
//!                                   lays and the narrowest allowed, the gaps to copper, to the board edge and to a
//!                                   hole, the rules on inner layers, the fewest thermal spokes a pad needs.
//! diffpair().name().positive().negative().gap().width().impedance().max_skew()
//! group().name().parts().region().near().max_distance().conviction().side().edge().mating()
//!                                   parts placed together: in the box `region` [x0, y0, x1, y1], within
//!                                   `max_distance` of `near` (".J1 > .pin5"); conviction 1 holds them fixed;
//!                                   `side` "top", "bottom" or "any" (the placer turns a part over where that costs
//!                                   less); `edge` the board edge they sit against ("left", "right", "top",
//!                                   "bottom"), their `mating` side (in the footprint's own frame) facing out.
//! keepout().name().points().layers().no()
//!                                   a region where `no` ("track", "via", "component", "pour") may be.
//! thermal().name().parts().watts().spacing().away_from().distance()
//! bus().name().nets().pitch()       nets that run together ("D*" matches by prefix): where their tracks run side by
//!                                   side they are parallel, `pitch` (mm) apart centre to centre.
//! objective().weights()             the weight of each term of the layout score.
//! ```
//!
//! How the board is made (docs/formats.md § logistics):
//!
//! ```text
//! fab().suppliers(json(…)).finish().mask().silk().min_track().min_space().min_drill().min_annular_ring()
//!      .max_layers().thicknesses([…]).assembly(json(…)).inspection(json(…)).compliance(json(…)).shipping(json(…))
//!                                   the fab house (the first of `suppliers`, with its terms), its capabilities for
//!                                   the service ordered (mm), the finish, the solder mask and silk colours, the
//!                                   assembly ({"house", "sides"}), the board's test ({"electrical", "procedure",
//!                                   "acceptance", …}), its compliance and shipping. One per board. The board's
//!                                   layers, thickness and copper stay its own.
//! ```
//!
//! The checks the board must pass and the findings it accepts on purpose (kept in the graph's meta, `checks` and
//! `accepted`, not as nodes: `super::circuit_checks`):
//!
//! ```text
//! check().id("drc").kind("drc").severity("warning")
//! accept().id().check().rule().refs().net().region(json(r#"{"min": [0, 0], "max": [3, 3]}"#)).reason()
//! ```
//!
//! Coordinates are the board's own: millimetres, Y DOWN, from the outline's corner. An end is bound by what it says,
//! never by where it is drawn: ".R1 > .pin2" (a pin of a component, by its number), ".V1" (a via, by name),
//! ".T1 > .end" (a point of another trace or arc: .start, .end or its index). Anything else is refused by name, never
//! guessed. Each node an element declares carries the element's call in `meta.source`.

use super::circuit_checks::declare_circuit_checks;
use super::element::{attributes, elements, fn_name, rust_name};
use super::ir::{slug, Scope};
use super::part_fields::{check_facet, check_offers};
use super::source::meta;
use super::{Declared, Family};
pub use super::{fragment, json, El, Json, Value};
use std::collections::HashMap;

elements! {
    /// The board: its elements.
    board: children;
    /// The cross-section, layer by layer (top first).
    stack: leaf;
    /// An IC mask layout, flattened: its polygons, the stream's units and its library.
    masks: leaf;
    /// A board drawing on a layer.
    graphic: leaf;
    /// Board text: `text().text("REV A").at([2, 3])…`.
    text: leaf;
    /// An aligned dimension.
    dimension: leaf;
    /// The KiCad forms the board carries for exchange.
    kicad: leaf;
    /// The KiCad net code of the schematic's net: `net().name("GND").code(1)`.
    net: leaf;
    /// The schematic's part on this board: `component().name("R1").footprint("smd-0805").pcb_x(10).pcb_y(10)`.
    component: leaf;
    /// A copper run.
    trace: leaf;
    /// A circular copper arc: start, a point on it, end.
    arc: leaf;
    /// A plated barrel.
    via: leaf;
    /// A copper pour.
    pour: leaf;
    /// The rules of some nets: `netclass().name("supply").nets(["P24", "FIN*"]).width(0.5)`.
    netclass: leaf;
    /// Two nets routed as a pair.
    diffpair: leaf;
    /// Parts placed together: `group().name("in1").parts(["R1", "U1"]).region([10, 30, 30, 50])`.
    group: leaf;
    /// A region where tracks, vias, components or pours may not be.
    keepout: leaf;
    /// The heat parts dissipate, and the spacing it asks for.
    thermal: leaf;
    /// Nets that run together, parallel where side by side: `bus().name("data").nets(["D*"]).pitch(0.6)`.
    bus: leaf;
    /// The weight of each term of the layout score.
    objective: leaf;
    /// How the board is made: `fab().suppliers(json(…)).finish("ENIG").min_track(0.09).assembly(json(…))`.
    fab: leaf;
    /// A check the board must pass: `check().id("drc").kind("drc")` (`super::circuit_checks`).
    check: leaf;
    /// A finding the board accepts on purpose: `accept().id("edge").check("drc").rule("clearance").region(…).reason(…)`.
    accept: leaf;
}

attributes! {
    /// The board's attributes, chained on its elements.
    pub trait PcbAttrs {
        /// The schematic file, relative to this one.
        schematic;
        /// A component's part, a conductor's name, a stack's id, a net's name.
        name;
        /// The outline's width (mm); a run's, a drawing's or a dimension's line width (mm).
        width;
        /// The outline's height (mm); a dimension's offset (mm).
        height;
        /// A two-layer cross-section's core (mm).
        core;
        /// A two-layer cross-section's copper (mm).
        copper;
        /// The finished thickness (mm), or a text's stroke thickness (mm).
        thickness;
        /// The layer table, a stack's layers, or a via's or pour's copper layers.
        layers;
        /// The CAD faces the board is on, or `true` on a trace that runs on them.
        surface;
        /// A stack's label.
        label;
        /// A stack's domain: "pcb" or "ic".
        domain;
        /// A stack's units: "mm", "um" or "nm".
        units;
        /// A stack's process: `json(r#"{"name": "JLC"}"#)`.
        process;
        /// A stack's overall thickness.
        overall_thickness;
        /// A mask layout's polygons: `json(r#"[{"layer": 1, "datatype": 0, "points": [[x, y], …]}]"#)`.
        polygons;
        /// A mask layout's library name (`libName`).
        lib_name;
        /// The stream's units: micrometres per database unit (`unitUmPerDb`).
        unit_um_per_db;
        /// A drawing's kind (line, arc, circle, rect, poly), or a text's kind.
        kind;
        /// A layer by name ("F.Cu", "Edge.Cuts"), or a component's side ("top" or "bottom").
        layer;
        /// Points: `[[x, y], …]` in mm.
        points;
        /// A drawing is filled.
        filled;
        /// A polygon's arcs: the indices in `points` where an arc's start, mid and end begin.
        arcs;
        /// A drawing's, a text's or a dimension's KiCad id.
        id;
        /// The KiCad forms an element carries, as s-expression text.
        kicad;
        /// A text's text.
        text;
        /// A text's field: "reference" or "value".
        field;
        /// Where a text sits: `[x, y]` in mm.
        at;
        /// A text's rotation (degrees).
        rotation;
        /// A text's size (mm).
        size;
        /// A text's width, when not its size (mm).
        size_x;
        /// A dimension's text size (mm).
        text_size;
        /// The KiCad file version.
        version;
        /// The KiCad generator.
        generator;
        /// The KiCad forms a board carries, as s-expression text.
        forms;
        /// A net's KiCad code.
        code;
        /// A component's footprint.
        footprint;
        /// The `.pretty` folder of a library footprint.
        library;
        /// Where a component or a via sits, across (mm).
        pcb_x;
        /// Where a component or a via sits, down (mm).
        pcb_y;
        /// A component's rotation (degrees).
        pcb_rotation;
        /// KiCad's lock on a footprint: a mouse does not move it (`true`, or left out).
        locked;
        /// Where a component's reference text sits: `[x, y]` in mm, in the footprint's own frame.
        ref_at;
        /// How far a component's reference text is turned where it sits (degrees).
        ref_rot;
        /// Where a component's value text sits: `[x, y]` in mm, in the footprint's own frame.
        value_at;
        /// The face of the board's surface a component sits on.
        chart;
        /// A KiCad uuid.
        uuid;
        /// What a run's first point lands on: `".R1 > .pin2"`, `".V1"`, `".T1 > .end"`.
        from;
        /// What a run's last point lands on.
        to;
        /// The schematic net the copper realises.
        net;
        /// A trace's KiCad segment uuids, one per segment.
        segment_uuids;
        /// A via's drill (mm).
        drill;
        /// A via's pad diameter (mm).
        diameter;
        /// The pads a via is plated into: `[".R1 > .pin2"]`.
        pads;
        /// What a pour's boundary points land on: `json(r#"[[0, ".J1 > .pin2"]]"#)`.
        terminals;
        /// The nets a net class names: `["P24", "FIN*"]` ("FIN*" matches by prefix).
        nets;
        /// A net class's clearance (mm).
        clearance;
        /// A net class's via pad diameter (mm).
        via_diameter;
        /// A net class's via drill (mm).
        via_drill;
        /// The current a net class's copper carries (A).
        current;
        /// How critical a net class is, 0 to 1: routed first, kept short.
        criticality;
        /// A net class's longest route (mm).
        max_length;
        /// A net class's most vias per net.
        max_vias;
        /// The narrowest track a net class allows (a neck-down, mm); its `width` is what the router lays.
        min_width;
        /// The least gap from a net class's copper to the board edge (mm).
        edge_clearance;
        /// The least gap from a net class's copper to a hole (mm).
        hole_clearance;
        /// A net class's rules on inner copper layers: `json(r#"{"clearance": 0.15, "width": 0.15}"#)`.
        inner;
        /// The fewest thermal spokes a pad of a net class needs to a pour.
        min_spokes;
        /// The impedance of a trace over its reference plane (ohm).
        impedance;
        /// A differential pair's positive net.
        positive;
        /// A differential pair's negative net.
        negative;
        /// The gap between a differential pair's tracks (mm).
        gap;
        /// The most length difference of a differential pair (mm).
        max_skew;
        /// The parts a group or a thermal fact names.
        parts;
        /// The box a group stays in: `[x0, y0, x1, y1]` (mm).
        region;
        /// The part or pin a group stays near: `".J1 > .pin5"`.
        near;
        /// How far a group may be from `near` (mm).
        max_distance;
        /// How fixed a group is: 0 (free) to 1 (fixed where it is).
        conviction;
        /// The side a group is on: "top", "bottom" or "any" (the placer turns each part over where that costs less).
        side;
        /// The board edge a group sits against: "left", "right", "top" or "bottom".
        edge;
        /// The side of a group's parts (in the footprint's own frame) that faces out of `edge`.
        mating;
        /// How far apart a bus's parallel tracks are, centre to centre (mm).
        pitch;
        /// What a keep-out keeps out: `["track", "via", "component", "pour"]`.
        no;
        /// The heat each part dissipates (W).
        watts;
        /// The least spacing between the parts of a thermal fact (mm).
        spacing;
        /// The parts a thermal fact keeps `distance` from.
        away_from;
        /// How far a thermal fact keeps from `away_from` (mm).
        distance;
        /// The weight of each term of the layout score: `json(r#"{"wirelength": 1, "vias": 2}"#)`.
        weights;
        /// A check's severity: "error", or "warning" (its errors are warnings).
        severity;
        /// A logistics check's options: `json(r#"{"quantities": [10, 100]}"#)`.
        options;
        /// The fab house and its terms (the first makes the board):
        /// `json(r#"[{"name": "JLCPCB", "currency": "USD", "priceBreaks": [{"quantity": 5, "unitPrice": 2}]}]"#)`.
        suppliers;
        /// The surface finish: "ENIG", "HASL".
        finish;
        /// The solder mask colour.
        mask;
        /// The silkscreen colour.
        silk;
        /// The house's narrowest track (mm).
        min_track;
        /// The house's narrowest space (mm).
        min_space;
        /// The house's smallest drill (mm).
        min_drill;
        /// The house's smallest annular ring (mm).
        min_annular_ring;
        /// The most copper layers the house makes.
        max_layers;
        /// The finished thicknesses the house offers (mm): `[0.8, 1.6]`.
        thicknesses;
        /// Who places the parts, on which sides: `json(r#"{"house": "JLCPCB", "sides": ["top"]}"#)`.
        assembly;
        /// The board's test: `json(r#"{"electrical": "flying-probe", "acceptance": "boots"}"#)`.
        inspection;
        /// The board's compliance: `json(r#"{"rohs": "compliant", "reach": "compliant", "eccn": "EAR99"}"#)`.
        compliance;
        /// The board's shipping: `json(r#"{"hsCode": "8534.00", "origin": "CN", "hazmat": "none"}"#)`.
        shipping;
        /// The id of the check whose findings an accept accepts.
        check;
        /// The rule of the findings an accept accepts: "clearance", "pin_conflict"; a trace's routing rule: "45", "90"
        /// or "any".
        rule;
        /// The parts an accepted finding is about, each of them: `["U1", "U3"]`.
        refs;
        /// Why an accepted finding is right on purpose.
        reason;
    }
}

pub(crate) const FAMILY: Family = Family { module: "pcb", roots: &["board"], declare };

pub const BOARD_PART: &str = "eda.footprint";
pub const CONDUCTOR: &str = "eda.conductor";
/// A board fact a `graphic`, `text`, `dimension`, `kicad` or `net` declares; the editor folds it into the circuit.
pub const BOARD_FACT: &str = "eda.boardfact";
/// The editor's own land patterns, by the name a `component().footprint(…)` gives (the circuit's id is `Authored:<name>`).
pub const BOARD_FOOTPRINTS: &[&str] = &["smd-0805", "axial-7.62", "header-2.54"];
/// A new board's layer table (the circuit editor's blank board): ordinal, name, type.
pub const BOARD_LAYERS: &[(u32, &str, &str)] = &[(0, "F.Cu", "signal"), (31, "B.Cu", "signal"), (36, "B.SilkS", "user"), (37, "F.SilkS", "user"), (44, "Edge.Cuts", "user")];
/// The drawings, by kind: how many points each has (none: a polygon, two or more).
const GRAPHICS: &[(&str, Option<usize>)] = &[("line", Some(2)), ("arc", Some(3)), ("circle", Some(2)), ("rect", Some(2)), ("poly", None)];
const CONDUCTORS: &[&str] = &["trace", "arc", "via", "pour"];
/// A run's routing rules: the angles its legs take between its points (45° steps, 90° steps, or any angle).
const TRACE_RULES: &[&str] = &["45", "90", "any"];
const FACTS: &[&str] = &["graphic", "text", "dimension", "kicad", "net", "netclass", "diffpair", "group", "keepout", "thermal", "bus", "objective", "fab"];
/// The layout's constraints, each a board fact (`fact` is its tag), as the TypeScript SDK's.
const CONSTRAINTS: &[&str] = &["netclass", "diffpair", "group", "keepout", "thermal", "bus", "objective"];
const NET_KINDS: &[&str] = &["signal", "power", "ground", "noisy", "sensitive", "highspeed"];
const KEEPOUT_KINDS: &[&str] = &["track", "via", "component", "pour"];
const EDGES: &[&str] = &["left", "right", "top", "bottom"];

fn is_names(v: &Json) -> bool {
    matches!(v, Json::Arr(l) if !l.is_empty() && l.iter().all(|x| x.as_str().is_some_and(|s| !s.is_empty())))
}

fn is_box(v: &Json) -> bool {
    match v {
        Json::Arr(l) if l.len() == 4 => {
            let n: Vec<f64> = l.iter().filter_map(finite).collect();
            n.len() == 4 && n[0] < n[2] && n[1] < n[3]
        }
        _ => false,
    }
}

/// A pin or a part a group stays near: `.J1` or `.J1 > .pin5`.
fn is_near(s: &str) -> bool {
    let name = |t: &str, extra: &str| !t.is_empty() && t.chars().all(|c| c.is_ascii_alphanumeric() || extra.contains(c));
    let mut parts = s.split('>').map(str::trim);
    let first = parts.next().and_then(|p| p.strip_prefix('.')).is_some_and(|p| name(p, "_#-"));
    let rest: Vec<&str> = parts.collect();
    first && rest.len() <= 1 && rest.iter().all(|p| p.strip_prefix('.').is_some_and(|p| name(p, "_+-")))
}

/// A constraint element's fact: what it takes, checked.
fn constraint_fact(el: &El) -> Result<Vec<(String, Json)>, String> {
    let mut out: Vec<(String, Json)> = vec![("fact".into(), Json::from(el.tag))];
    let unit = |prop: &str| -> Result<Option<f64>, String> {
        let v = num(el, prop, false)?;
        if v.is_some_and(|v| !(0.0..=1.0).contains(&v)) {
            return Err(format!("{}: {} is from 0 to 1", at(el), rust_name(prop)));
        }
        Ok(v)
    };
    let names = |prop: &str, required: bool| -> Result<Option<Json>, String> {
        match plain(el, prop, is_names, "a list of names")? {
            None if required => Err(format!("{} needs {}", at(el), rust_name(prop))),
            v => Ok(v),
        }
    };
    fn put(out: &mut Vec<(String, Json)>, k: &str, v: Option<impl Into<Json>>) {
        if let Some(v) = v {
            out.push((k.into(), v.into()));
        }
    }
    let has = |out: &Vec<(String, Json)>, k: &str| out.iter().any(|(key, _)| key == k);
    match el.tag {
        "netclass" => {
            refuse_unknown(el, &["name", "nets", "kind", "width", "minWidth", "clearance", "edgeClearance", "holeClearance", "inner", "minSpokes", "viaDiameter", "viaDrill", "current", "criticality", "maxLength", "maxVias", "impedance"])?;
            let kind = text_of(el, "kind", false)?;
            if kind.as_deref().is_some_and(|k| !NET_KINDS.contains(&k)) {
                return Err(format!("{}: kind is {}", at(el), NET_KINDS.join(", ")));
            }
            let max_vias = num(el, "maxVias", false)?;
            if max_vias.is_some_and(|v| v.fract() != 0.0 || v < 0.0) {
                return Err(format!("{}: {} is a whole number, 0 or more", at(el), rust_name("maxVias")));
            }
            put(&mut out, "name", text_of(el, "name", true)?);
            put(&mut out, "nets", names("nets", true)?);
            put(&mut out, "kind", kind);
            for k in ["width", "minWidth", "clearance", "edgeClearance", "holeClearance", "viaDiameter", "viaDrill", "current", "maxLength", "impedance"] {
                put(&mut out, k, positive(el, k, false)?);
            }
            put(&mut out, "criticality", unit("criticality")?);
            put(&mut out, "maxVias", max_vias);
            if let (Some(min), Some(w)) = (num(el, "minWidth", false)?, num(el, "width", false)?) {
                if min > w {
                    return Err(format!("{}: {} is at most width", at(el), rust_name("minWidth")));
                }
            }
            let rules = |v: &Json| matches!(v, Json::Obj(e) if !e.is_empty() && e.iter().all(|(k, x)| ["clearance", "width", "minWidth"].contains(&k.as_str()) && finite(x).is_some_and(|x| x > 0.0)));
            if let Some(inner) = plain(el, "inner", rules, "the rules on inner copper layers: { clearance, width, minWidth } (mm)")? {
                out.push(("inner".into(), inner));
            }
            let spokes = num(el, "minSpokes", false)?;
            if spokes.is_some_and(|v| v.fract() != 0.0 || !(0.0..=4.0).contains(&v)) {
                return Err(format!("{}: {} is a whole number from 0 to 4", at(el), rust_name("minSpokes")));
            }
            put(&mut out, "minSpokes", spokes);
            if let (Some(d), Some(h)) = (num(el, "viaDiameter", false)?, num(el, "viaDrill", false)?) {
                if d <= h {
                    return Err(format!("{}: {} is more than {}", at(el), rust_name("viaDiameter"), rust_name("viaDrill")));
                }
            }
        }
        "diffpair" => {
            refuse_unknown(el, &["name", "positive", "negative", "gap", "width", "impedance", "maxSkew"])?;
            let (p, n) = (text_of(el, "positive", true)?, text_of(el, "negative", true)?);
            if p == n {
                return Err(format!("{}: positive and negative are two nets", at(el)));
            }
            put(&mut out, "name", text_of(el, "name", true)?);
            put(&mut out, "positive", p);
            put(&mut out, "negative", n);
            for k in ["gap", "width", "impedance", "maxSkew"] {
                put(&mut out, k, positive(el, k, false)?);
            }
        }
        "group" => {
            refuse_unknown(el, &["name", "parts", "region", "near", "maxDistance", "conviction", "side", "edge", "mating"])?;
            put(&mut out, "name", text_of(el, "name", true)?);
            put(&mut out, "parts", names("parts", true)?);
            put(&mut out, "region", plain(el, "region", is_box, "[x0, y0, x1, y1] in mm, x0 < x1 and y0 < y1")?);
            let near = text_of(el, "near", false)?;
            if near.as_deref().is_some_and(|s| !is_near(s)) {
                return Err(format!("{}: near is a part (\".J1\") or a pin (\".J1 > .pin5\")", at(el)));
            }
            let max_distance = positive(el, "maxDistance", false)?;
            if max_distance.is_some() && near.is_none() {
                return Err(format!("{}: {} needs near", at(el), rust_name("maxDistance")));
            }
            put(&mut out, "near", near);
            put(&mut out, "maxDistance", max_distance);
            put(&mut out, "conviction", unit("conviction")?);
            let side = text_of(el, "side", false)?;
            if side.as_deref().is_some_and(|s| s != "top" && s != "bottom" && s != "any") {
                return Err(format!("{}: side is \"top\", \"bottom\" or \"any\"", at(el)));
            }
            put(&mut out, "side", side);
            for k in ["edge", "mating"] {
                let v = text_of(el, k, false)?;
                if v.as_deref().is_some_and(|e| !EDGES.contains(&e)) {
                    let edges: Vec<String> = EDGES.iter().map(|e| format!("\"{e}\"")).collect();
                    return Err(format!("{}: {k} is {}", at(el), edges.join(", ")));
                }
                put(&mut out, k, v);
            }
            if has(&out, "mating") && !has(&out, "edge") {
                return Err(format!("{}: mating needs edge", at(el)));
            }
        }
        "keepout" => {
            refuse_unknown(el, &["name", "points", "layers", "no"])?;
            put(&mut out, "name", text_of(el, "name", false)?);
            let points: Vec<Json> = point_list(el, "points", 3, None)?.iter().map(|(x, y)| Json::Arr(vec![(*x).into(), (*y).into()])).collect();
            out.push(("points".into(), Json::Arr(points)));
            if el.attr_of("layers").is_some() {
                let layers: Vec<Json> = copper_layers(el, None)?.into_iter().map(Json::from).collect();
                out.push(("layers".into(), Json::Arr(layers)));
            }
            let what = format!("a list of {}", KEEPOUT_KINDS.join(", "));
            put(&mut out, "no", plain(el, "no", |v| is_names(v) && matches!(v, Json::Arr(l) if l.iter().all(|k| k.as_str().is_some_and(|k| KEEPOUT_KINDS.contains(&k)))), &what)?);
        }
        "thermal" => {
            refuse_unknown(el, &["name", "parts", "watts", "spacing", "awayFrom", "distance"])?;
            put(&mut out, "name", text_of(el, "name", true)?);
            put(&mut out, "parts", names("parts", true)?);
            put(&mut out, "watts", positive(el, "watts", true)?);
            put(&mut out, "spacing", positive(el, "spacing", false)?);
            put(&mut out, "awayFrom", names("awayFrom", false)?);
            put(&mut out, "distance", positive(el, "distance", false)?);
            if has(&out, "awayFrom") != has(&out, "distance") {
                return Err(format!("{}: give away_from and distance together", at(el)));
            }
        }
        "bus" => {
            refuse_unknown(el, &["name", "nets", "pitch"])?;
            put(&mut out, "name", text_of(el, "name", true)?);
            put(&mut out, "nets", names("nets", true)?);
            put(&mut out, "pitch", positive(el, "pitch", false)?);
        }
        _ => {
            refuse_unknown(el, &["weights"])?;
            let ok = |v: &Json| matches!(v, Json::Obj(e) if e.iter().all(|(_, x)| finite(x).is_some_and(|x| x >= 0.0)));
            match plain(el, "weights", ok, "an object of term → weight (0 or more)")? {
                Some(w) => out.push(("weights".into(), w)),
                None => return Err(format!("{} needs weights", at(el))),
            }
        }
    }
    Ok(out)
}
const SCHEMATIC_ONLY: &[&str] = &["schX", "schY", "schRotation", "resistance", "capacitance", "inductance", "voltage", "current"];

/// A trace point's id: the editor's own (`start`, `end`, and `p<i>` between them).
pub fn trace_point_id(index: usize, count: usize) -> String {
    if index == 0 {
        "start".into()
    } else if index + 1 == count {
        "end".into()
    } else {
        format!("p{index}")
    }
}

const FAB_CAPABILITIES: &[&str] = &["minTrack", "minSpace", "minDrill", "minAnnularRing"];
const SIDES: &[&str] = &["top", "bottom"];
const ELECTRICAL: &[&str] = &["flying-probe", "fixture", "none"];
const INSPECTION_KEYS: &[&str] = &["dimensions", "procedure", "acceptance", "sampling", "electrical", "minutes", "ratePerHour", "currency"];

/// The board's `fab()`: the house that makes it (`suppliers`, the first is the one that makes it), its capabilities
/// for the service ordered, the finish, mask and silk, the assembly, the board's test and its compliance and shipping
/// (docs/formats.md § logistics). The board's layers, thickness and copper are its own and are not restated here.
fn fab_fact(el: &El) -> Result<Vec<(String, Json)>, String> {
    let fields = [&["suppliers", "finish", "mask", "silk"][..], FAB_CAPABILITIES, &["maxLayers", "thicknesses", "assembly", "inspection", "compliance", "shipping"]].concat();
    refuse_unknown(el, &fields)?;
    let w = at(el);
    let mut out = vec![("fact".to_string(), Json::from("fab"))];
    if let Some(v) = el.attr_of("suppliers") {
        out.push(("suppliers".into(), check_offers(&w, v)?));
    }
    for k in ["finish", "mask", "silk"] {
        if let Some(v) = text_of(el, k, false)? {
            if v.trim().is_empty() {
                return Err(format!("{w}: {k} is text"));
            }
            out.push((k.into(), v.into()));
        }
    }
    for k in FAB_CAPABILITIES {
        if let Some(v) = positive(el, k, false)? {
            out.push((k.to_string(), v.into()));
        }
    }
    if let Some(n) = num(el, "maxLayers", false)? {
        if !(n.fract() == 0.0 && n >= 1.0) {
            return Err(format!("{w}: max_layers is a whole number of copper layers"));
        }
        out.push(("maxLayers".into(), n.into()));
    }
    let thick = |v: &Json| matches!(v, Json::Arr(l) if !l.is_empty() && l.iter().all(|x| finite(x).is_some_and(|t| t > 0.0)));
    if let Some(t) = plain(el, "thicknesses", thick, "a list of the finished thicknesses the house offers (mm)")? {
        out.push(("thicknesses".into(), t));
    }
    if let Some(a) = el.attr_of("assembly") {
        if !is_object(a) || a_keys(a).any(|k| k != "house" && k != "sides") {
            return Err(format!("{w}: assembly is {{ house, sides }}"));
        }
        if a.get("house").is_some() && !matches!(a.get("house"), Some(Json::Str(h)) if !h.trim().is_empty()) {
            return Err(format!("{w}: assembly.house is the house that places the parts"));
        }
        if let Some(sides) = a.get("sides") {
            let ok = match sides {
                Json::Arr(l) => {
                    let names: Vec<&str> = l.iter().filter_map(Json::as_str).filter(|s| SIDES.contains(s)).collect();
                    !l.is_empty() && names.len() == l.len() && names.iter().enumerate().all(|(i, s)| !names[..i].contains(s))
                }
                _ => false,
            };
            if !ok {
                return Err(format!(r#"{w}: assembly.sides is a list of "top", "bottom""#));
            }
        }
        out.push(("assembly".into(), a.clone()));
    }
    if let Some(i) = el.attr_of("inspection") {
        if !is_object(i) {
            return Err(format!("{w}: inspection is an object ({{ {} }})", INSPECTION_KEYS.join(", ")));
        }
        if let Some(k) = a_keys(i).find(|k| !INSPECTION_KEYS.contains(k)) {
            return Err(format!("{w}: inspection.{k} is not read (it takes {})", INSPECTION_KEYS.join(", ")));
        }
        if i.get("electrical").is_some_and(|e| !e.as_str().is_some_and(|e| ELECTRICAL.contains(&e))) {
            return Err(format!("{w}: inspection.electrical is one of {}", ELECTRICAL.join(", ")));
        }
        for k in ["procedure", "acceptance", "sampling"] {
            if i.get(k).is_some_and(|v| !v.as_str().is_some_and(|s| !s.trim().is_empty())) {
                return Err(format!("{w}: inspection.{k} is text"));
            }
        }
        for k in ["minutes", "ratePerHour"] {
            if i.get(k).is_some_and(|v| !v.as_f64().is_some_and(|n| n >= 0.0)) {
                return Err(format!("{w}: inspection.{k} is a number, 0 or more"));
            }
        }
        if i.get("currency").is_some_and(|c| !c.as_str().is_some_and(|c| c.len() == 3 && c.bytes().all(|b| b.is_ascii_uppercase()))) {
            return Err(format!(r#"{w}: inspection.currency is an ISO 4217 code ("USD")"#));
        }
        out.push(("inspection".into(), i.clone()));
    }
    for k in ["compliance", "shipping"] {
        if let Some(v) = el.attr_of(k) {
            out.push((k.into(), check_facet(&w, k, v)?));
        }
    }
    Ok(out)
}

/// The keys of a JSON object, in order.
fn a_keys(v: &Json) -> impl Iterator<Item = &str> {
    let entries: &[(String, Json)] = match v {
        Json::Obj(e) => e,
        _ => &[],
    };
    entries.iter().map(|(k, _)| k.as_str())
}

/// How a message names an element: `component().name("R1")`, `trace()`.
fn at(el: &El) -> String {
    match el.attr_of("name") {
        Some(v @ Json::Str(_)) => format!("{}().name({})", fn_name(el.tag), v.text()),
        _ => format!("{}()", fn_name(el.tag)),
    }
}

fn refuse_unknown(el: &El, allowed: &[&str]) -> Result<(), String> {
    for (k, _) in &el.attrs {
        if allowed.contains(&k.as_str()) {
            continue;
        }
        if SCHEMATIC_ONLY.contains(&k.as_str()) {
            return Err(format!("{}: {} is the schematic's; a board says only where its parts sit and where its copper runs", at(el), rust_name(k)));
        }
        return Err(format!("{}: {} is not read on a board", at(el), rust_name(k)));
    }
    Ok(())
}

fn finite(v: &Json) -> Option<f64> {
    v.as_f64().filter(|n| n.is_finite())
}

fn num(el: &El, prop: &str, required: bool) -> Result<Option<f64>, String> {
    match el.attr_of(prop) {
        None if required => Err(format!("{} needs {}", at(el), rust_name(prop))),
        None => Ok(None),
        Some(v) => finite(v).map(Some).ok_or_else(|| format!("{}: {} is a number (mm or degrees), not {}", at(el), rust_name(prop), v.text())),
    }
}

fn positive(el: &El, prop: &str, required: bool) -> Result<Option<f64>, String> {
    let v = num(el, prop, required)?;
    if v.is_some_and(|v| v <= 0.0) {
        return Err(format!("{}: {} is more than 0", at(el), rust_name(prop)));
    }
    Ok(v)
}

fn text_of(el: &El, prop: &str, required: bool) -> Result<Option<String>, String> {
    match el.attr_of(prop) {
        None if required => Err(format!("{} needs {}", at(el), rust_name(prop))),
        None => Ok(None),
        Some(Json::Str(s)) => Ok(Some(s.clone())),
        Some(v) => Err(format!("{}: {} is text, not {}", at(el), rust_name(prop), v.text())),
    }
}

fn xy(x: f64, y: f64) -> Json {
    Json::obj().with("x", x).with("y", y)
}

/// `[x, y]` in mm, finite.
fn pair(v: &Json) -> Option<(f64, f64)> {
    match v {
        Json::Arr(p) if p.len() == 2 => Some((finite(&p[0])?, finite(&p[1])?)),
        _ => None,
    }
}

fn point_list(el: &El, prop: &str, least: usize, count: Option<usize>) -> Result<Vec<(f64, f64)>, String> {
    let list = match el.attr_of(prop) {
        Some(Json::Arr(list)) if list.len() >= least && count.is_none_or(|c| list.len() == c) => list,
        _ => {
            let n = count.map(|c| c.to_string()).unwrap_or_else(|| format!("at least {least}"));
            return Err(format!("{}: {} is a list of {n} [x, y]", at(el), rust_name(prop)));
        }
    };
    list.iter()
        .enumerate()
        .map(|(i, p)| pair(p).ok_or_else(|| format!("{}: point {i} is [x, y] in mm, not {}", at(el), p.text())))
        .collect()
}

fn at_point(el: &El, prop: &str) -> Result<Json, String> {
    let v = el.attr_of(prop).unwrap_or(&Json::Null);
    let (x, y) = pair(v).ok_or_else(|| format!("{}: {} is [x, y] in mm, not {}", at(el), rust_name(prop), if *v == Json::Null { "undefined".into() } else { v.text() }))?;
    Ok(xy(x, y))
}

fn copper(layer: &str) -> bool {
    layer.strip_suffix(".Cu").is_some_and(|h| !h.is_empty() && h.chars().all(|c| c.is_ascii_alphanumeric()))
}

fn copper_layers(el: &El, fallback: Option<&[&str]>) -> Result<Vec<String>, String> {
    let layers: Option<Vec<Json>> = match el.attr_of("layers") {
        Some(Json::Arr(l)) => Some(l.clone()),
        Some(_) => None,
        None => fallback.map(|f| f.iter().map(|l| Json::from(*l)).collect()),
    };
    match layers {
        Some(l) if !l.is_empty() && l.iter().all(|x| x.as_str().is_some_and(copper)) => Ok(l.iter().map(|x| x.as_str().unwrap_or_default().to_string()).collect()),
        _ => Err(format!("{}: layers is a list of copper layers (\"F.Cu\", \"In1.Cu\", \"B.Cu\")", at(el))),
    }
}

/// Carried KiCad forms, as s-expression text (`.kicad("(stroke (type solid))")`).
fn kicad_of(el: &El, prop: &str, inputs: &mut Vec<(String, Json)>) -> Result<(), String> {
    if let Some(v) = text_of(el, prop, false)? {
        inputs.push(("kicad".into(), v.into()));
    }
    Ok(())
}

/// Plain data an attribute holds, when `check` says it has the shape it needs.
fn plain(el: &El, prop: &str, check: impl Fn(&Json) -> bool, what: &str) -> Result<Option<Json>, String> {
    match el.attr_of(prop) {
        None => Ok(None),
        Some(v) if check(v) => Ok(Some(v.clone())),
        Some(_) => Err(format!("{}: {} is {what}", at(el), rust_name(prop))),
    }
}

fn is_object(v: &Json) -> bool {
    matches!(v, Json::Obj(_))
}

/// A flattened mask layout's polygons: each `{ layer, datatype, points }`, its points `[x, y]` in micrometres.
fn is_mask_polygons(v: &Json) -> bool {
    let Json::Arr(polygons) = v else { return false };
    !polygons.is_empty()
        && polygons.iter().all(|p| match p {
            Json::Obj(fields) => {
                let layer = fields.iter().find(|(k, _)| k == "layer").map(|(_, v)| v);
                let datatype = fields.iter().find(|(k, _)| k == "datatype").map(|(_, v)| v);
                let int = |v: Option<&Json>| matches!(v, Some(Json::Num(n)) if n.fract() == 0.0 && *n >= 0.0);
                let points = fields.iter().find(|(k, _)| k == "points").map(|(_, v)| v);
                int(layer)
                    && int(datatype)
                    && fields.iter().all(|(k, _)| ["layer", "datatype", "points"].contains(&k.as_str()))
                    && match points {
                        Some(Json::Arr(ps)) if ps.len() >= 3 => ps.iter().all(|q| matches!(q, Json::Arr(xy) if xy.len() == 2 && xy.iter().all(|c| matches!(c, Json::Num(n) if n.is_finite())))),
                        _ => false,
                    }
            }
            _ => false,
        })
}

fn is_str(v: Option<&Json>) -> bool {
    matches!(v, Some(Json::Str(_)))
}

/// A selector's name: letters, digits, `_ # -`.
fn ref_name(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '#' | '-'))
}

fn pin_name(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '+' | '-'))
}

/// `.REF` → REF.
fn one_selector(sel: &str) -> Option<&str> {
    let n = sel.trim().strip_prefix('.')?;
    ref_name(n).then_some(n)
}

/// `.REF > .pin` → (REF, pin).
fn two_selector(sel: &str) -> Option<(&str, &str)> {
    let (owner, which) = sel.trim().strip_prefix('.')?.split_once('>')?;
    let (owner, which) = (owner.trim_end(), which.trim_start().strip_prefix('.')?);
    (ref_name(owner) && pin_name(which)).then_some((owner, which))
}

/// A library footprint's ref: `Library:Footprint`.
fn footprint_ref(fp: &str) -> bool {
    match fp.split_once(':') {
        Some((lib, name)) => !lib.is_empty() && !name.is_empty() && !name.contains(':') && !name.contains('/'),
        None => false,
    }
}

fn declare(root: El) -> Result<Declared, String> {
    if root.attr_of("schematic").is_none() {
        return Err("board(): schematic is the schematic file's path, relative to this file (a board in code names its schematic)".into());
    }
    declare_board_file(&root, None).map(Declared::Graph)
}

/// Declare a board in code (its root is `board([…]).schematic("…")`).
pub fn declare_board_file(root: &El, name: Option<&str>) -> Result<Json, String> {
    refuse_unknown(root, &["schematic", "name", "width", "height", "core", "copper", "thickness", "layers", "surface"])?;
    let schematic = match root.attr_of("schematic") {
        Some(Json::Str(s)) if !s.trim().is_empty() && !s.starts_with('/') => s.clone(),
        _ => return Err("board(): schematic is the schematic file's path, relative to this file".into()),
    };
    let title = match root.attr_of("name") {
        Some(Json::Str(n)) => n.clone(),
        _ => name.unwrap_or("Board").to_string(),
    };
    let mut s = Scope::new(
        &format!("eda:{}", slug(&title)),
        Json::obj().with("domain", "eda").with("rung", "board").with("name", title.as_str()).with("schematic", schematic.as_str()),
    );
    let has_board = declare_board(&mut s, root)?;
    let mut graph = s.build();
    let outputs = if has_board { vec![Json::from("board")] } else { sinks(&graph) };
    if !outputs.is_empty() {
        let meta = graph.remove("meta").unwrap_or_else(Json::obj);
        graph.set("outputs", outputs);
        graph.set("meta", meta);
    }
    Ok(graph)
}

/// Nodes no other node wires from: a graph's terminals when it names none.
fn sinks(graph: &Json) -> Vec<Json> {
    let Some(Json::Obj(nodes)) = graph.get("nodes") else { return Vec::new() };
    let used: Vec<&str> = nodes
        .iter()
        .filter_map(|(_, n)| match n.get("inputs") {
            Some(Json::Obj(inputs)) => Some(inputs.iter().filter_map(|(_, v)| v.get("wire").and_then(|w| w.get("node")).and_then(Json::as_str))),
            _ => None,
        })
        .flatten()
        .collect();
    nodes.iter().filter(|(id, _)| !used.contains(&id.as_str())).map(|(id, _)| Json::from(id.as_str())).collect()
}

/// What a run's end or a pour's point lands on.
enum Owner<'a> {
    Component,
    Conductor(&'a El, String),
}

/// Declare the board's nodes; whether it declared the board node itself.
fn declare_board(s: &mut Scope, root: &El) -> Result<bool, String> {
    let children: Vec<&El> = declare_circuit_checks(&mut s.meta, root.child_elements().collect())?;
    let (w, h) = (positive(root, "width", false)?, positive(root, "height", false)?);
    let (core, copper_mm, thickness) = (positive(root, "core", false)?, positive(root, "copper", false)?, positive(root, "thickness", false)?);
    let layers = plain(
        root,
        "layers",
        |v| matches!(v, Json::Arr(l) if !l.is_empty() && l.iter().all(|l| is_object(l) && is_str(l.get("name")) && l.get("ordinal").and_then(finite).is_some())),
        "the layer table: [{ ordinal, name, type, userName }, …]",
    )?;
    if w.is_none() != h.is_none() {
        return Err("board(): give width and height together".into());
    }
    if core.is_none() != copper_mm.is_none() {
        return Err("board(): give core and copper together".into());
    }
    let stacks: Vec<&El> = children.iter().copied().filter(|el| el.tag == "stack").collect();
    if stacks.len() > 1 {
        return Err("a board has one stack()".into());
    }
    if !stacks.is_empty() && core.is_some() {
        return Err("board(): core and copper are a two-layer stack(); give one or the other".into());
    }
    if thickness.is_some() && core.is_some() {
        return Err("board(): core and copper say the thickness".into());
    }
    if core.is_some() && w.is_none() {
        return Err("board(): core and copper need the outline (width and height)".into());
    }
    let surface = plain(
        root,
        "surface",
        |v| match v {
            Json::Obj(entries) => is_str(v.get("cadRef")) && v.get("domain").is_some_and(is_object) && entries.iter().all(|(k, _)| ["cadRef", "domain", "trims", "curvedTrims"].contains(&k.as_str())),
            _ => false,
        },
        "the CAD faces the board is on: { cadRef, domain, trims, curvedTrims }",
    )?;
    let has_board = w.is_some()
        || thickness.is_some()
        || layers.is_some()
        || surface.is_some()
        || !stacks.is_empty()
        || children.iter().any(|el| ["graphic", "text", "dimension", "kicad"].contains(&el.tag));

    let mut stack_thickness = None;
    if let (Some(core), Some(cu)) = (core, copper_mm) {
        stack_thickness = Some(core + 2.0 * cu);
        let layer = |name: &str, role: &str, t: f64| Json::obj().with("name", name).with("role", role).with("thickness", t);
        let inputs = vec![
            ("id".to_string(), Json::from("stack")),
            ("domain".into(), "pcb".into()),
            ("units".into(), "mm".into()),
            ("layers".into(), Json::Arr(vec![layer("F.Cu", "conductor", cu), layer("core", "dielectric", core), layer("B.Cu", "conductor", cu)])),
        ];
        s.add("eda.stack", inputs, Some("stack"), Some("Cross-section"), meta(root))?;
    }
    for el in &stacks {
        refuse_unknown(el, &["name", "label", "domain", "units", "process", "layers", "overallThickness"])?;
        let stack_layers = plain(
            el,
            "layers",
            |v| matches!(v, Json::Arr(l) if l.iter().all(|l| is_object(l) && is_str(l.get("name")) && is_str(l.get("role")))),
            "the layers, top first: [{ name, role, thickness, … }, …]",
        )?
        .ok_or("stack() needs layers")?;
        let domain = text_of(el, "domain", false)?.unwrap_or_else(|| "pcb".into());
        let units = text_of(el, "units", false)?.unwrap_or_else(|| "mm".into());
        if domain != "pcb" && domain != "ic" {
            return Err("stack(): domain is \"pcb\" or \"ic\"".into());
        }
        if !["mm", "um", "nm"].contains(&units.as_str()) {
            return Err("stack(): units is \"mm\", \"um\" or \"nm\"".into());
        }
        let process = plain(el, "process", is_object, "an object ({ name, … })")?;
        let mut inputs = vec![
            ("id".to_string(), Json::from(text_of(el, "name", false)?.unwrap_or_else(|| "stack".into()))),
            ("domain".into(), domain.into()),
            ("units".into(), units.into()),
            ("layers".into(), stack_layers),
        ];
        if let Some(p) = process {
            inputs.push(("process".into(), p));
        }
        if el.attr_of("overallThickness").is_some() {
            inputs.push(("overallThickness".into(), positive(el, "overallThickness", false)?.unwrap_or_default().into()));
        }
        let label = text_of(el, "label", false)?.unwrap_or_else(|| "Cross-section".into());
        s.add("eda.stack", inputs, Some("stack"), Some(&label), meta(el))?;
    }
    if has_board {
        let layers = layers.unwrap_or_else(|| {
            Json::Arr(BOARD_LAYERS.iter().map(|(o, n, t)| Json::obj().with("ordinal", *o).with("name", *n).with("type", *t)).collect())
        });
        let mut inputs = vec![("layers".to_string(), layers)];
        if let Some(t) = stack_thickness.or(thickness) {
            inputs.push(("thicknessMm".into(), t.into()));
        }
        if core.is_some() || !stacks.is_empty() {
            inputs.push(("stack".into(), super::ir::wire("stack", "stack")));
        }
        if let (Some(w), Some(h)) = (w, h) {
            let outline = Json::obj()
                .with("id", "outline")
                .with("kind", "rect")
                .with("points", vec![xy(0.0, 0.0), xy(w, h)])
                .with("widthMm", 0.05)
                .with("layer", "Edge.Cuts")
                .with("filled", false);
            inputs.push(("boardArtwork".into(), Json::obj().with("graphics", vec![outline]).with("texts", Json::Arr(Vec::new()))));
        }
        if let Some(sf) = &surface {
            inputs.push(("surfaceMount".into(), sf.clone()));
        }
        s.add("eda.board", inputs, Some("board"), Some("Board"), meta(root))?;
    }

    // Names first: an end may name a trace or a via written after it.
    let mut components: Vec<String> = Vec::new();
    let mut conductors: HashMap<String, &El> = HashMap::new();
    let mut nets: Vec<String> = Vec::new();
    let mut constraint_names: Vec<String> = Vec::new();
    let mut kicad_forms = 0;
    let mut fabs = 0;
    for el in &children {
        let nm = el.attr_of("name");
        if el.tag == "component" {
            let nm = match nm {
                Some(Json::Str(n)) if !n.is_empty() => n.clone(),
                _ => return Err("component() needs the name of the schematic's part".into()),
            };
            if components.contains(&nm) || conductors.contains_key(&nm) {
                return Err(format!("two elements are called {nm}"));
            }
            components.push(nm);
        } else if CONDUCTORS.contains(&el.tag) {
            let Some(nm) = nm else { continue };
            let nm = match nm {
                Json::Str(n) if !n.is_empty() && n.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') => n.clone(),
                _ => return Err(format!("{}: a name is letters, digits, _ and -", at(el))),
            };
            if components.contains(&nm) || conductors.contains_key(&nm) {
                return Err(format!("two elements are called {nm}"));
            }
            conductors.insert(nm, el);
        } else if el.tag == "net" {
            let nm = match nm {
                Some(Json::Str(n)) if !n.is_empty() => n.clone(),
                _ => return Err("net() needs the name of the schematic's net".into()),
            };
            if nets.contains(&nm) {
                return Err(format!("two net() elements name {nm}"));
            }
            nets.push(nm);
        } else if el.tag == "kicad" {
            kicad_forms += 1;
            if kicad_forms > 1 {
                return Err("a board has one kicad()".into());
            }
        } else if el.tag == "fab" {
            fabs += 1;
            if fabs > 1 {
                return Err("a board has one fab()".into());
            }
        } else if CONSTRAINTS.contains(&el.tag) {
            let key = match (el.tag, nm) {
                ("objective", _) => Some("objective".to_string()),
                (tag, Some(Json::Str(n))) => Some(format!("{tag}:{n}")),
                _ => None,
            };
            if let Some(key) = key {
                if constraint_names.contains(&key) {
                    return Err(if el.tag == "objective" { "a board has one objective()".to_string() } else { format!("two {}() elements are called {}", el.tag, key.split_once(':').map(|(_, n)| n).unwrap_or_default()) });
                }
                constraint_names.push(key);
            }
        } else if el.tag != "stack" && el.tag != "masks" && !FACTS.contains(&el.tag) {
            return Err(format!("{}() is not read on a board (see commandagi::design::pcb)", fn_name(el.tag)));
        }
    }
    // Each conductor's node id: its name, else the next free `cu_<n>`, in the order they are written.
    let mut ids: Vec<String> = Vec::new();
    let mut n = 0;
    for el in &children {
        if !CONDUCTORS.contains(&el.tag) {
            continue;
        }
        match el.attr_of("name").and_then(Json::as_str) {
            Some(nm) => ids.push(nm.to_string()),
            None => loop {
                n += 1;
                let id = format!("cu_{n}");
                if !conductors.contains_key(&id) && !components.contains(&id) {
                    ids.push(id);
                    break;
                }
            },
        }
    }
    let mut ids = ids.into_iter();

    let owner_of = |name: &str| -> Option<Owner> {
        if components.iter().any(|c| c == name) {
            return Some(Owner::Component);
        }
        conductors.get(name).map(|el| Owner::Conductor(el, name.to_string()))
    };
    let point_count = |el: &El| -> Result<usize, String> { if el.tag == "arc" { Ok(3) } else { point_list(el, "points", 2, None).map(|p| p.len()) } };
    let end = |sel: &Json, el: &El, point: usize| -> Result<Json, String> {
        let Some(sel) = sel.as_str() else {
            return Err(format!("{}: an end is a selector (\".R1 > .pin2\", \".V1\", \".T1 > .end\")", at(el)));
        };
        let p = Json::obj().with("point", point as f64);
        if let Some(name) = one_selector(sel) {
            return match owner_of(name) {
                Some(Owner::Conductor(via, id)) if via.tag == "via" => Ok(p.with("via", id)),
                _ => Err(format!("{}: there is no via {name}", at(el))),
            };
        }
        let (owner, which) = two_selector(sel).ok_or_else(|| format!("{}: {} is not \".REF > .pin1\", \".VIA\" or \".TRACE > .end\"", at(el), Json::from(sel).text()))?;
        match owner_of(owner) {
            Some(Owner::Component) => {
                let number = match which.get(..3) {
                    Some(head) if head.eq_ignore_ascii_case("pin") && which.len() > 3 => &which[3..],
                    _ => which,
                };
                Ok(p.with("ref", owner).with("number", number))
            }
            Some(Owner::Conductor(run, id)) if run.tag == "trace" || run.tag == "arc" => {
                let count = point_count(run)?;
                let index = match which {
                    "start" => Some(0),
                    "end" => count.checked_sub(1),
                    w if w.chars().all(|c| c.is_ascii_digit()) => w.parse::<usize>().ok(),
                    _ => None,
                };
                match index {
                    Some(i) if i < count => Ok(p.with("run", id).with("runPoint", trace_point_id(i, count))),
                    _ => Err(format!("{}: trace {owner} has no point {which}", at(el))),
                }
            }
            _ => Err(format!("{}: there is no component or trace {owner}", at(el))),
        }
    };
    let net_of = |el: &El, inputs: &mut Vec<(String, Json)>| -> Result<(), String> {
        if let Some(v) = text_of(el, "net", false)? {
            if v.trim().is_empty() {
                return Err(format!("{}: net names the schematic's net", at(el)));
            }
            inputs.push(("net".into(), v.into()));
        }
        Ok(())
    };
    let uuid_of = |el: &El, inputs: &mut Vec<(String, Json)>| -> Result<(), String> {
        if let Some(v) = text_of(el, "uuid", false)? {
            inputs.push(("uuid".into(), v.into()));
        }
        Ok(())
    };

    for el in &children {
        let el: &El = el;
        match el.tag {
            "masks" => {
                refuse_unknown(el, &["name", "label", "libName", "unitUmPerDb", "polygons"])?;
                let polygons = plain(el, "polygons", is_mask_polygons, "the mask layout's polygons: [{ layer, datatype, points: [[x, y], …] }, …], points in micrometres")?
                    .ok_or("masks() needs polygons")?;
                let lib_name = text_of(el, "libName", false)?;
                let mut inputs = vec![("polygons".to_string(), polygons), ("unitUmPerDb".into(), positive(el, "unitUmPerDb", true)?.unwrap_or_default().into())];
                if let Some(l) = &lib_name {
                    inputs.push(("libName".into(), l.clone().into()));
                }
                let id = text_of(el, "name", false)?.unwrap_or_else(|| "masks".into());
                let label = text_of(el, "label", false)?.or_else(|| lib_name.clone()).unwrap_or_else(|| "Masks".into());
                s.add("eda.masks", inputs, Some(&id), Some(&label), meta(el))?;
            }
            "component" => {
                refuse_unknown(el, &["name", "footprint", "library", "pcbX", "pcbY", "pcbRotation", "layer", "locked", "refAt", "refRot", "valueAt", "chart", "uuid", "kicad"])?;
                let reference = el.attr_of("name").and_then(Json::as_str).unwrap_or_default().to_string();
                let fp = el.attr_of("footprint");
                let fp_text = fp.and_then(Json::as_str);
                let shown = |v: Option<&Json>| v.map(Json::text).unwrap_or_else(|| "undefined".into());
                let library = el.attr_of("library");
                if let Some(library) = library {
                    let pretty = library.as_str().is_some_and(|l| {
                        let l = l.to_lowercase();
                        l.ends_with(".pretty") || l.ends_with(".pretty/")
                    });
                    if !pretty {
                        return Err(format!("{}: library names a footprint library folder (a .pretty), not {}", at(el), library.text()));
                    }
                    if !fp_text.is_some_and(footprint_ref) {
                        return Err(format!("{}: a library footprint is its ref, \"Library:Footprint\" (\"Package_SO:SOIC-8\"), not {}", at(el), shown(fp)));
                    }
                } else if !fp_text.is_some_and(|f| BOARD_FOOTPRINTS.contains(&f) || footprint_ref(f)) {
                    return Err(format!(
                        "{}: footprint is one of {}, or a library footprint by its ref (\"Package_SO:SOIC-8\") with its library, not {}",
                        at(el),
                        BOARD_FOOTPRINTS.join(", "),
                        shown(fp)
                    ));
                }
                let fp = fp_text.unwrap_or_default();
                // A library footprint named with no library is named only: its pads are not on the board yet, so it is not placed.
                let named = library.is_none() && !BOARD_FOOTPRINTS.contains(&fp);
                if named && (el.attr_of("pcbX").is_some() || el.attr_of("pcbY").is_some()) {
                    return Err(format!("{}: a footprint is placed with its pads: name its library (a .pretty folder)", at(el)));
                }
                let (x, y, rot) = (num(el, "pcbX", false)?, num(el, "pcbY", false)?, num(el, "pcbRotation", false)?);
                let side = el.attr_of("layer");
                if side.is_some_and(|s| s.as_str() != Some("top") && s.as_str() != Some("bottom")) {
                    return Err(format!("{}: layer is \"top\" or \"bottom\"", at(el)));
                }
                if x.is_none() != y.is_none() {
                    return Err(format!("{}: give pcb_x and pcb_y together", at(el)));
                }
                if x.is_none() && (rot.is_some() || side.is_some()) {
                    return Err(format!("{}: pcb_rotation and layer need pcb_x and pcb_y", at(el)));
                }
                // Where the reference text sits, in the footprint's own frame (its library's, on the top): the placer
                // moves it off copper and off other references.
                let ref_at = el.attr_of("refAt");
                if ref_at.is_some_and(|v| !matches!(v, Json::Arr(l) if l.len() == 2 && l.iter().all(|n| finite(n).is_some()))) {
                    return Err(format!("{}: ref_at is [x, y] in mm, in the footprint's own frame", at(el)));
                }
                if ref_at.is_some() && x.is_none() {
                    return Err(format!("{}: ref_at needs pcb_x and pcb_y", at(el)));
                }
                // How far the reference text is turned where it sits: the placer turns a text it has no upright place for.
                let ref_rot = num(el, "refRot", false)?;
                if ref_rot.is_some() && x.is_none() {
                    return Err(format!("{}: ref_rot needs pcb_x and pcb_y", at(el)));
                }
                // Where its value text sits, in the same frame: the layout moves it too where the footprint draws it on
                // a silkscreen.
                let value_at = el.attr_of("valueAt");
                if value_at.is_some_and(|v| !matches!(v, Json::Arr(l) if l.len() == 2 && l.iter().all(|n| finite(n).is_some()))) {
                    return Err(format!("{}: value_at is [x, y] in mm, in the footprint's own frame", at(el)));
                }
                if value_at.is_some() && x.is_none() {
                    return Err(format!("{}: value_at needs pcb_x and pcb_y", at(el)));
                }
                let locked = match el.attr_of("locked") {
                    None => false,
                    Some(Json::Bool(true)) => true,
                    Some(_) => return Err(format!("{}: locked is true or left out", at(el))),
                };
                let chart = text_of(el, "chart", false)?;
                if chart.is_some() && surface.is_none() {
                    return Err(format!("{}: chart names a face of the board's surface, and the board() has no surface", at(el)));
                }
                if chart.is_some() && x.is_none() {
                    return Err(format!("{}: chart needs pcb_x and pcb_y", at(el)));
                }
                let mut inputs = vec![("ref".to_string(), Json::from(reference.as_str()))];
                match library.and_then(Json::as_str) {
                    Some(l) => {
                        inputs.push(("footprint".into(), fp.into()));
                        inputs.push(("library".into(), l.strip_suffix('/').unwrap_or(l).into()));
                    }
                    None if named => inputs.push(("footprint".into(), fp.into())),
                    None => inputs.push(("footprint".into(), format!("Authored:{fp}").into())),
                }
                if let (Some(x), Some(y)) = (x, y) {
                    let side = side.and_then(Json::as_str).unwrap_or("top");
                    inputs.push(("placement".into(), Json::obj().with("x", x).with("y", y).with("rot", rot.unwrap_or(0.0)).with("side", side)));
                }
                if locked {
                    inputs.push(("locked".into(), Json::Bool(true)));
                }
                if let Some(r) = ref_at {
                    inputs.push(("refAt".into(), r.clone()));
                }
                if let Some(r) = ref_rot {
                    inputs.push(("refRot".into(), Json::from(r)));
                }
                if let Some(r) = value_at {
                    inputs.push(("valueAt".into(), r.clone()));
                }
                if let Some(c) = chart {
                    inputs.push(("surfaceMount".into(), Json::obj().with("chart", c)));
                }
                uuid_of(el, &mut inputs)?;
                kicad_of(el, "kicad", &mut inputs)?;
                s.add(BOARD_PART, inputs, Some(&format!("fp_{reference}")), Some(&reference), meta(el))?;
            }
            "trace" | "arc" => {
                let is_trace = el.tag == "trace";
                let mut allowed = vec!["name", "layer", "width", "points", "from", "to", "net", "uuid", "kicad"];
                if is_trace {
                    allowed.extend(["segmentUuids", "surface", "rule"]);
                }
                refuse_unknown(el, &allowed)?;
                let rule = match el.attr_of("rule") {
                    None => "any".to_string(),
                    Some(Json::Str(r)) if TRACE_RULES.contains(&r.as_str()) => r.clone(),
                    Some(other) => return Err(format!("{}: rule is \"45\", \"90\" or \"any\", not {}", at(el), other.text())),
                };
                let on_surface = match el.attr_of("surface") {
                    None => false,
                    Some(Json::Bool(true)) => true,
                    Some(_) => return Err(format!("{}: surface is true or left out", at(el))),
                };
                if on_surface && surface.is_none() {
                    return Err(format!("{}: surface runs the trace on the board's surface, and the board() has no surface", at(el)));
                }
                let pts = if is_trace { point_list(el, "points", 2, None)? } else { point_list(el, "points", 3, Some(3))? };
                let layer = match el.attr_of("layer") {
                    Some(Json::Str(l)) if copper(l) => l.clone(),
                    _ => return Err(format!("{}: layer is a copper layer (\"F.Cu\", \"In1.Cu\", \"B.Cu\")", at(el))),
                };
                let count = pts.len();
                let mut terminals = Vec::new();
                if let Some(from) = el.attr_of("from") {
                    terminals.push(end(from, el, 0)?);
                }
                if let Some(to) = el.attr_of("to") {
                    terminals.push(end(to, el, count - 1)?);
                }
                let segment_uuids = plain(el, "segmentUuids", |v| matches!(v, Json::Arr(l) if l.iter().all(|u| u.as_str().is_some())), "a list of uuids, one per segment")?;
                let points: Vec<Json> = pts.iter().enumerate().map(|(i, (x, y))| xy(*x, *y).with("id", trace_point_id(i, count))).collect();
                let mut inputs = vec![
                    ("kind".to_string(), Json::from(if is_trace { "run" } else { "arc" })),
                    ("points".into(), points.into()),
                    ("widthMm".into(), positive(el, "width", true)?.unwrap_or_default().into()),
                    ("layer".into(), layer.into()),
                ];
                // A run is routed under its rule; an arc has none.
                if is_trace {
                    inputs.push(("rule".into(), rule.into()));
                }
                if !terminals.is_empty() {
                    inputs.push(("terminals".into(), terminals.into()));
                }
                if let Some(u) = segment_uuids {
                    inputs.push(("segmentUuids".into(), u));
                }
                if on_surface {
                    let domain = surface.as_ref().and_then(|sf| sf.get("domain")).cloned().unwrap_or(Json::Null);
                    inputs.push(("mount".into(), Json::obj().with("kind", "unwrap").with("domain", domain)));
                }
                uuid_of(el, &mut inputs)?;
                net_of(el, &mut inputs)?;
                kicad_of(el, "kicad", &mut inputs)?;
                let id = ids.next().expect("each conductor has an id");
                s.add(CONDUCTOR, inputs, Some(&id), Some(if is_trace { "Trace" } else { "Arc" }), meta(el))?;
            }
            "via" => {
                refuse_unknown(el, &["name", "pcbX", "pcbY", "drill", "diameter", "layers", "pads", "net", "uuid", "kicad"])?;
                let layers = copper_layers(el, Some(&["F.Cu", "B.Cu"]))?;
                if layers.len() < 2 {
                    return Err(format!("{}: layers is two or more copper layers", at(el)));
                }
                let drill = positive(el, "drill", true)?.unwrap_or_default();
                let diameter = positive(el, "diameter", true)?.unwrap_or_default();
                if diameter <= drill {
                    return Err(format!("{}: the diameter is more than the drill", at(el)));
                }
                let pads: Vec<Json> = match el.attr_of("pads") {
                    None => Vec::new(),
                    Some(Json::Arr(p)) if !p.is_empty() => p.clone(),
                    Some(_) => return Err(format!("{}: pads is a list of pins (\".R1 > .pin2\")", at(el))),
                };
                let mut pad_refs = Vec::new();
                for p in &pads {
                    let t = end(p, el, 0)?;
                    match (t.get("ref"), t.get("number")) {
                        (Some(r), Some(n)) => pad_refs.push(Json::obj().with("ref", r.clone()).with("number", n.clone())),
                        _ => return Err(format!("{}: a via is plated into a pad (\".R1 > .pin2\"), not {}", at(el), p.text())),
                    }
                }
                let (x, y) = (num(el, "pcbX", true)?.unwrap_or_default(), num(el, "pcbY", true)?.unwrap_or_default());
                let mut inputs = vec![
                    ("kind".to_string(), Json::from("via")),
                    ("points".into(), vec![xy(x, y)].into()),
                    ("drillMm".into(), drill.into()),
                    ("padDiameterMm".into(), diameter.into()),
                    ("layers".into(), layers.into_iter().map(Json::from).collect::<Vec<_>>().into()),
                ];
                if !pad_refs.is_empty() {
                    inputs.push(("pads".into(), pad_refs.into()));
                }
                uuid_of(el, &mut inputs)?;
                net_of(el, &mut inputs)?;
                kicad_of(el, "kicad", &mut inputs)?;
                let id = ids.next().expect("each conductor has an id");
                s.add(CONDUCTOR, inputs, Some(&id), Some("Via"), meta(el))?;
            }
            "pour" => {
                refuse_unknown(el, &["name", "layers", "points", "terminals", "net", "uuid", "kicad"])?;
                let pts = point_list(el, "points", 3, None)?;
                let raw: Vec<Json> = match el.attr_of("terminals") {
                    None => Vec::new(),
                    Some(Json::Arr(list))
                        if list.iter().all(|t| matches!(t, Json::Arr(p) if p.len() == 2 && p[0].as_f64().is_some_and(|n| n.is_finite() && n.fract() == 0.0) && p[1].as_str().is_some())) =>
                    {
                        list.clone()
                    }
                    Some(_) => return Err(format!("{}: terminals is a list of [point, selector] ([[0, \".J1 > .pin2\"], …])", at(el))),
                };
                let mut terminals = Vec::new();
                for t in &raw {
                    let Json::Arr(p) = t else { continue };
                    let point = p[0].as_f64().unwrap_or_default();
                    if point < 0.0 || point >= pts.len() as f64 {
                        return Err(format!("{}: the pour has no point {}", at(el), p[0].text()));
                    }
                    terminals.push(end(&p[1], el, point as usize)?);
                }
                let mut inputs = vec![
                    ("kind".to_string(), Json::from("pour")),
                    ("points".into(), pts.iter().map(|(x, y)| xy(*x, *y)).collect::<Vec<_>>().into()),
                    ("layers".into(), copper_layers(el, None)?.into_iter().map(Json::from).collect::<Vec<_>>().into()),
                ];
                if !terminals.is_empty() {
                    inputs.push(("terminals".into(), terminals.into()));
                }
                uuid_of(el, &mut inputs)?;
                net_of(el, &mut inputs)?;
                kicad_of(el, "kicad", &mut inputs)?;
                let id = ids.next().expect("each conductor has an id");
                s.add(CONDUCTOR, inputs, Some(&id), Some("Pour"), meta(el))?;
            }
            "graphic" => {
                refuse_unknown(el, &["kind", "layer", "points", "width", "filled", "arcs", "id", "kicad"])?;
                let kind = text_of(el, "kind", true)?.unwrap_or_default();
                let Some((_, count)) = GRAPHICS.iter().find(|(k, _)| *k == kind) else {
                    let kinds: Vec<&str> = GRAPHICS.iter().map(|(k, _)| *k).collect();
                    return Err(format!("{}: kind is {}", at(el), kinds.join(", ")));
                };
                let filled = match el.attr_of("filled") {
                    None => None,
                    Some(Json::Bool(b)) => Some(*b),
                    Some(_) => return Err(format!("{}: filled is true or false", at(el))),
                };
                let open = kind == "line" || kind == "arc";
                if filled.is_some() && open {
                    return Err(format!("{}: a {kind} is not filled", at(el)));
                }
                let mut inputs = vec![("fact".to_string(), Json::from("graphic"))];
                if el.attr_of("id").is_some() {
                    inputs.push(("id".into(), text_of(el, "id", false)?.unwrap_or_default().into()));
                }
                inputs.push(("kind".into(), kind.as_str().into()));
                let least = if kind == "poly" { 2 } else { count.unwrap_or(2) };
                let corners = point_list(el, "points", least, *count)?;
                inputs.push(("points".into(), corners.iter().map(|(x, y)| xy(*x, *y)).collect::<Vec<_>>().into()));
                // A polygon's arcs: where each one's three points (start, mid, end) begin in `points`.
                if let Some(arcs) = el.attr_of("arcs") {
                    let mut next = 0usize;
                    let starts: Option<Vec<usize>> = match arcs {
                        Json::Arr(list) if kind == "poly" && !list.is_empty() => list
                            .iter()
                            .map(|v| match v {
                                Json::Num(n) if n.fract() == 0.0 && *n >= next as f64 && (*n as usize) + 3 <= corners.len() => {
                                    next = *n as usize + 3;
                                    Some(*n as usize)
                                }
                                _ => None,
                            })
                            .collect(),
                        _ => None,
                    };
                    if starts.is_none() {
                        return Err(format!("{}: arcs is, on a poly, the increasing indices in points where an arc's start, mid and end begin (3 points each, not overlapping)", at(el)));
                    }
                    inputs.push(("arcStarts".into(), arcs.clone()));
                }
                inputs.push(("widthMm".into(), num(el, "width", true)?.unwrap_or_default().into()));
                inputs.push(("layer".into(), text_of(el, "layer", true)?.unwrap_or_default().into()));
                if !open {
                    inputs.push(("filled".into(), filled.unwrap_or(false).into()));
                }
                kicad_of(el, "kicad", &mut inputs)?;
                s.add(BOARD_FACT, inputs, None, Some("Graphic"), meta(el))?;
            }
            "text" => {
                refuse_unknown(el, &["text", "field", "at", "rotation", "layer", "size", "sizeX", "thickness", "kind", "id", "kicad"])?;
                let (t, field) = (text_of(el, "text", false)?, text_of(el, "field", false)?);
                if t.is_none() == field.is_none() {
                    return Err("text(): give text, or field (\"reference\" or \"value\")".into());
                }
                if field.as_ref().is_some_and(|f| f != "reference" && f != "value") {
                    return Err("text(): field is \"reference\" or \"value\"".into());
                }
                let mut inputs = vec![("fact".to_string(), Json::from("text"))];
                if el.attr_of("id").is_some() {
                    inputs.push(("id".into(), text_of(el, "id", false)?.unwrap_or_default().into()));
                }
                match (t, field) {
                    (Some(t), _) => inputs.push(("text".into(), t.into())),
                    (None, Some(f)) => inputs.push(("field".into(), f.into())),
                    (None, None) => {}
                }
                inputs.push(("at".into(), at_point(el, "at")?));
                inputs.push(("rot".into(), num(el, "rotation", false)?.unwrap_or(0.0).into()));
                inputs.push(("layer".into(), text_of(el, "layer", true)?.unwrap_or_default().into()));
                let size = num(el, "size", true)?.unwrap_or_default();
                inputs.push(("size".into(), size.into()));
                inputs.push(("sizeX".into(), num(el, "sizeX", false)?.unwrap_or(size).into()));
                inputs.push(("thickness".into(), num(el, "thickness", true)?.unwrap_or_default().into()));
                inputs.push(("kind".into(), text_of(el, "kind", false)?.unwrap_or_else(|| "text".into()).into()));
                kicad_of(el, "kicad", &mut inputs)?;
                s.add(BOARD_FACT, inputs, None, Some("Text"), meta(el))?;
            }
            "dimension" => {
                refuse_unknown(el, &["points", "height", "layer", "width", "textSize", "id", "kicad"])?;
                let mut inputs = vec![
                    ("fact".to_string(), Json::from("dimension")),
                    ("id".into(), text_of(el, "id", false)?.unwrap_or_default().into()),
                    ("points".into(), point_list(el, "points", 2, Some(2))?.iter().map(|(x, y)| xy(*x, *y)).collect::<Vec<_>>().into()),
                    ("height".into(), num(el, "height", true)?.unwrap_or_default().into()),
                    ("layer".into(), text_of(el, "layer", true)?.unwrap_or_default().into()),
                    ("widthMm".into(), num(el, "width", true)?.unwrap_or_default().into()),
                    ("textSize".into(), num(el, "textSize", true)?.unwrap_or_default().into()),
                ];
                kicad_of(el, "kicad", &mut inputs)?;
                s.add(BOARD_FACT, inputs, None, Some("Dimension"), meta(el))?;
            }
            "kicad" => {
                refuse_unknown(el, &["version", "generator", "forms"])?;
                let mut inputs = vec![("fact".to_string(), Json::from("kicad"))];
                if let Some(v) = num(el, "version", false)? {
                    inputs.push(("version".into(), v.into()));
                }
                if let Some(g) = text_of(el, "generator", false)? {
                    inputs.push(("generator".into(), g.into()));
                }
                kicad_of(el, "forms", &mut inputs)?;
                s.add(BOARD_FACT, inputs, None, Some("KiCad"), meta(el))?;
            }
            "net" => {
                refuse_unknown(el, &["name", "code"])?;
                let code = num(el, "code", true)?.unwrap_or_default();
                if code.fract() != 0.0 || code < 0.0 {
                    return Err(format!("{}: code is a KiCad net code (0 or more)", at(el)));
                }
                let name = el.attr_of("name").cloned().unwrap_or(Json::Null);
                let inputs = vec![("fact".to_string(), Json::from("net")), ("name".into(), name), ("code".into(), code.into())];
                s.add(BOARD_FACT, inputs, None, Some("Net"), meta(el))?;
            }
            "fab" => {
                s.add(BOARD_FACT, fab_fact(el)?, None, Some("Fab"), meta(el))?;
            }
            tag if CONSTRAINTS.contains(&tag) => {
                s.add(BOARD_FACT, constraint_fact(el)?, None, Some(tag), meta(el))?;
            }
            _ => {}
        }
    }
    Ok(has_board)
}
