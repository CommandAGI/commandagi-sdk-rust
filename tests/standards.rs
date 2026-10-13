// The engineering tables as the crate carries them, held to numbers typed here from the standards' own tables (the
// shipped CSVs name the page each row was read from; `cloud://global/standards/<table>.csv`). The TypeScript SDK's
// src/design/standards.test.ts and the Python SDK's tests/test_standards.py hold the same values: one set of tables
// makes every SDK's data.

use commandagi::design::standards::*;
use commandagi::design::threed::{hole, part, ThreedAttrs};
use commandagi::design::{declare, json, Json};

fn num(row: &Json, key: &str) -> f64 {
    row.get(key).and_then(Json::as_f64).unwrap_or_else(|| panic!("no number {key} in {}", row.text()))
}

fn row(table: &str, key: &str, fields: &[(&str, f64)]) {
    let r = standard_row(table, key).unwrap();
    for (k, v) in fields {
        assert_eq!(num(&r, k), *v, "{table} {key} {k}");
    }
}

fn diameter(name: &str) -> f64 {
    num(&hole_size(name).unwrap(), "diameter")
}

#[test]
fn a_sample_of_each_dimension_table_is_the_standards_number() {
    // ISO 4762 hexagon socket head cap screws: head diameter, head height, socket.
    row("iso-4762", "M3", &[("dkMax", 5.5), ("kMax", 3.0), ("s", 2.5)]);
    row("iso-4762", "M8", &[("dkMax", 13.0), ("kMax", 8.0), ("s", 6.0)]);
    // ISO 7380-1 button head, ISO 10642 countersunk head, ISO 4017 hexagon head, ISO 4026 set screw.
    row("iso-7380-1", "M5", &[("dkMax", 9.5), ("kMax", 2.75), ("s", 3.0)]);
    row("iso-10642", "M6", &[("dkTheoreticalMax", 13.44), ("kMax", 3.72), ("s", 4.0), ("headAngleDeg", 90.0)]);
    row("iso-4014-4017", "M10", &[("sMax", 16.0), ("kNom", 6.4)]);
    row("iso-4026", "M6", &[("s", 3.0)]);
    // ISO 4032 and ISO 4035 nuts; ISO 7089 and ISO 7093-1 washers.
    row("iso-4032", "M8", &[("sMax", 13.0), ("mMax", 6.8)]);
    row("iso-4035", "M8", &[("mMax", 4.0)]);
    row("iso-7089", "M6", &[("d1Nom", 6.4), ("d2Nom", 12.0), ("hNom", 1.6)]);
    row("iso-7093-1", "M6", &[("d1Nom", 6.4), ("d2Nom", 18.0), ("hNom", 1.6)]);
    // ISO 261 coarse pitch; ASME B18.3 inch socket head cap screw.
    row("metric-threads", "M5", &[("d", 5.0), ("pitchCoarse", 0.8)]);
    row("asme-b18.3-shcs", "#4-40", &[("headDiameterMaxIn", 0.183), ("threadsPerInch", 40.0)]);
    // ISO 15 bearing boundary dimensions; a HIWIN MGN12H block; a MISUMI 2020 profile; ISO 2338 pins.
    row("bearings", "608", &[("d", 8.0), ("D", 22.0), ("B", 7.0)]);
    row("bearings", "625", &[("d", 5.0), ("D", 16.0), ("B", 5.0)]);
    row("mgn-linear-guide", "MGN12H", &[("H", 13.0), ("W", 27.0), ("L", 45.4), ("WR", 12.0), ("HR", 8.0), ("P", 25.0)]);
    row("t-slot-profiles", "HFS5-2020", &[("width", 20.0), ("height", 20.0), ("slotWidth", 6.0)]);
    row("iso-2338", "6", &[("chamferC", 1.2)]);
    row("heat-set-inserts", "M3x5.7", &[("length", 5.7), ("recommendedHoleDiameter", 4.0)]);
    assert!(standard_row("iso-4762", "M7").unwrap_err().starts_with("table \"iso-4762\" has no row \"M7\" (it has M2, "));
}

