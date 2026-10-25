// Systems in Rust against the documents the TypeScript SDK's systems.test.ts pins for the same elements: a requirements
// document, a system and an interface, each record an element with its fields verbatim; a wrong field refused by name.

use commandagi::design::systems::*;
use commandagi::design::{declare, Declared};

fn document(root: El) -> (String, Json) {
    match declare(root).unwrap() {
        Declared::Document { format, document, .. } => (format, document),
        Declared::Graph(_) => panic!("a systems document is a document"),
    }
}

fn bad(root: El) -> String {
    declare(root).unwrap_err()
}

fn j(text: &str) -> Json {
    Json::parse(text).unwrap()
}

#[test]
fn a_requirements_document_is_its_requirements_with_refs() {
    let (format, doc) = document(
        requirements([
            requirement().id("mass").text("At most 500 g.").method("analysis").verified_by("Rover.system.rs#mass").design(["cad/Rover.3d.rs"]),
            requirement().id("drop").text("Survives a 1 m drop.").method("test").rationale("a person drops it"),
        ])
        .name("Rover"),
    );
    assert_eq!(format, "requirements");
    assert_eq!(
        doc,
        j(r#"{"name": "Rover", "requirements": [
            {"id": "mass", "text": "At most 500 g.", "method": "analysis", "verifiedBy": "Rover.system.rs#mass", "design": ["cad/Rover.3d.rs"]},
            {"id": "drop", "text": "Survives a 1 m drop.", "method": "test", "rationale": "a person drops it"}]}"#)
    );
    assert!(bad(requirements([requirement().id("x").text("t").method("review")])).contains(r#"requirement().id("x"): method must be one of test, analysis, inspection, demonstration (got "review")"#));
    assert!(bad(requirements([requirement().id("a b").text("t")])).contains("id is letters, digits"));
    assert!(bad(requirements([requirement().id("x")])).contains("needs text"));
    assert!(bad(requirements([requirement().id("x").text("t").verified_by(3)])).contains("verifiedBy is a ref or a list of refs"));
}

#[test]
fn a_system_is_its_records_as_written() {
    let (format, doc) = document(
        system([
            mode().id("lte"),
            rail().id("+5V").voltage(5).source("USB"),
            rail().id("VBAT").voltage(3.8),
            converter().id("U3").input("+5V").output("VBAT").efficiency(0.88).source("estimate"),
            load().id("modem").part("U4").rail("VBAT").mode("lte").current(0.67).peak_current(0.9).source("datasheet table 35"),
            contribution().id("shell").quantity("mass").part("P1").value(15.2).unit("g").source("check-fit.mjs"),
            bom().ref_("bom.csv"),
            budget().id("power").quantity("power").rail("+5V").mode("lte").limit(10).unit("W").margin(0.2),
            budget().id("peak").quantity("power").rail("+5V").mode("lte").peak(true).limit(10).unit("W"),
            check().id("link").kind("interface").of("Link.interface.rs"),
        ])
        .name("Cam"),
    );
    assert_eq!(format, "system");
    let keys: Vec<String> = match &doc {
        Json::Obj(entries) => entries.iter().map(|(k, _)| k.clone()).collect(),
        _ => panic!(),
    };
    assert_eq!(keys, ["name", "modes", "rails", "converters", "loads", "contributions", "boms", "budgets", "checks"]);
    assert_eq!(doc.get("rails"), Some(&j(r#"[{"id": "+5V", "voltage": 5, "source": "USB"}, {"id": "VBAT", "voltage": 3.8}]"#)));
    assert_eq!(doc.get("boms"), Some(&j(r#"[{"ref": "bom.csv"}]"#)));
    assert_eq!(doc.get("loads"), Some(&j(r#"[{"id": "modem", "part": "U4", "rail": "VBAT", "mode": "lte", "current": 0.67, "peakCurrent": 0.9, "source": "datasheet table 35"}]"#)));
    let cases: Vec<(El, &str)> = vec![
        (budget().id("b").quantity("mass").limit(1).peak(true), "peak is true or false, on a power budget"),
        (load().id("l").part("U1").rail("r").mode("m").current(1).peak_current("2 A"), "peakCurrent is a number"),
        (load().id("l").part("U1").rail("r").mode("m").power(1).duty(1.5), "duty is the share of time the load draws"),
        (budget().id("b").quantity("Exhaust flow").limit(1).unit("m3/h"), r#"budget().id("b"): quantity must be one of mass, power, cost, thermal, or a quantity's name"#),
        (budget().id("b").quantity("exhaust-flow").limit(600), r#"budget().id("b"): a quantity of exhaust-flow states its unit"#),
        (contribution().id("c").quantity("exhaust-flow").part("P").value(35).source("datasheet"), r#"contribution().id("c"): a quantity of exhaust-flow states its unit"#),
        (budget().id("b").quantity("power").limit(1), "a power budget names its rail and its mode"),
        (budget().id("b").quantity("thermal").limit(1).mode("m"), "thermal resistance to the ambient (resistance, K/W)"),
        (budget().id("b").quantity("mass").limit(1).margin(1.5), "margin is a fraction"),
        (budget().id("b").quantity("cost").per(100).limit(30), "a cost budget names the design whose logistics roll-up it reads"),
        (contribution().id("c").quantity("cost").part("P").value(1).source("a quote"), "a cost is stated in the design's logistics"),
        (converter().id("c").input("a").output("b").efficiency(88), "efficiency is a number in (0, 1]"),
        (load().id("l").part("U1").rail("r").mode("m"), "a load states its current (A) or its power (W)"),
        (contribution().id("c").quantity("mass").part("P").value(1), "source says where the number was read"),
        (check().id("k").kind("erc").of("x"), "kind must be one of interface, trace"),
        (mode().id("m").with_attr("watts", 3), "watts is not read"),
    ];
    for (el, message) in cases {
        let said = bad(system([el]));
        assert!(said.contains(message), "{said}");
    }
}

#[test]
fn an_interface_is_its_frame_pins_holes_messages_params_and_sides() {
    let (format, doc) = document(
        interface([
            frame().description("the base board's top, pin 1 at the origin").unit("mm"),
            pin().number(1).signal("+5V").required(true).at([-11.43, 0.0]),
            pin().number(4).signal("MODEM_TXD").gpio(13).dir("in"),
            hole().id("m2-a").at([0.0, 0.0]).diameter(2.2),
            message().id("lte-only").text("AT+CNMP=38"),
            param().id("baud").value(115200).unit("Bd"),
            param().id("framing").value("8N1"),
            side().id("base").ref_("ee/base.circuit.ts").parts(["J1", "J2"]).origin([0.0, 1.4]).rotation(90.0).face("back"),
            side().id("firmware").ref_("fw/config.h").defines(json(r#"{"BOARD": 1}"#)).pins(json(r#"{"RX_PIN": 4}"#)).params(json(r#"{"BAUD": "baud"}"#)),
            side().id("host").ref_("devices/cam/definition.tsx").channel("modem").params(json(r#"{"baud": "baud"}"#)),
        ])
        .name("Header"),
    );
    assert_eq!(format, "interface");
    assert_eq!(doc.get("frame"), Some(&j(r#"{"description": "the base board's top, pin 1 at the origin", "unit": "mm"}"#)));
    let keys: Vec<String> = match &doc {
        Json::Obj(entries) => entries.iter().map(|(k, _)| k.clone()).collect(),
        _ => panic!(),
    };
    assert_eq!(keys, ["name", "frame", "pins", "holes", "messages", "params", "sides"]);
    assert_eq!(
        doc.get("params").and_then(|s| if let Json::Arr(a) = s { a.get(1).cloned() } else { None }),
        Some(j(r#"{"id": "framing", "value": "8N1"}"#)),
        "a text a channel states has no unit"
    );
    assert_eq!(
        doc.get("sides").and_then(|s| if let Json::Arr(a) = s { a.get(0).cloned() } else { None }),
        Some(j(r#"{"id": "base", "ref": "ee/base.circuit.ts", "parts": ["J1", "J2"], "origin": [0.0, 1.4], "rotation": 90.0, "face": "back"}"#))
    );
    assert_eq!(
        doc.get("sides").and_then(|s| if let Json::Arr(a) = s { a.get(1).cloned() } else { None }),
        Some(j(r#"{"id": "firmware", "ref": "fw/config.h", "defines": {"BOARD": 1}, "pins": {"RX_PIN": 4}, "params": {"BAUD": "baud"}}"#))
    );
    assert_eq!(
        doc.get("sides").and_then(|s| if let Json::Arr(a) = s { a.get(2).cloned() } else { None }),
        Some(j(r#"{"id": "host", "ref": "devices/cam/definition.tsx", "channel": "modem", "params": {"baud": "baud"}}"#))
    );
    let cases: Vec<(El, &str)> = vec![
        (pin().number(0).signal("x"), "number is the pin's number, 1, 2, 3"),
        (pin().number(1).signal("x").at([1.0]), "at is [x, y] mm"),
        (hole().id("h").at([0.0, 0.0, 0.0]), r#"hole().id("h"): at is [x, y] mm"#),
        (message().id("m"), "a message has its text, its code, or both"),
        (side().id("s").ref_("x").kind("mechanical"), "kind must be one of circuit, 3d, firmware, definition"),
        (side().id("s").ref_("x").channel(3), "channel is the id of a channel of the device definition"),
        (side().id("s").ref_("x").rotation("90"), "rotation is the angle the interface's frame is turned on this side, in degrees counter-clockwise"),
        (side().id("s").ref_("x").face("bottom"), r#"side().id("s"): face must be one of front, back (got "bottom")"#),
        (param().id("p").value(true), "value is a number with its unit in unit (115200), or a text a channel states"),
        (param().id("p").value("8N1").unit("Bd"), "a parameter stated as a text has no unit"),
        (frame().unit("in"), "positions are in mm"),
    ];
    for (el, message) in cases {
        let said = bad(interface([el]));
        assert!(said.contains(message), "{said}");
    }
}
