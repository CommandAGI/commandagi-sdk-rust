// The schematic vocabulary against the graph the Python and TypeScript SDKs declare for the same sheet
// (`divider.json`: the Python SDK's run of the shipped Divider.sch.py, `meta.source` taken out).

use commandagi::design::schematic::*;
use commandagi::design::{declare, Json};

fn divider() -> El {
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

fn graph(root: El) -> Json {
    declare(root).unwrap().graph().unwrap()
}

fn bad(root: El) -> String {
    declare(root).unwrap_err()
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
    let golden = Json::parse(include_str!("divider.json")).unwrap();
    assert_eq!(without_sources(graph(divider())).text(), golden.text());
}

#[test]
fn every_node_carries_the_call_it_came_from() {
    let g = graph(divider());
    let Some(Json::Obj(nodes)) = g.get("nodes") else { panic!("no nodes") };
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
    let g = graph(ladder);
    let r2 = g.get("nodes").and_then(|n| n.get("R2")).unwrap();
    assert_eq!(r2.get("inputs").and_then(|i| i.get("value")), Some(&Json::from("1000")));
    let sites = commandagi::design::source::sites().text();
    assert!(sites.contains(&format!("[{},50,3]", line!() - 5)), "{sites}");
}

#[test]
fn a_fragment_is_its_elements() {
    let sheet = group("F", [resistor("R1").sch_x(0).sch_y(0), fragment((2..=3).map(|i| resistor(format!("R{i}")).sch_x(i).sch_y(0)))]);
    let ids: Vec<String> = match graph(sheet).get("nodes") {
        Some(Json::Obj(n)) => n.iter().map(|(k, _)| k.clone()).collect(),
        _ => vec![],
    };
    assert_eq!(ids, ["R1", "sym_R1_1", "R2", "sym_R2_1", "R3", "sym_R3_1"]);
}

#[test]
fn refuses_what_the_sheet_cannot_say() {
    assert_eq!(bad(group("S", [ground("PWR1")])), "ground(\"PWR1\"): a ground symbol's name starts with # (#PWR1), as KiCad names power symbols");
    assert_eq!(bad(group("S", [junction("J1").resistance("1k")])), "junction(\"J1\"): resistance is not read on a schematic");
    assert_eq!(bad(group("S", [resistor("R1").sch_x(1).sch_y(2).sch_rotation(45)])), "resistor(\"R1\"): sch_rotation is 0, 90, 180 or 270");
    assert_eq!(bad(group("S", [resistor("R1").sch_x("1").sch_y(2)])), "resistor(\"R1\"): sch_x is a number (mm or degrees), not \"1\"");
    assert_eq!(bad(group("S", [resistor("R1"), trace(".R1 > .pin1", "net.GND")])), "trace(\".R1 > .pin1\", \"net.GND\"): R1 is not on the sheet (give it sch_x and sch_y)");
    assert_eq!(bad(group("S", [resistor("R1").sch_x(0).sch_y(0), trace(".R1 > .pin3", "net.GND")])), "trace(\".R1 > .pin3\", \"net.GND\"): R1 has no pin pin3");
    assert_eq!(bad(group("S", [resistor("R1"), resistor("R1")])), "two parts are called R1");
    assert_eq!(bad(group("S", [voltagesource("V1").excitation(json("{oops"))])), "voltagesource(\"V1\"): excitation is JSON text (expected a string at 1)");
    assert_eq!(bad(group("", [])), "group() needs a name");
}

#[test]
fn an_empty_group_declares_an_empty_sheet() {
    assert_eq!(graph(group("Empty", [])).text(), r#"{"id":"eda:Empty","nodes":{},"meta":{"domain":"eda","rung":"board","name":"Empty"}}"#);
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
        code("blinker").source("blinker.circuit.ts").inputs(json(r#"{"resistor": "330"}"#)),
        trace(".U1 > .pin7", ".R1 > .pin1"),
        netlabel("OUT", ".U1 > .1"),
    ]);
    let g = graph(sheet);
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
    assert!(bad(group("S", [part("U1").symbol("LM358").library("a.kicad_sym")])).contains("library ref"));
    assert!(bad(group("S", [part("U1").symbol("A:B")])).contains("library is the path"));
    assert!(bad(group("S", [resistor("R1").sch_x(0).sch_y(0), unit("R1", 2).sch_x(1).sch_y(1)])).contains("one unit"));
    assert!(bad(group("S", [part("U1").symbol("A:B").library("a.kicad_sym"), unit("U1", 1).sch_x(1).sch_y(1)])).contains("unit is 2 or more"));
    assert!(bad(group("S", [part("U1").symbol("A:B").library("a.kicad_sym"), unit("U1", 2)])).contains("give it sch_x and sch_y"));
    assert!(bad(group("S", [part("U1").symbol("A:B").library("a.kicad_sym"), unit("U1", 2).sch_x(1).sch_y(1), unit("U1", 2).sch_x(2).sch_y(1)])).contains("placed twice"));
    assert!(bad(group("S", [resistor("R1").sch_x(0).sch_y(0).sch_mirror("z")])).contains(r#"sch_mirror is "x" or "y""#));
    assert!(bad(group("S", [code("c").source("c.ts").inputs(json(r#"{"source": "d.ts"}"#))])).contains("source is the file"));
}

#[test]
fn a_netlist_circuit_lists_its_parts_pins_and_nets_and_carries_its_kicad_files() {
    let circuit = group("Board", [
        part("U1").value("LM358").symbol("Amplifier_Operational:LM358").pins(json(r#"[{"number": "1", "name": "OUT"}, {"number": "8", "name": "V+"}]"#)),
        part("C1").value("100n").pins(["1", "2"]).spice(json(r#"[{"id": "c", "model": "C100N", "terminals": {"a": "1", "b": "2"}}]"#)),
        net("VCC").pins([".U1 > .pin8", ".C1 > .1"]),
        attachment("Board.kicad_sch").role("schematic").file("Board.kicad_sch"),
    ]);
    let g = graph(circuit);
    let u1 = node(&g, "U1");
    assert_eq!(u1.get("inputs").unwrap().text(), r#"{"ref":"U1","value":"LM358","symbol":"Amplifier_Operational:LM358","pins":[{"id":"p1","number":"1","name":"OUT"},{"id":"p2","number":"8","name":"V+"}]}"#);
    assert_eq!(u1.get("type").and_then(Json::as_str), Some(part_type_for(&[("p1".into(), "1".into()), ("p2".into(), "8".into())]).as_str()));
    assert!(node(&g, "C1").get("inputs").unwrap().text().contains(r#""spice":[{"id":"c","model":"C100N","terminals":{"a":"1","b":"2"}}]"#));
    assert_eq!(node(&g, "net_VCC").text(), r#"{"id":"net_VCC","type":"eda.net","inputs":{"pins.1":{"wire":{"node":"U1","port":"p2"}},"pins.2":{"wire":{"node":"C1","port":"p1"}}},"label":"VCC"}"#);
    assert_eq!(node(&g, "source_Board.kicad_sch").text(), r#"{"id":"source_Board.kicad_sch","type":"eda.source","inputs":{"name":"Board.kicad_sch","role":"schematic","file":"Board.kicad_sch"},"label":"Board.kicad_sch"}"#);
    assert!(bad(group("S", [part("U1").pins(["1"]).sch_x(0).sch_y(0)])).contains("has no place on the sheet"));
    assert!(bad(group("S", [part("U1").pins(["1"]), net("N").pins([".U1 > .2"])])).contains("U1 has no pin 2"));
    assert!(bad(group("S", [attachment("a").role("art").file("a")])).contains("role is one of"));
    assert!(bad(group("S", [part("C1").pins(["1"]).spice(json(r#"[{"id": "c"}]"#))])).contains("spice is a list"));
}

#[test]
fn a_trace_path_runs_from_its_from_to_its_to() {
    let sheet = |path: [&str; 3]| {
        group("P", [
            resistor("R1").sch_x(0).sch_y(0),
            junction("J1").sch_x(5).sch_y(5),
            resistor("R2").sch_x(10).sch_y(0),
            trace(".R1 > .pin2", ".R2 > .pin1").path(path),
        ])
    };
    let g = graph(sheet([".R1 > .pin2", ".J1", ".R2 > .pin1"]));
    assert_eq!(node(&g, "w_2").get("inputs").unwrap().text(), r#"{"ends.1":{"wire":{"node":"J1","port":"v"}},"ends.2":{"wire":{"node":"sym_R2_1","port":"p1"}}}"#);
    assert!(bad(sheet([".R1 > .pin1", ".J1", ".R2 > .pin1"])).contains("path is the list of selectors from"));
}

#[test]
fn a_part_names_the_pins_it_leaves_open_on_purpose_by_pin_number() {
    let g = graph(group("Open", [
        resistor("R1").resistance("1k").sch_x(0).sch_y(0).no_connect(["2"]),
        part("K2").symbol("station:SSR").library("station.kicad_sym").sch_x(50.8).sch_y(0).no_connect(["3", "4"]),
        part("J1").pins(["1", "2", "3"]).no_connect(["3"]),
    ]));
    assert!(node(&g, "R1").get("inputs").unwrap().text().contains(r#""noConnect":["2"]"#));
    assert_eq!(node(&g, "K2").get("inputs").unwrap().text(), r#"{"ref":"K2","symbol":"station:SSR","library":"station.kicad_sym","pins":[],"noConnect":["3","4"]}"#);
    assert!(node(&g, "J1").get("inputs").unwrap().text().contains(r#""noConnect":["3"]"#));
    assert!(bad(group("S", [part("K2").symbol("a:B").library("a.kicad_sym").no_connect(["3", "3"])])).contains("noConnect is a list of pin numbers"));
    assert!(bad(group("S", [part("K2").symbol("a:B").library("a.kicad_sym").no_connect(json("[3]"))])).contains("noConnect is a list of pin numbers"));
}

/// A part's field as the part carries it: KiCad's `property` form, as the board editor writes one (compact JSON).
fn field_form(name: &str, value: &str) -> String {
    let sym = |v: &str| format!(r#"{{"kind":"atom","v":"{v}","q":false}}"#);
    let text = |v: &str| format!(r#"{{"kind":"atom","v":"{v}","q":true}}"#);
    let list = |items: Vec<String>| format!(r#"{{"kind":"list","items":[{}]}}"#, items.join(","));
    list(vec![
        sym("property"),
        text(name),
        text(value),
        list(vec![sym("at"), sym("0"), sym("0"), sym("0")]),
        list(vec![sym("effects"), list(vec![sym("font"), list(vec![sym("size"), sym("1.27"), sym("1.27")])]), list(vec![sym("hide"), sym("yes")])]),
    ])
}

#[test]
fn a_part_states_its_engineering_data_as_kicad_property_forms() {
    let g = graph(group("Fields", [
        resistor("R1").resistance("3k").sch_x(0).sch_y(0).part_number("MF-R-3K").manufacturer("Yageo").mpn("RC0805FR-073KL").rating("0.125 W"),
        part("K2").symbol("station:SSR").library("station.kicad_sym").sch_x(50.8).sch_y(0).mpn("G3NA-225B").attrs(json(r#"{"Tolerance": "1%"}"#)),
        part("J1").pins(["1", "2"]).part_number("MF-J-2"),
    ]));
    let foreign = |id: &str| node(&g, id).get("inputs").unwrap().get("foreign").unwrap().text();
    let r1 = [field_form("CommandAGI IPN", "MF-R-3K"), field_form("Manufacturer", "Yageo"), field_form("MPN", "RC0805FR-073KL"), field_form("Rating", "0.125 W")];
    assert_eq!(foreign("R1"), format!("[{}]", r1.join(",")));
    assert_eq!(foreign("K2"), format!("[{},{}]", field_form("MPN", "G3NA-225B"), field_form("Tolerance", "1%")));
    assert_eq!(foreign("J1"), format!("[{}]", field_form("CommandAGI IPN", "MF-J-2")));
    let k2 = || part("K2").symbol("a:B").library("a.kicad_sym");
    assert!(bad(group("S", [k2().mpn("")])).contains(r#"mpn is text ("RC0805FR-073KL")"#));
    assert!(bad(group("S", [k2().mpn(3)])).contains(r#"mpn is text ("RC0805FR-073KL")"#));
    assert!(bad(group("S", [k2().attrs(json(r#"{"Tolerance": 1}"#))])).contains(r#"attrs is an object of text ({ Tolerance: "1%" })"#));
    // One field, one way to say it: a field the part has an argument for, or its own name, value or footprint.
    assert!(bad(group("S", [k2().attrs(json(r#"{"MPN": "X"}"#))])).contains("attrs names MPN: say it with mpn"));
    assert!(bad(group("S", [k2().attrs(json(r#"{"CommandAGI IPN": "X"}"#))])).contains("attrs names CommandAGI IPN: say it with part_number"));
    assert!(bad(group("S", [k2().attrs(json(r#"{"Value": "X"}"#))])).contains("attrs names Value, which is the part's own"));
}

#[test]
fn how_a_parts_pinout_and_pads_are_known_and_its_package_are_fields_off_the_grammar_refused() {
    let checked = "datasheet https://www.ti.com/lit/ds/symlink/sn74lvc1t45.pdf p. 3, 2026-10-15";
    let pkg = "6 pins, pitch 0.95 mm, body 2.9 x 1.6 mm";
    let g = graph(group("S", [part("U5").pins(["1", "2", "3", "4", "5", "6"]).pinout_verified(checked).footprint_verified("measured 2026-10-15, calipers").package_dimensions(pkg)]));
    let foreign = node(&g, "U5").get("inputs").unwrap().get("foreign").unwrap().text();
    let want = [field_form("Pinout Verified", checked), field_form("Footprint Verified", "measured 2026-10-15, calipers"), field_form("Package Dimensions", pkg)];
    assert_eq!(foreign, format!("[{}]", want.join(",")));
    let u5 = || part("U5").pins(["1", "2", "3"]);
    for wrong in ["checked", "datasheet p. 3, 2026-10-15", "datasheet https://x.test/a.pdf p. 3", "measured yesterday"] {
        assert!(bad(group("S", [u5().pinout_verified(wrong)])).contains("pinout_verified is text"), "{wrong}");
    }
    assert!(bad(group("S", [u5().package_dimensions("SOT-23")])).contains("package_dimensions is text"));
    assert!(bad(group("S", [u5().attrs(json(r#"{"Pinout Verified": "memory"}"#))])).contains("say it with pinout_verified"));
}

fn without_source(list: &Json) -> String {
    match list {
        Json::Arr(items) => Json::Arr(
            items
                .iter()
                .cloned()
                .map(|mut c| {
                    c.remove("source");
                    c
                })
                .collect(),
        )
        .text(),
        other => other.text(),
    }
}

#[test]
fn a_circuits_checks_and_accepts_are_kept_in_the_graphs_meta_not_as_nodes() {
    let line = line!() + 3;
    let g = graph(group("S", [
        resistor("R1").resistance("1k").sch_x(0).sch_y(0),
        check().id("erc").kind("erc").severity("warning"),
        accept().id("open").check("erc").rule("pin_not_connected").refs(["R1"]).reason("a test point"),
    ]));
    let meta = g.get("meta").unwrap();
    assert_eq!(without_source(meta.get("checks").unwrap()), r#"[{"id":"erc","kind":"erc","severity":"warning"}]"#);
    assert_eq!(without_source(meta.get("accepted").unwrap()), r#"[{"id":"open","check":"erc","rule":"pin_not_connected","refs":["R1"],"reason":"a test point"}]"#);
    // Each carries the call it came from, as a node does, so an editor can write it back.
    let Some(Json::Arr(checks)) = meta.get("checks") else { panic!() };
    assert_eq!(checks[0].get("source").and_then(|s| s.get("site")), Some(&Json::Arr(vec![line.into(), 9u32.into()])));
    let Some(Json::Obj(nodes)) = g.get("nodes") else { panic!() };
    assert!(nodes.iter().all(|(_, n)| !matches!(n.get("type").and_then(Json::as_str), Some("check" | "accept"))));
    // The circuit is the same with them or without them.
    let plain = graph(group("S", [resistor("R1").resistance("1k").sch_x(0).sch_y(0)]));
    assert_eq!(without_sources(g.clone()).get("nodes").unwrap().text(), without_sources(plain).get("nodes").unwrap().text());
}

#[test]
fn a_wrong_check_or_accept_is_refused_by_name() {
    let r1 = || resistor("R1").resistance("1k");
    assert!(bad(group("S", [r1(), check().id("erc").kind("erc").resistance(1)])).contains(r#"check().id("erc"): resistance is not read (it takes id, kind, severity, options)"#));
    assert!(bad(group("S", [r1(), check().id("erc").kind("erc").severity("fatal")])).contains(r#"severity is "error" or "warning""#));
    assert!(bad(group("S", [r1(), check().kind("erc")])).contains("check(): id is the check's name (erc)"));
    assert!(bad(group("S", [r1(), check().id("a b").kind("erc")])).contains("id is letters, digits, _ . -"));
    assert!(bad(group("S", [r1(), check().id("erc").kind("erc"), accept().id("erc").check("erc").rule("x").reason("y")])).contains("two checks or accepts are called erc"));
    assert!(bad(group("S", [r1(), accept().id("a").check("erc").rule("x")])).contains(r#"accept().id("a"): reason is why the finding is right on purpose"#));
    assert!(bad(group("S", [r1(), accept().id("a").check("erc").rule("x").refs(Vec::<&str>::new()).reason("y")])).contains("refs is a list of part references"));
    assert!(bad(group("S", [r1(), accept().id("a").check("erc").rule("x").region(json(r#"{"min": [0]}"#)).reason("y")])).contains("region is"));
}
