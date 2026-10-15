// Tolerances as the crate reads them: classes against numbers typed here from ISO 286-2 and ISO 492, a fit's kind, and
// a parameter's tolerance through the 3D reader. The TypeScript SDK's src/design/tolerances.test.ts and the Python
// SDK's tests/test_tolerances.py hold the same cases.

use commandagi::design::threed::{parameter, part, ThreedAttrs};
use commandagi::design::tolerances::*;
use commandagi::design::{declare, json, El, Json};

fn tol(text: &str) -> Json {
    Json::parse(text).unwrap()
}

fn limits(nominal: f64, text: &str) -> Limits {
    tolerance_limits(nominal, &tol(text)).unwrap()
}

#[test]
fn a_class_is_read_from_the_tables_at_its_size() {
    let h7 = limits(25.0, r#"{"class": "H7"}"#);
    assert_eq!((h7.upper, h7.lower), (0.021, 0.0));
    let g6 = limits(25.0, r#"{"class": "g6"}"#);
    assert_eq!((g6.upper, g6.lower), (-0.007, -0.02));
    assert_eq!(limits(10.0, r#"{"class": "H7"}"#).max, 10.015);
    assert_eq!(limits(15.0, r#"{"class": "ISO 492 bore"}"#).min, 14.992);
    assert!(h7.source.is_some());
    let unknown = tolerance_limits(25.0, &tol(r#"{"class": "H77"}"#)).unwrap_err();
    assert!(unknown.contains("no tolerance class \"H77\""), "{unknown}");
    let big = tolerance_limits(900.0, &tol(r#"{"class": "H7"}"#)).unwrap_err();
    assert_eq!(big, "H7 is tabled over 0 up to 120 mm, not at 900 mm");
    let own = limits(12.0, r#"{"upper": 0.05, "lower": -0.05}"#);
    assert_eq!((own.min, own.max), (11.95, 12.05));
}

#[test]
fn a_fit_is_clearance_transition_or_interference() {
    let fit = |h: &str, s: &str, d: f64| fit_of(&limits(d, &format!(r#"{{"class": "{h}"}}"#)), &limits(d, &format!(r#"{{"class": "{s}"}}"#)));
    assert_eq!(fit("H7", "g6", 25.0), Fit { kind: "clearance", min_clearance: 0.007, max_clearance: 0.041 });
    assert_eq!(fit("H7", "p6", 25.0).kind, "interference");
    assert_eq!(fit("H7", "k6", 25.0).kind, "transition");
    assert_eq!(fit("ISO 492 bore", "k5", 15.0).kind, "interference");
}

fn doc(children: Vec<El>) -> Result<Json, String> {
    declare(part(children).name("P")).map(|d| d.graph().unwrap())
}

#[test]
fn a_parameters_tolerance_is_read_into_the_graph() {
    let g = doc(vec![parameter().name("shaftD").value(15).tolerance(json(r#"{"class": "k5"}"#))]).unwrap();
    assert_eq!(g.get("nodes").and_then(|n| n.get("shaftD")).and_then(|n| n.get("inputs")).map(Json::text), Some(r#"{"value":15,"tolerance":{"class":"k5"}}"#.to_string()));
    for (tol, said) in [
        (r#"{"class": "k5", "upper": 0.1, "lower": 0}"#, "tolerance: give the class (k5) or the deviations, not both"),
        (r#"{"upper": 0.1}"#, "tolerance: give both deviations, upper and lower (0 for none)"),
        (r#"{"upper": -0.1, "lower": 0.1}"#, "tolerance: the upper deviation (-0.1) is below the lower (0.1)"),
    ] {
        let err = doc(vec![parameter().name("w").value(6).tolerance(json(tol))]).unwrap_err();
        assert!(err.ends_with(said), "{err}");
    }
}
