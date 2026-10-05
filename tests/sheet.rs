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
