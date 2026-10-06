// Machine jobs against the documents the TypeScript SDK's fab.test.ts pins for the same elements: records are
// elements, operations in their order, each site by its path; what the vocabulary has no words for is refused by name.

use commandagi::design::fab::*;
use commandagi::design::{declare, Declared};

fn document(root: El) -> (String, Json, Json) {
    match declare(root).unwrap() {
        Declared::Document { format, document, sources } => (format, document, sources),
        Declared::Graph(_) => panic!("a machine job is a document"),
    }
}

fn bad(root: El) -> String {
    declare(root).unwrap_err()
}

#[test]
fn a_machining_setup_is_its_setup_with_each_site_by_its_path() {
    let line = line!() + 1;
    let root = cam([
        stock().material_id("plywood").thickness_mm(6).x_mm(90).y_mm(70),
        machine().post("grbl").max_spindle_rpm(10000),
        fixture().name("clamp").x_mm(-14).y_mm(25).w_mm(20).d_mm(20).z_mm(4),
        operation().id("op-2").op("mill_contour").profile("contour_wood").tabs(json(r#"{"count": 4, "lengthMm": 5, "heightMm": 1.5}"#)),
        operation().id("op-1").op("mill_pocket").profile("pocket_wood").enabled(false).params(json(r#"{"toolDiameterMm": 3.175}"#)),
        runs_on().unit("cloud://global/worlds/fab-cell/world.tsx#cnc-1").channel("gcode").name("cnc"),
    ]);
    let (format, doc, sources) = document(root);
    assert_eq!(format, "cam");
    let golden = Json::parse(
        r#"{"operations":[
            {"id":"op-2","op":"mill_contour","profile":"contour_wood","tabs":{"count":4,"lengthMm":5,"heightMm":1.5}},
            {"id":"op-1","op":"mill_pocket","profile":"pocket_wood","enabled":false,"params":{"toolDiameterMm":3.175}}],
          "stock":{"materialId":"plywood","thicknessMm":6,"xMm":90,"yMm":70},
          "machine":{"post":"grbl","maxSpindleRpm":10000},
          "fixtures":[{"name":"clamp","xMm":-14,"yMm":25,"wMm":20,"dMm":20,"zMm":4}],
          "runsOn":{"unit":"cloud://global/worlds/fab-cell/world.tsx#cnc-1","channel":"gcode","name":"cnc"}}"#,
    )
    .unwrap();
    assert_eq!(doc.text(), golden.text());
    let site = |l: u32, c: u32| Json::obj().with("site", vec![Json::from(l), Json::from(c)]);
    let expected = Json::obj()
        .with("", site(line, 16))
        .with("stock", site(line + 1, 9))
        .with("machine", site(line + 2, 9))
        .with("fixture@0", site(line + 3, 9))
        .with("operation#op-2", site(line + 4, 9))
        .with("operation#op-1", site(line + 5, 9))
        .with("runsOn", site(line + 6, 9));
    assert_eq!(sources.text(), expected.text());
}

#[test]
fn a_machining_setup_refuses_by_name_what_it_has_no_words_for() {
    assert_eq!(bad(cam([operation().id("a").op("face")])), r#"operation().id("a") needs profile"#);
    assert_eq!(
        bad(cam([operation().id("a").op("face").profile("p"), operation().id("a").op("face").profile("p")])),
        r#"two operation() in cam() have id "a""#
    );
    assert_eq!(bad(cam([stock(), stock()])), "cam() has one stock()");
    assert!(bad(cam([spool()])).starts_with("spool() is not an element of a machining setup"));
    assert!(bad(cam([stock().with_attr("colour", "red")])).starts_with("stock(): colour is not read"));
    assert!(bad(cam([stock().x_mm(f64::NAN)])).contains("x_mm is NaN, not a finite number"));
}

#[test]
fn a_slicing_setup_has_its_profile_on_the_root_and_each_record_one_element() {
    let root = slice([
        source().file_id("carrier.stl").name("carrier.stl"),
        spool().material_id("pla").diameter_mm(1.75),
        machine().unit("cloud://global/worlds/fab-cell/world.tsx#printer-1").channel("gcode").name("ender"),
    ])
    .profile("fdm_pla_0.20_draft")
    .params(json(r#"{"layerHeightMm": 0.2}"#));
    let (format, doc, _) = document(root);
    assert_eq!(format, "slice");
    let golden = Json::parse(
        r#"{"profile":"fdm_pla_0.20_draft","params":{"layerHeightMm":0.2},"source":{"fileId":"carrier.stl","name":"carrier.stl"},
            "spool":{"materialId":"pla","diameterMm":1.75},
            "machine":{"unit":"cloud://global/worlds/fab-cell/world.tsx#printer-1","channel":"gcode","name":"ender"}}"#,
    )
    .unwrap();
    assert_eq!(doc.text(), golden.text());
    assert_eq!(bad(slice([])), "slice() needs profile");
    assert_eq!(bad(slice([machine().unit("x")]).profile("p")), "machine() needs channel");
}
