// A study in Rust against the document the TypeScript SDK's study.test.ts pins for the same elements: fixtures, load
// cases with typed loads, mesh, criteria, each site by its path; a tag of another kind and a wrong field are refused
// by name.

use commandagi::design::study::*;
use commandagi::design::{declare, Declared};

fn document(root: El) -> (String, Json, Json) {
    match declare(root).unwrap() {
        Declared::Document { format, document, sources } => (format, document, sources),
        Declared::Graph(_) => panic!("a study is a document"),
    }
}

fn bad(root: El) -> String {
    declare(root).unwrap_err()
}

fn face(select: &str) -> Value {
    json(&format!(r#"{{"kind": "face", "select": {select}}}"#))
}

fn of(children: Vec<El>) -> El {
    study(children).kind("structural").part("a.3d.tsx")
}

#[test]
fn a_structural_study_is_its_question_with_each_site_by_its_path() {
    let line = line!() + 1;
    let root = study([
        material().grade("PETG").poisson(0.38),
        fixture().id("mount").type_("fixed").faces(face(r#"{"normal": [0, 0, -1]}"#)),
        fixture().id("hinge").type_("fixed").faces(face(r#"{"normal": [-1, 0, 0]}"#)).region(json(r#"{"inBox": {"min": [0, -5, 0], "max": [0, 5, 0]}}"#)),
        load_case([
            force().id("grip").faces(face(r#"{"farthest": [1, 0, 0]}"#)).total([-40, 0, 0]),
            pressure().id("air").faces(face(r#"{"normal": [0, 1, 0]}"#)).value(0.1),
        ])
        .id("clamp"),
        mesh().order(2).size(2).refine([1.0, 0.7, 0.5]),
        solver().kind("calculix").analysis("static").timeout(1800),
        criterion().id("safety").quantity("vonMises.max").limit("yield / 2"),
        revise().parameter("thickness").min(3).max(8),
    ])
    .kind("structural")
    .part("Jaw.3d.tsx")
    .units("mm-N-MPa-t");
    let (format, doc, sources) = document(root);
    assert_eq!(format, "study");
    // The TypeScript SDK's study.test.ts declares this document for the same elements, key for key.
    let golden = Json::parse(
        r#"{"kind":"structural","part":"Jaw.3d.tsx","units":"mm-N-MPa-t",
          "material":{"grade":"PETG","poisson":0.38},
          "fixtures":[{"id":"mount","type":"fixed","faces":{"kind":"face","select":{"normal":[0,0,-1]}}},
            {"id":"hinge","type":"fixed","faces":{"kind":"face","select":{"normal":[-1,0,0]}},"region":{"inBox":{"min":[0,-5,0],"max":[0,5,0]}}}],
          "loadCases":[{"id":"clamp","loads":[
            {"type":"force","id":"grip","faces":{"kind":"face","select":{"farthest":[1,0,0]}},"total":[-40,0,0]},
            {"type":"pressure","id":"air","faces":{"kind":"face","select":{"normal":[0,1,0]}},"value":0.1}]}],
          "mesh":{"order":2,"size":2,"refine":[1,0.7,0.5]},
          "solver":{"kind":"calculix","analysis":"static","timeout":1800},
          "criteria":[{"id":"safety","quantity":"vonMises.max","limit":"yield / 2"}],
          "revise":[{"parameter":"thickness","min":3,"max":8}]}"#,
    )
    .unwrap();
    assert_eq!(doc.text(), golden.text());
    let site = |l: u32, c: u32| Json::obj().with("site", vec![Json::from(l), Json::from(c)]);
    let expected = Json::obj()
        .with("", site(line, 16))
        .with("material", site(line + 1, 9))
        .with("fixture#mount", site(line + 2, 9))
        .with("fixture#hinge", site(line + 3, 9))
        .with("loadCase#clamp", site(line + 4, 9))
        .with("loadCase#clamp/force#grip", site(line + 5, 13))
        .with("loadCase#clamp/pressure#air", site(line + 6, 13))
        .with("mesh", site(line + 9, 9))
        .with("solver", site(line + 10, 9))
        .with("criterion#safety", site(line + 11, 9))
        .with("revise#thickness", site(line + 12, 9));
    assert_eq!(sources.text(), expected.text());
}

#[test]
fn a_flow_study_is_the_fluid_its_inlets_and_outlets_on_one_body() {
    let (format, doc, _) = document(
        study([
            fluid().nu(1e-6),
            inlet().id("inlet").faces(face(r#"{"normal": [-1, 0, 0]}"#)).velocity(0.5),
            inlet().id("side").faces(face(r#"{"normal": [0, 1, 0]}"#)).velocity(0.2),
            outlet().id("outlet").faces(face(r#"{"normal": [1, 0, 0]}"#)),
            flow_mesh().cells([20, 3, 3]).levels([1, 2, 4]),
            criterion().id("drop").quantity("pressureDrop").limit(2),
        ])
        .kind("flow")
        .part("Channel.3d.tsx")
        .units("m-kg-s"),
    );
    assert_eq!(format, "study");
    let golden = Json::parse(
        r#"{"kind": "flow", "part": "Channel.3d.tsx", "units": "m-kg-s", "fluid": {"nu": 1e-6},
            "inlets": [{"id": "inlet", "faces": {"kind": "face", "select": {"normal": [-1, 0, 0]}}, "velocity": 0.5},
                {"id": "side", "faces": {"kind": "face", "select": {"normal": [0, 1, 0]}}, "velocity": 0.2}],
            "outlets": [{"id": "outlet", "faces": {"kind": "face", "select": {"normal": [1, 0, 0]}}}],
            "flowMesh": {"cells": [20, 3, 3], "levels": [1, 2, 4]},
            "criteria": [{"id": "drop", "quantity": "pressureDrop", "limit": 2}]}"#,
    )
    .unwrap();
    assert_eq!(doc.text(), golden.text());
    assert_eq!(bad(study([mesh().size(2)]).kind("flow").part("a.3d.tsx")), "mesh() belongs to a structural study, not to a flow one");
}

#[test]
fn a_step_response_study_names_its_steps_gains_and_window() {
    let (format, doc, _) = document(
        study([
            step().id("lift").joint("shoulder_lift").from(0.54).to(0.84),
            gains().joint("shoulder_lift").kp(12).kv(0.6),
            response().duration(1).dt(0.001).band(0.02),
            criterion().id("settle").quantity("settlingTime").limit(0.3).step("lift"),
        ])
        .kind("step-response")
        .part("../worlds/arm-cell/world.tsx#arm")
        .units(STEP_RESPONSE_UNITS),
    );
    assert_eq!(format, "study");
    let golden = Json::parse(
        r#"{"kind": "step-response", "part": "../worlds/arm-cell/world.tsx#arm", "units": "rad-s",
            "steps": [{"id": "lift", "joint": "shoulder_lift", "from": 0.54, "to": 0.84}],
            "gains": [{"joint": "shoulder_lift", "kp": 12, "kv": 0.6}],
            "response": {"duration": 1, "dt": 0.001, "band": 0.02},
            "criteria": [{"id": "settle", "quantity": "settlingTime", "limit": 0.3, "step": "lift"}]}"#,
    )
    .unwrap();
    assert_eq!(doc.text(), golden.text());
    let sr = |children: Vec<El>| study(children).kind("step-response").part("definition.tsx");
    assert_eq!(bad(sr(vec![mesh().size(2)])), "mesh() belongs to a structural study, not to a step-response one");
    assert_eq!(bad(sr(vec![step().id("a").joint("j").from(0)])), r#"step().id("a") needs to"#);
}

#[test]
fn a_study_refuses_by_name_what_it_has_no_words_for() {
    assert_eq!(bad(study([]).kind("structural")), "study() needs part");
    assert_eq!(bad(study([]).kind("thermal").part("a.3d.tsx")), r#"study() kind is one of structural, flow, step-response, not "thermal""#);
    assert_eq!(bad(of(vec![force().id("f").faces(face("{}")).total([0, 0, 1])])), "force() stands in load_case(), not in study()");
    assert_eq!(bad(of(vec![fixture().id("a").type_("fixed")])), r#"fixture().id("a") needs faces"#);
    assert_eq!(bad(of(vec![mesh().order(2).size(2).poisson(0.3)])), "mesh(): poisson is not read (order, size, refine)");
    assert_eq!(bad(of(vec![material().grade("PETG").youngs_modulus(3)])), "material(): youngs_modulus is not read (grade, body, poisson)");
    assert_eq!(bad(of(vec![material().grade("PETG"), material().grade("PLA")])), "study() has one material()");
    assert_eq!(
        bad(of(vec![criterion().id("c").quantity("vonMises.max").limit(1), criterion().id("c").quantity("displacement.max").limit(1)])),
        r#"two criterion() in study() have id "c""#
    );
    assert_eq!(bad(of(vec![mesh().size(f64::NAN)])), "mesh() size is NaN, not a finite number");
}

trait Unknown {
    fn youngs_modulus(self, v: impl Into<Value>) -> Self;
}
impl Unknown for El {
    /// An attribute no study reads (a material's E is the materials table's).
    fn youngs_modulus(self, v: impl Into<Value>) -> El {
        self.with_attr("youngsModulus", v)
    }
}