#[test]
fn a_hole_by_name_is_the_clearance_hole_of_iso_273_or_the_tap_drill_of_iso_2306() {
    let fits = ["close fit", "normal fit", "loose fit"];
    assert_eq!(fits.map(|f| diameter(&format!("M5 {f}"))), [5.3, 5.5, 5.8]);
    assert_eq!(fits.map(|f| diameter(&format!("M8 {f}"))), [8.4, 9.0, 10.0]);
    assert_eq!(diameter("M6 tap"), 5.0);
    assert_eq!(diameter("m10  Tap"), 8.5);
    assert!(hole_size("M4 close fit").unwrap().get("source").and_then(Json::as_str).unwrap().contains("ISO 273"));
    // An inch thread: ASME B18.2.8 normal fit for 1/4 is 0.281 in.
    assert_eq!(diameter("1/4-20 normal fit"), 7.1374);
    assert_eq!(hole_size("M4 snug").unwrap_err(), r#""M4 snug": a hole size is a thread and one of close fit, normal fit, loose fit, tap ("M4 close fit")"#);
    assert!(hole_size("M7.5 tap").unwrap_err().starts_with(r#""M7.5 tap": no hole table has the thread M7.5 (it has M1.6, "#));
    // The head of a counterbore (DIN 974-1, for an ISO 4762 screw) and of a countersink (DIN 74 form F, 90°).
    let bore = sized_hole(&Json::parse(r#"{"size": "M4 close fit", "style": "counterbore"}"#).unwrap()).unwrap();
    assert_eq!((num(&bore, "diameter"), num(&bore, "headDiameter"), num(&bore, "headDepth")), (4.3, 8.0, 4.4));
    let sink = sized_hole(&Json::parse(r#"{"size": "M4 normal fit", "style": "countersink"}"#).unwrap()).unwrap();
    assert_eq!((num(&sink, "diameter"), num(&sink, "headDiameter"), num(&sink, "headDepth")), (4.5, 9.2, 2.35));
    let tap = sized_hole(&Json::parse(r#"{"size": "M4 tap", "style": "simple"}"#).unwrap()).unwrap();
    assert_eq!(tap.text(), r#"{"size":"M4 tap","style":"simple","diameter":3.3}"#);
    // The size gives the head too: a head that differs is refused (clear the size to give your own).
    assert_eq!(
        sized_hole(&Json::parse(r#"{"size": "M4 close fit", "style": "counterbore", "headDiameter": 9}"#).unwrap()).unwrap_err(),
        r#"headDiameter: "M4 close fit" is 8 mm (got 9); leave headDiameter out, or clear size"#
    );
}

#[test]
fn a_material_is_a_row_with_its_density_and_its_source_and_a_mass_is_volume_times_density() {
    assert_eq!(num(&material("6061-T6", &[]).unwrap(), "densityGCm3"), 2.7);
    assert_eq!(num(&material("304", &[]).unwrap(), "densityGCm3"), 8.0);
    assert_eq!(num(&material("petg", &[]).unwrap(), "densityGCm3"), 1.27);
    assert_eq!(mass_grams(1000.0, "304", &[]).unwrap(), 8.0);
    // A table of your own is read before the shipped one.
    let mine = [Json::parse(r#"{"id": "304", "densityGCm3": 7.93, "source": "mill certificate 4711"}"#).unwrap()];
    assert_eq!(mass_grams(1000.0, "304", &mine).unwrap(), 7.93);
    assert!(material("unobtainium", &[]).unwrap_err().starts_with(r#"no materials table has a row "unobtainium" (the shipped table has 6061-T6, "#));
    for m in materials() {
        assert!(m.get("source").is_some() && m.get("sourceUrl").is_some() && num(m, "densityGCm3") > 0.0, "{}", m.text());
    }
    let Some(Json::Obj(tables)) = standards().get("tables") else { panic!("no tables") };
    for (name, _) in tables {
        let Some(Json::Arr(sources)) = standards().get("sources").and_then(|s| s.get(name)) else { panic!("no sources of {name}") };
        assert!(!sources.is_empty() && sources.iter().all(|s| s.get("source").is_some() && s.get("sourceUrl").is_some() && s.get("observedAt").is_some()), "{name}");
    }
}

#[test]
fn a_hole_that_names_a_size_has_that_sizes_diameter_and_head() {
    let face = r#"{"kind": "face", "signature": {"point": [0, 0, 6], "normal": [0, 0, 1]}}"#;
    let read = |h: commandagi::design::El| declare(part([h.id("h1").face(json(face))]).name("P")).map(|d| d.graph().unwrap().get("nodes").and_then(|n| n.get("h1")).and_then(|n| n.get("inputs")).cloned().unwrap());
    let bore = read(hole().size("M4 close fit").style("counterbore")).unwrap();
    assert_eq!((num(&bore, "diameter"), num(&bore, "headDiameter"), num(&bore, "headDepth"), num(&bore, "depth")), (4.3, 8.0, 4.4, 10.0));
    assert_eq!(num(&read(hole().size("M3 tap").through(false).depth(8)).unwrap(), "diameter"), 2.5);
    assert_eq!(num(&read(hole().size("M4 close fit").diameter(4.3)).unwrap(), "diameter"), 4.3);
    assert_eq!(
        read(hole().size("M4 close fit").diameter(6)).unwrap_err(),
        r#"hole().id("h1"): diameter: "M4 close fit" is 4.3 mm (got 6); leave diameter out, or clear size"#
    );
    assert!(read(hole().size("M4 snug")).unwrap_err().starts_with(r#"hole().id("h1"): "M4 snug": a hole size is a thread and one of"#));
}
