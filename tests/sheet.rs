// The schematic vocabulary against the graph the Python and TypeScript SDKs declare for the same sheet
// (`divider.json`: the Python SDK's run of the shipped Divider.sch.py, `meta.source` taken out).

use commandagi::design::sheet::*;
use commandagi::design::{declare, Json};

fn divider() -> Group {
    group("Divider", [
        voltagesource("V1").voltage("9").sch_x(114.3).sch_y(114.3),
        resistor("R1").resistance("3k").sch_x(114.3).sch_y(88.9),
        resistor("R2").resistance("1.5k").sch_x(139.7).sch_y(88.9),
        ground("#PWR1").sch_x(139.7).sch_y(114.3),
        trace(".V1 > .pos", ".R1 > .pin1"),
        trace(".R1 > .pin2", ".R2 > .pin1"),
        trace(".R2 > .pin2", ".#PWR1 > .pin1"),
        trace(".V1 > .neg", ".#PWR1 > .pin1"),
        netlabel("OUT", ".R1 > .pin2"),
    ])
}

fn without_sources(mut graph: Json) -> Json {
    if let Some(Json::Obj(nodes)) = graph.get("nodes").cloned() {
        let nodes = nodes.into_iter().map(|(id, mut n)| {
            n.remove("meta");
            (id, n)
        });
        graph.set("nodes", Json::Obj(nodes.collect()));
    }
    graph
}

#[test]
fn declares_the_graph_the_other_sdks_declare() {
    let graph = declare(divider()).unwrap();
    let golden = Json::parse(include_str!("divider.json")).unwrap();
    assert_eq!(without_sources(graph).text(), golden.text());
}

#[test]
fn every_node_carries_the_call_it_came_from() {
    let graph = declare(divider()).unwrap();
    let Some(Json::Obj(nodes)) = graph.get("nodes") else { panic!("no nodes") };
    // R1's part and its placement both come from the call in line 10 of this file, column 9.
    for id in ["R1", "sym_R1_1"] {
        let node = &nodes.iter().find(|(k, _)| k == id).unwrap().1;
        let site = node.get("meta").and_then(|m| m.get("source")).and_then(|s| s.get("site")).unwrap();
        assert_eq!(site, &Json::Arr(vec![10u32.into(), 9u32.into()]), "{id}");
    }
}

#[test]
fn a_call_in_a_loop_counts_each_evaluation() {
    let ladder = group("Ladder", (1..=3).map(|i| resistor(format!("R{i}")).resistance(1000).sch_x(i * 10).sch_y(0)));
    let graph = declare(ladder).unwrap();
    let r2 = graph.get("nodes").and_then(|n| n.get("R2")).unwrap();
    assert_eq!(r2.get("inputs").and_then(|i| i.get("value")), Some(&Json::from("1000")));
    let sites = commandagi::design::source::sites().text();
    assert!(sites.contains("[53,50,3]"), "{sites}");
}

#[test]
fn a_fragment_is_its_elements() {
    let sheet = group("F", [resistor("R1").sch_x(0).sch_y(0), fragment((2..=3).map(|i| resistor(format!("R{i}")).sch_x(i).sch_y(0)))]);
    let graph = declare(sheet).unwrap();
    let ids: Vec<String> = match graph.get("nodes") {
        Some(Json::Obj(n)) => n.iter().map(|(k, _)| k.clone()).collect(),
        _ => vec![],
    };
    assert_eq!(ids, ["R1", "sym_R1_1", "R2", "sym_R2_1", "R3", "sym_R3_1"]);
}

#[test]
fn refuses_what_the_sheet_cannot_say() {
    let bad = |g: Group| declare(g).unwrap_err();
    assert_eq!(bad(group("S", [ground("PWR1")])), "ground(\"PWR1\"): a ground symbol's name starts with # (#PWR1), as KiCad names power symbols");
    assert_eq!(bad(group("S", [junction("J1").resistance("1k")])), "junction(\"J1\"): resistance is not read on a schematic");
    assert_eq!(bad(group("S", [resistor("R1").sch_x(1).sch_y(2).sch_rotation(45)])), "resistor(\"R1\"): sch_rotation is 0, 90, 180 or 270");
    assert_eq!(bad(group("S", [resistor("R1"), trace(".R1 > .pin1", "net.GND")])), "trace(): R1 is not on the sheet (give it sch_x and sch_y)");
    assert_eq!(bad(group("S", [resistor("R1").sch_x(0).sch_y(0), trace(".R1 > .pin3", "net.GND")])), "trace(): R1 has no pin pin3");
    assert_eq!(bad(group("S", [resistor("R1"), resistor("R1")])), "two parts are called R1");
    assert!(bad(group("S", [voltagesource("V1").excitation("{oops")])).starts_with("voltagesource(\"V1\"): excitation is JSON text"));
}

