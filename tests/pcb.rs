// A board in Rust declares the board half of a circuit the TypeScript SDK's pcb.test.ts pins for the same elements:
// the outline and cross-section, each component's footprint and placement, and the copper with what its ends land on;
// ids that do not change between runs, each call's site in `meta.source`, and the named refusals.

use commandagi::design::pcb::*;
use commandagi::design::{declare, El, Json};

fn board_of(children: Vec<El>) -> El {
    board(children).schematic("Divider.sch.rs")
}

fn graph(root: El) -> Json {
    declare(root).unwrap().graph().unwrap()
}

fn bad(root: El) -> String {
    declare(root).unwrap_err()
}

fn node<'a>(g: &'a Json, id: &str) -> &'a Json {
    g.get("nodes").and_then(|n| n.get(id)).unwrap_or_else(|| panic!("no node {id}"))
}

fn inputs(g: &Json, id: &str) -> String {
    node(g, id).get("inputs").unwrap().text()
}

fn compact(text: &str) -> String {
    Json::parse(text).unwrap().text()
}

#[test]
fn a_board_declares_its_outline_stack_footprints_and_copper() {
    let l0 = line!() + 1;
    let root = board_of(vec![
        component().name("R1").footprint("smd-0805").pcb_x(10).pcb_y(10),
        component().name("R2").footprint("axial-7.62").pcb_x(25).pcb_y(10).pcb_rotation(90).layer("bottom"),
        component().name("V1").footprint("smd-0805"),
        trace().layer("F.Cu").width(0.2).points([[11, 10], [18, 14], [24, 10]]).from(".R1 > .pin2").to(".R2 > .1"),
        trace().layer("B.Cu").width(0.25).points([[18, 14], [18, 20]]).from(".VIA1"),
        via().name("VIA1").pcb_x(18).pcb_y(14).drill(0.4).diameter(0.8),
    ])
    .width(40)
    .height(30)
    .core(1.5)
    .copper(0.035);
    let g = graph(root);
    assert_eq!(g.get("meta").and_then(|m| m.get("schematic")), Some(&Json::from("Divider.sch.rs")));
    assert_eq!(g.get("outputs").unwrap().text(), r#"["board"]"#);
    let board = node(&g, "board").get("inputs").unwrap();
    assert_eq!(board.get("thicknessMm"), Some(&Json::Num(1.57)));
    assert_eq!(board.get("stack").unwrap().text(), r#"{"wire":{"node":"stack","port":"stack"}}"#);
    assert_eq!(board.get("boardArtwork").unwrap().text(), compact(r#"{"graphics":[{"id":"outline","kind":"rect","points":[{"x":0,"y":0},{"x":40,"y":30}],"widthMm":0.05,"layer":"Edge.Cuts","filled":false}],"texts":[]}"#));
    assert_eq!(
        node(&g, "stack").get("inputs").and_then(|i| i.get("layers")).unwrap().text(),
        compact(r#"[{"name":"F.Cu","role":"conductor","thickness":0.035},{"name":"core","role":"dielectric","thickness":1.5},{"name":"B.Cu","role":"conductor","thickness":0.035}]"#)
    );
    let site = |l: u32, c: u32| Json::obj().with("source", Json::obj().with("site", vec![Json::from(l), Json::from(c)])).text();
    // The root's site is its call in `board_of`.
    assert_eq!(node(&g, "board").get("meta").unwrap().text(), site(9, 5));
    assert_eq!(
        node(&g, "fp_R1").text(),
        compact(&format!(r#"{{"id":"fp_R1","type":"eda.footprint","inputs":{{"ref":"R1","footprint":"Authored:smd-0805","placement":{{"x":10,"y":10,"rot":0,"side":"top"}}}},"label":"R1","meta":{}}}"#, site(l0 + 1, 9)))
    );
    assert_eq!(node(&g, "fp_R2").get("inputs").and_then(|i| i.get("placement")).unwrap().text(), r#"{"x":25,"y":10,"rot":90,"side":"bottom"}"#);
    assert_eq!(node(&g, "fp_V1").get("inputs").and_then(|i| i.get("placement")), None, "a footprint with no pcb_x and pcb_y is not placed");
    assert_eq!(
        inputs(&g, "cu_1"),
        compact(r#"{"kind":"run","points":[{"x":11,"y":10,"id":"start"},{"x":18,"y":14,"id":"p1"},{"x":24,"y":10,"id":"end"}],"widthMm":0.2,"layer":"F.Cu","rule":"any","terminals":[{"point":0,"ref":"R1","number":"2"},{"point":2,"ref":"R2","number":"1"}]}"#)
    );
    assert_eq!(node(&g, "cu_1").get("meta").unwrap().text(), site(l0 + 4, 9));
    assert_eq!(node(&g, "cu_2").get("inputs").and_then(|i| i.get("terminals")).unwrap().text(), r#"[{"point":0,"via":"VIA1"}]"#);
}

#[test]
fn an_end_names_a_via_or_another_traces_point_written_before_or_after_it() {
    let g = graph(board_of(vec![
        trace().layer("F.Cu").width(0.2).points([[0, 0], [5, 0]]).to(".VIA1"),
        trace().name("T1").layer("F.Cu").width(0.2).points([[5, 0], [5, 5], [9, 5]]),
        trace().layer("F.Cu").width(0.2).points([[5, 5], [5, 9]]).from(".T1 > .1"),
        via().name("VIA1").pcb_x(5).pcb_y(0).drill(0.3).diameter(0.6),
    ]));
    assert!(g.get("nodes").and_then(|n| n.get("board")).is_none(), "a board with no width and height declares no outline yet");
    assert_eq!(g.get("outputs").unwrap().text(), r#"["cu_1","T1","cu_2","VIA1"]"#, "with no board, every node is a terminal");
    assert_eq!(node(&g, "cu_1").get("inputs").and_then(|i| i.get("terminals")).unwrap().text(), r#"[{"point":1,"via":"VIA1"}]"#);
    assert_eq!(node(&g, "cu_2").get("inputs").and_then(|i| i.get("terminals")).unwrap().text(), r#"[{"point":0,"run":"T1","runPoint":"p1"}]"#);
    assert_eq!(inputs(&g, "VIA1"), r#"{"kind":"via","points":[{"x":5,"y":0}],"drillMm":0.3,"padDiameterMm":0.6,"layers":["F.Cu","B.Cu"]}"#);
}

#[test]
fn a_library_footprint_is_its_ref_and_the_pretty_folder_that_holds_it() {
    let g = graph(board_of(vec![component().name("U1").footprint("Package_SO:SOIC-8").library("footprints.pretty").pcb_x(12).pcb_y(8).pcb_rotation(90)]));
    assert_eq!(inputs(&g, "fp_U1"), r#"{"ref":"U1","footprint":"Package_SO:SOIC-8","library":"footprints.pretty","placement":{"x":12,"y":8,"rot":90,"side":"top"}}"#);
    assert!(bad(board_of(vec![component().name("U1").footprint("SOIC-8").library("footprints.pretty")])).contains(r#"its ref, "Library:Footprint""#));
    assert!(bad(board_of(vec![component().name("U1").footprint("Package_SO:SOIC-8").library("SOIC-8.kicad_mod")])).contains("a footprint library folder (a .pretty)"));
}

#[test]
fn a_board_refuses_by_name_what_it_cannot_say() {
    assert!(bad(board_of(vec![component().name("R1").footprint("0603")])).contains("footprint is one of smd-0805"));
    assert_eq!(
        bad(board_of(vec![component().name("R1").footprint("smd-0805").with_attr("schX", 3)])),
        r#"component().name("R1"): sch_x is the schematic's; a board says only where its parts sit and where its copper runs"#
    );
    assert!(bad(board_of(vec![component().name("R1").footprint("smd-0805").pcb_x(3)])).ends_with("give pcb_x and pcb_y together"));
    assert_eq!(bad(board_of(vec![El::new("resistor").with_attr("name", "R1")])), "resistor() is not read on a board (see commandagi::design::pcb)");
    assert_eq!(bad(board_of(vec![]).width(10)), "board(): give width and height together");
    assert!(bad(board_of(vec![trace().layer("F.SilkS").width(0.2).points([[0, 0], [1, 0]])])).contains("copper layer"));
    assert!(bad(board_of(vec![arc().layer("F.Cu").width(0.2).points([[0, 0], [1, 0]])])).ends_with("points is a list of 3 [x, y]"));
    assert!(bad(board_of(vec![stack().layers(Vec::<i32>::new())]).core(1).copper(0.03).width(5).height(5)).contains("core and copper are a two-layer stack()"));
    assert_eq!(bad(board_of(vec![kicad(), kicad()])), "a board has one kicad()");
    assert!(bad(board_of(vec![text().at([0, 0]).layer("F.SilkS").size(1).thickness(0.1)])).contains("give text, or field"));
    assert!(bad(board_of(vec![trace().layer("F.Cu").width(0.2).points([[0, 0], [1, 0]]).from(".R9 > .pin1")])).ends_with("there is no component or trace R9"));
    assert_eq!(bad(board_of(vec![component().name("V1").footprint("smd-0805"), via().name("V1").pcb_x(0).pcb_y(0).drill(0.3).diameter(0.6)])), "two elements are called V1");
    assert!(bad(board([]).schematic("/abs.sch.rs")).contains("relative to this file"));
    assert!(bad(board([])).contains("a board in code names its schematic"));
}

#[test]
fn a_board_says_what_a_kicad_board_holds() {
    let layers = r#"[{"ordinal": 0, "name": "F.Cu", "type": "signal"}, {"ordinal": 4, "name": "In1.Cu", "type": "signal"}, {"ordinal": 2, "name": "B.Cu", "type": "signal"}, {"ordinal": 25, "name": "Edge.Cuts", "type": "user"}]"#;
    let g = graph(
        board_of(vec![
            stack().name("four-layer").label("JLC 4-layer").process(json(r#"{"name": "JLC"}"#)).layers(json(r#"[{"name": "F.Cu", "role": "conductor", "thickness": 0.035}]"#)),
            kicad().version(20260206).generator("pcbnew").forms(r#"(paper "A4")"#),
            graphic().kind("line").layer("Edge.Cuts").points([[0, 0], [40, 0]]).width(0.1).id("e0").kicad("(stroke (type solid))"),
            text().text("REV A").at([2, 3]).layer("F.SilkS").size(1).thickness(0.15),
            net().name("GND").code(1),
            component().name("C1").footprint("Lib:C_0805").library("Board.pretty").pcb_x(4).pcb_y(5).uuid("u-c1"),
            trace().layer("In1.Cu").width(0.2).points([[0, 0], [5, 0]]).net("GND").uuid("u-t1").kicad("(locked yes)"),
            arc().name("A1").layer("F.Cu").width(0.2).points([[0, 0], [1, 1], [2, 0]]).to(".C1 > .pin1"),
            via().name("V1").pcb_x(5).pcb_y(0).drill(0.3).diameter(0.6).layers(["F.Cu", "In1.Cu", "B.Cu"]).pads([".C1 > .pin2"]).net("GND"),
            pour().layers(["In1.Cu"]).points([[0, 0], [9, 0], [9, 9]]).terminals(json(r#"[[0, ".V1"]]"#)).net("GND").kicad("(min_thickness 0.25)"),
        ])
        .layers(json(layers)),
    );
    assert_eq!(inputs(&g, "board"), compact(&format!(r#"{{"layers":{layers},"stack":{{"wire":{{"node":"stack","port":"stack"}}}}}}"#)));
    let mut stack = node(&g, "stack").clone();
    stack.remove("meta");
    assert_eq!(stack.text(), compact(r#"{"id":"stack","type":"eda.stack","inputs":{"id":"four-layer","domain":"pcb","units":"mm","layers":[{"name":"F.Cu","role":"conductor","thickness":0.035}],"process":{"name":"JLC"}},"label":"JLC 4-layer"}"#));
    let Some(Json::Obj(nodes)) = g.get("nodes") else { panic!() };
    let facts: Vec<String> = nodes.iter().filter(|(_, n)| n.get("type") == Some(&Json::from("eda.boardfact"))).map(|(_, n)| n.get("inputs").unwrap().text()).collect();
    assert_eq!(
        facts,
        [
            compact(r#"{"fact":"kicad","version":20260206,"generator":"pcbnew","kicad":"(paper \"A4\")"}"#),
            compact(r#"{"fact":"graphic","id":"e0","kind":"line","points":[{"x":0,"y":0},{"x":40,"y":0}],"widthMm":0.1,"layer":"Edge.Cuts","kicad":"(stroke (type solid))"}"#),
            compact(r#"{"fact":"text","text":"REV A","at":{"x":2,"y":3},"rot":0,"layer":"F.SilkS","size":1,"sizeX":1,"thickness":0.15,"kind":"text"}"#),
            compact(r#"{"fact":"net","name":"GND","code":1}"#),
        ]
    );
    assert_eq!(node(&g, "fp_C1").get("inputs").and_then(|i| i.get("uuid")), Some(&Json::from("u-c1")));
    assert_eq!(inputs(&g, "cu_1"), compact(r#"{"kind":"run","points":[{"x":0,"y":0,"id":"start"},{"x":5,"y":0,"id":"end"}],"widthMm":0.2,"layer":"In1.Cu","rule":"any","uuid":"u-t1","net":"GND","kicad":"(locked yes)"}"#));
    assert_eq!(inputs(&g, "A1"), compact(r#"{"kind":"arc","points":[{"x":0,"y":0,"id":"start"},{"x":1,"y":1,"id":"p1"},{"x":2,"y":0,"id":"end"}],"widthMm":0.2,"layer":"F.Cu","terminals":[{"point":2,"ref":"C1","number":"1"}]}"#));
    assert_eq!(node(&g, "V1").get("inputs").and_then(|i| i.get("pads")).unwrap().text(), r#"[{"ref":"C1","number":"2"}]"#);
    assert_eq!(inputs(&g, "cu_2"), compact(r#"{"kind":"pour","points":[{"x":0,"y":0},{"x":9,"y":0},{"x":9,"y":9}],"layers":["In1.Cu"],"terminals":[{"point":0,"via":"V1"}],"net":"GND","kicad":"(min_thickness 0.25)"}"#));
}

#[test]
fn a_board_on_the_faces_of_a_cad_part() {
    let surface = r#"{"cadRef": "../case.3d.tsx", "domain": {"charts": [{"id": "top"}]}}"#;
    let g = graph(
        board_of(vec![
            component().name("R1").footprint("smd-0805").pcb_x(4).pcb_y(5).chart("top"),
            trace().layer("F.Cu").width(0.3).points([[4, 5], [9, 5]]).from(".R1 > .pin1").surface(true),
        ])
        .surface(json(surface)),
    );
    assert_eq!(node(&g, "board").get("inputs").and_then(|i| i.get("surfaceMount")).unwrap().text(), compact(surface));
    assert_eq!(node(&g, "fp_R1").get("inputs").and_then(|i| i.get("surfaceMount")).unwrap().text(), r#"{"chart":"top"}"#);
    assert_eq!(node(&g, "cu_1").get("inputs").and_then(|i| i.get("mount")).unwrap().text(), r#"{"kind":"unwrap","domain":{"charts":[{"id":"top"}]}}"#);
    assert!(bad(board_of(vec![component().name("R1").footprint("smd-0805").pcb_x(4).pcb_y(5).chart("top")])).contains("has no surface"));
    assert!(bad(board_of(vec![trace().layer("F.Cu").width(0.3).points([[4, 5], [9, 5]]).surface(true)])).contains("has no surface"));
}
