// A design's logistics in code (docs/formats.md § logistics): a schematic part's suppliers and facets as KiCad
// property forms, read back by part_fields; a circuit's logistics check() options; a board's fab(); a 3D document's
// item(), assemble() and the logistics fields of body().engineering(…). The TypeScript SDK's logistics.test.ts holds
// the same cases.

use commandagi::design::{declare, El, Json};

fn graph_of(root: El) -> Json {
    declare(root).unwrap().graph().unwrap()
}

fn bad(root: El) -> String {
    declare(root).unwrap_err()
}

fn node<'a>(g: &'a Json, id: &str) -> &'a Json {
    g.get("nodes").and_then(|n| n.get(id)).unwrap_or_else(|| panic!("no node {id}"))
}

/// Each `property` form's name and value.
fn props(inputs: &Json) -> Vec<(String, String)> {
    let Some(Json::Arr(forms)) = inputs.get("foreign") else { panic!("no foreign forms") };
    forms
        .iter()
        .map(|f| {
            let Some(Json::Arr(items)) = f.get("items") else { panic!("not a list") };
            let v = |i: usize| items[i].get("v").and_then(Json::as_str).unwrap().to_string();
            (v(1), v(2))
        })
        .collect()
}

const LCSC: &str = r#"{"name": "LCSC", "sku": "C21264512", "currency": "USD", "priceBreaks": [{"quantity": 1, "unitPrice": 9.4}, {"quantity": 10, "unitPrice": 9.15}], "leadTimeDays": 3, "stock": 1, "source": "lcsc.com", "date": "2026-10-15"}"#;
const ALTERNATES: &str = r#"[{"manufacturer": "SIMCom", "mpn": "A7670E-LASE"}]"#;
const DATASHEET: &str = "https://www.simcom.com/product/A7670X.html";
const LIFECYCLE: &str = r#"{"status": "active", "onHand": 2, "reorderPoint": 5}"#;
const COMPLIANCE: &str = r#"{"rohs": "compliant", "reach": "compliant", "eccn": "5A992.c", "certifications": ["FCC ID 2AJYU-8BAE003", "CE"]}"#;
const SHIPPING: &str = r#"{"hsCode": "8517.62", "origin": "CN", "hazmat": "none"}"#;
const FORMS: &[(&str, &str)] = &[
    ("MPN", "A7670G-LLSE"),
    ("LCSC Part", "C21264512"),
    ("LCSC Terms", r#"{"currency":"USD","priceBreaks":[[1,9.4],[10,9.15]],"leadTimeDays":3,"stock":1,"source":"lcsc.com","date":"2026-10-15"}"#),
    ("DigiKey Part", "A7670G-ND"),
    ("Alternates", r#"[{"manufacturer":"SIMCom","mpn":"A7670E-LASE"}]"#),
    ("Datasheet", "https://www.simcom.com/product/A7670X.html"),
    ("Lifecycle", "active"),
    ("On Hand", "2"),
    ("Reorder Point", "5"),
    ("RoHS", "compliant"),
    ("REACH", "compliant"),
    ("ECCN", "5A992.c"),
    ("Certifications", "FCC ID 2AJYU-8BAE003, CE"),
    ("HS Code", "8517.62"),
    ("Country of Origin", "CN"),
    ("Hazmat", "none"),
];

fn suppliers() -> String {
    format!(r#"[{LCSC}, {{"name": "DigiKey", "sku": "A7670G-ND"}}]"#)
}

#[test]
fn a_schematic_part_states_its_suppliers_and_facets_as_kicad_properties_read_back_by_part_fields() {
    use commandagi::design::schematic::*;
    let u4 = part("U4")
        .pins(["1", "2"])
        .mpn("A7670G-LLSE")
        .suppliers(json(&suppliers()))
        .alternates(json(ALTERNATES))
        .datasheet(DATASHEET)
        .lifecycle(json(LIFECYCLE))
        .compliance(json(COMPLIANCE))
        .shipping(json(SHIPPING));
    let g = graph_of(group("L", [u4]));
    let inputs = node(&g, "U4").get("inputs").unwrap();
    let want: Vec<(String, String)> = FORMS.iter().map(|(n, v)| (n.to_string(), v.to_string())).collect();
    assert_eq!(props(inputs), want);
    let parse = |t: &str| Json::parse(t).unwrap();
    let back = Json::obj()
        .with("mpn", "A7670G-LLSE")
        .with("alternates", parse(ALTERNATES))
        .with("datasheet", DATASHEET)
        .with("lifecycle", parse(LIFECYCLE))
        .with("compliance", parse(COMPLIANCE))
        .with("shipping", parse(SHIPPING))
        .with("suppliers", parse(&suppliers()));
    assert_eq!(part_fields(inputs).text(), back.text());
}

#[test]
fn a_wrong_logistics_field_is_refused_by_name_and_attrs_cannot_say_what_an_argument_says() {
    use commandagi::design::schematic::*;
    let sheet = |p: El| bad(group("L", [p]));
    let u4 = || part("U4").pins(["1"]);
    let refused = [
        (u4().suppliers(json(r#"[{"name": "LCSC", "price": 3}]"#)), "suppliers[0]: price is not a term of an offer"),
        (u4().suppliers(json(r#"[{"name": "LCSC", "priceBreaks": [{"quantity": 0, "unitPrice": 1}]}]"#)), "priceBreaks is a list of { quantity, unitPrice }"),
        (u4().suppliers(json(r#"[{"name": "LCSC", "currency": "usd"}]"#)), "currency is an ISO 4217 code"),
        (u4().lifecycle(json(r#"{"onHand": 3}"#)), "lifecycle needs its status"),
        (u4().lifecycle(json(r#"{"status": "dead"}"#)), "lifecycle.status is one of active, nrnd, eol, obsolete"),
        (u4().compliance(json(r#"{"rohs": "yes"}"#)), "compliance.rohs is one of compliant, exempt, non-compliant"),
        (u4().shipping(json(r#"{"origin": "China"}"#)), "shipping.origin is an ISO 3166 alpha-2 code"),
        (u4().attrs(json(r#"{"LCSC Part": "C1"}"#)), "attrs names LCSC Part: say it with suppliers"),
        (u4().attrs(json(r#"{"RoHS": "compliant"}"#)), "attrs names RoHS: say it with compliance"),
        (u4().attrs(json(r#"{"Datasheet": "x"}"#)), "attrs names Datasheet: say it with datasheet"),
    ];
    for (el, why) in refused {
        let got = sheet(el);
        assert!(got.contains(why), "{got:?} does not say {why:?}");
    }
}

#[test]
fn a_circuits_logistics_check_keeps_its_options() {
    use commandagi::design::schematic::*;
    let g = graph_of(group("L", [check().id("cost").kind("cost").options(json(r#"{"quantities": [10, 100]}"#))]));
    let Some(Json::Arr(checks)) = g.get("meta").and_then(|m| m.get("checks")) else { panic!("no checks") };
    let mut c = checks[0].clone();
    c.remove("source");
    assert_eq!(c.text(), r#"{"id":"cost","kind":"cost","options":{"quantities":[10,100]}}"#);
    assert!(bad(group("L", [check().id("cost").kind("cost").options(json("[10]"))])).contains("options is an object"));
}

#[test]
fn a_boards_fab_is_one_board_fact_checked_by_name() {
    use commandagi::design::pcb::*;
    let board_of = |children: Vec<El>| board(children).schematic("L.sch.rs").width(20).height(20);
    let fab_json = r#"{"suppliers": [{"name": "JLCPCB", "currency": "USD", "priceBreaks": [{"quantity": 5, "unitPrice": 2}], "leadTimeDays": 7, "source": "jlcpcb.com quote", "date": "2026-10-15"}], "finish": "ENIG", "mask": "green", "silk": "white", "minTrack": 0.09, "minSpace": 0.09, "minDrill": 0.15, "minAnnularRing": 0.13, "maxLayers": 6, "thicknesses": [1.6], "assembly": {"house": "JLCPCB", "sides": ["top"]}, "inspection": {"electrical": "flying-probe", "acceptance": "boots"}, "compliance": {"rohs": "compliant", "reach": "compliant", "eccn": "EAR99"}, "shipping": {"hsCode": "8534.00", "origin": "CN", "hazmat": "none"}}"#;
    let f = Json::parse(fab_json).unwrap();
    let el = fab()
        .suppliers(f.get("suppliers").cloned().unwrap())
        .finish("ENIG")
        .mask("green")
        .silk("white")
        .min_track(0.09)
        .min_space(0.09)
        .min_drill(0.15)
        .min_annular_ring(0.13)
        .max_layers(6)
        .thicknesses([1.6])
        .assembly(json(r#"{"house": "JLCPCB", "sides": ["top"]}"#))
        .inspection(json(r#"{"electrical": "flying-probe", "acceptance": "boots"}"#))
        .compliance(json(r#"{"rohs": "compliant", "reach": "compliant", "eccn": "EAR99"}"#))
        .shipping(json(r#"{"hsCode": "8534.00", "origin": "CN", "hazmat": "none"}"#));
    let g = graph_of(board_of(vec![el]));
    let Some(Json::Obj(nodes)) = g.get("nodes") else { panic!("no nodes") };
    let facts: Vec<&Json> = nodes
        .iter()
        .map(|(_, n)| n)
        .filter(|n| n.get("type").and_then(Json::as_str) == Some(BOARD_FACT) && n.get("inputs").and_then(|i| i.get("fact")).and_then(Json::as_str) == Some("fab"))
        .collect();
    assert_eq!(facts.len(), 1);
    let mut want = Json::obj().with("fact", "fab");
    if let Json::Obj(entries) = f {
        for (k, v) in entries {
            want.set(&k, v);
        }
    }
    assert_eq!(facts[0].get("inputs").unwrap().text(), want.text());
    assert!(bad(board_of(vec![fab(), fab()])).contains("a board has one fab()"));
    assert!(bad(board_of(vec![fab().layers(4)])).contains("fab(): layers is not read on a board"));
    assert!(bad(board_of(vec![fab().assembly(json(r#"{"house": "JLCPCB", "sides": ["left"]}"#))])).contains(r#"assembly.sides is a list of "top", "bottom""#));
    assert!(bad(board_of(vec![fab().inspection(json(r#"{"electrical": "x-ray"}"#))])).contains("inspection.electrical is one of flying-probe, fixture, none"));
}

#[test]
fn a_3d_documents_item_and_assemble_are_lists_of_the_document_and_body_engineering_takes_the_logistics_fields() {
    use commandagi::design::threed::*;
    let engineering = r#"{"process": "fdm-printing", "make": {"device": "devices/ender/definition.tsx", "cycleMinutes": 30, "ratePerHour": 2, "currency": "USD"}, "inspection": {"procedure": "look", "acceptance": "flat"}}"#;
    let tape = r#"{"process": "purchased", "suppliers": [{"name": "DigiKey", "sku": "3M4026-ND"}]}"#;
    let g = graph_of(
        part([
            box_().id("b").center([0, 0, 5]).size([10, 10, 10]).operation("new"),
            body().id("b").engineering(json(engineering)),
            item().id("tape").name("Foam tape").qty(2).engineering(json(tape)),
            assemble().id("s1").order(1).parts(["b", "tape"]).by("in-house").minutes(2),
        ])
        .name("P"),
    );
    let value = |id: &str| node(&g, id).get("inputs").and_then(|i| i.get("value")).unwrap().text();
    assert_eq!(value("items"), format!(r#"[{{"id":"tape","name":"Foam tape","qty":2,"engineering":{}}}]"#, Json::parse(tape).unwrap().text()));
    assert_eq!(value("assemblySteps"), r#"[{"id":"s1","order":1,"parts":["b","tape"],"by":"in-house","minutes":2}]"#);
    let refused = |el: El| bad(part([el]).name("P"));
    assert!(refused(body().id("b").engineering(json(r#"{"vendor": {"name": "X"}}"#))).contains("vendor"));
    assert!(refused(item().id("t").name("T").qty(0)).contains("qty"));
    assert!(refused(body().id("b").engineering(json(r#"{"lifecycle": {"status": "dead"}}"#))).contains("status"));
    assert!(refused(slot().name("items").value(json("[]"))).contains("items is not a slot"));
}