#[test]
fn an_empty_group_declares_an_empty_sheet() {
    let graph = declare(group("Empty", [])).unwrap();
    assert_eq!(graph.text(), r#"{"id":"eda:Empty","nodes":{},"meta":{"domain":"eda","rung":"board","name":"Empty"}}"#);
}

fn node(graph: &Json, id: &str) -> Json {
    let mut n = graph.get("nodes").and_then(|n| n.get(id)).unwrap_or_else(|| panic!("no node {id}")).clone();
    n.remove("meta");
    n
}

#[test]
fn a_library_part_names_its_symbol_by_ref_places_each_unit_mirrors_and_a_code_part_is_a_code_node() {
    let sheet = group("Rail", [
        part("U1").symbol("Amplifier_Operational:LM358").library("opamps.kicad_sym").value("LM358").sch_x(50.8).sch_y(25.4).sch_mirror("x"),
        unit("U1", 2).sch_x(101.6).sch_y(25.4).sch_rotation(180),
        resistor("R1").resistance("10k").sch_x(76.2).sch_y(50.8).sch_mirror("y"),
        code("blinker").source("blinker.circuit.ts").inputs(r#"{"resistor": "330"}"#),
        trace(".U1 > .pin7", ".R1 > .pin1"),
        netlabel("OUT", ".U1 > .1"),
    ]);
    let g = declare(sheet).unwrap();
    let u1 = node(&g, "U1");
    assert_eq!(u1.get("type").and_then(Json::as_str), Some(part_type_for(&[]).as_str()));
    assert_eq!(u1.get("inputs").unwrap().text(), r#"{"ref":"U1","value":"LM358","symbol":"Amplifier_Operational:LM358","library":"opamps.kicad_sym","pins":[]}"#, "the pins are the library's");
    let sym1 = node(&g, "sym_U1_1");
    assert_eq!(sym1.get("type").and_then(Json::as_str), Some(sch_symbol_type_for(&[]).as_str()));
    assert_eq!(sym1.get("inputs").unwrap().text(), r#"{"unit":1,"style":1,"at":{"x":50.8,"y":25.4},"rot":0,"mirror":"x","part":{"wire":{"node":"U1","port":"@part"}}}"#);
    assert_eq!(node(&g, "sym_U1_2").get("inputs").unwrap().text(), r#"{"unit":2,"style":1,"at":{"x":101.6,"y":25.4},"rot":180,"mirror":"","part":{"wire":{"node":"U1","port":"@part"}}}"#);
    // A unit's placement maps to its unit(…) call, one line below the part's.
    let line = |id: &str| match g.get("nodes").and_then(|n| n.get(id)).and_then(|n| n.get("meta")).and_then(|m| m.get("source")).and_then(|s| s.get("site")) {
        Some(Json::Arr(v)) => v[0].as_f64().unwrap(),
        other => panic!("{id} has no site: {other:?}"),
    };
    assert_eq!(line("sym_U1_2"), line("sym_U1_1") + 1.0);
    assert_eq!(node(&g, "sym_R1_1").get("inputs").and_then(|i| i.get("mirror")), Some(&Json::from("y")));
    assert_eq!(node(&g, "w_1").get("inputs").unwrap().text(), r#"{"ends.1":{"wire":{"node":"U1","port":"pin:7"}},"ends.2":{"wire":{"node":"sym_R1_1","port":"p1"}}}"#, "a library pin by number, bound by the editor");
    assert_eq!(node(&g, "lbl_OUT").get("inputs").and_then(|i| i.get("on")).unwrap().text(), r#"{"wire":{"node":"U1","port":"pin:1"}}"#);
    assert_eq!(node(&g, "blinker").text(), r#"{"id":"blinker","type":"code","inputs":{"source":"blinker.circuit.ts","resistor":"330"},"label":"blinker.circuit.ts"}"#);
}

#[test]
fn refuses_what_a_library_part_unit_or_code_part_cannot_say() {
    let bad = |g: Group| declare(g).unwrap_err();
    assert!(bad(group("S", [part("U1").symbol("LM358").library("a.kicad_sym")])).contains("library ref"));
    assert!(bad(group("S", [part("U1").symbol("A:B")])).contains("library is the path"));
    assert!(bad(group("S", [resistor("R1").sch_x(0).sch_y(0), unit("R1", 2).sch_x(1).sch_y(1)])).contains("one unit"));
    assert!(bad(group("S", [part("U1").symbol("A:B").library("a.kicad_sym"), unit("U1", 1).sch_x(1).sch_y(1)])).contains("unit is 2 or more"));
    assert!(bad(group("S", [part("U1").symbol("A:B").library("a.kicad_sym"), unit("U1", 2)])).contains("give it sch_x and sch_y"));
    assert!(bad(group("S", [part("U1").symbol("A:B").library("a.kicad_sym"), unit("U1", 2).sch_x(1).sch_y(1), unit("U1", 2).sch_x(2).sch_y(1)])).contains("placed twice"));
    assert!(bad(group("S", [resistor("R1").sch_x(0).sch_y(0).sch_mirror("z")])).contains(r#"sch_mirror is "x" or "y""#));
    assert!(bad(group("S", [code("c").source("c.ts").inputs(r#"{"source": "d.ts"}"#)])).contains("source is the file"));
}
