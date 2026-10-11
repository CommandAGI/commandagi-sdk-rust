// A 3D document in Rust declares the graph the TypeScript SDK's threed.test.ts pins for the same elements: features
// as nodes of their type, a sketch's entities in its `sketch` port, parameters as inputs whose drives are their
// bindings, the built-in planes, bodies and slots; each call's site in `meta.source` / `meta.sources`.

use commandagi::design::threed::*;
use commandagi::design::{declare, El, Json};

fn graph(root: El) -> Json {
    declare(root).unwrap().graph().unwrap()
}

fn bad(root: El) -> String {
    declare(root).unwrap_err()
}

fn node<'a>(g: &'a Json, id: &str) -> &'a Json {
    g.get("nodes").and_then(|n| n.get(id)).unwrap_or_else(|| panic!("no node {id}"))
}

fn site(line: u32, column: u32) -> Json {
    Json::obj().with("site", vec![Json::from(line), Json::from(column)])
}

fn parse(text: &str) -> Json {
    Json::parse(text).unwrap()
}

#[test]
fn a_part_declares_the_documents_graph() {
    let l0 = line!() + 2;
    let plate = part([
        parameter().name("depth").value(6).unit("mm").bindings(json(r#"[{"target": "extrude1", "field": "distance"}]"#)),
        sketch([
            point().id("p1").x(0).y(0),
            point().id("p2").x(40).y(0),
            point().id("p3").x(40).y(20),
            line().id("l1").a("p1").b("p2"),
            line().id("l2").a("p2").b("p3"),
            line().id("l3").a("p3").b("p1"),
            constraint().id("c1").kind("horizontal").entities(["l1"]),
        ])
        .id("sketch1")
        .name("Sketch1")
        .plane(json(r#"{"type": "datum", "plane": "plane_xy"}"#)),
        extrude().id("extrude1").name("Extrude1").profile(json(r#"{"sketch": "sketch1"}"#)).distance(6).operation("new"),
        fillet().id("fillet1").name("Fillet1").radius(1).edges(json(r#"[{"kind": "edge", "signature": {"point": [20, 0, 6]}}]"#)).consumes(["extrude1"]).suppressed(true),
        body().id("extrude1").material("aluminium-6061"),
        slot().name("environment").value(json(r#"{"gravity": [0, 0, -9.81]}"#)),
    ])
    .name("Plate");
    let g = graph(plate);
    assert_eq!(g.get("id").unwrap().text(), r#""3d-plate""#);
    assert_eq!(g.get("meta").unwrap().text(), parse(r#"{"name":"Plate","units":"mm","presentation":{"order":["sketch1","extrude1","fillet1"]}}"#).text());
    let with_meta = |text: &str, meta: Json| parse(text).with("meta", meta).text();
    assert_eq!(
        node(&g, "depth").text(),
        with_meta(r#"{"id":"depth","type":"input","label":"depth","inputs":{"value":6,"unit":"mm","drives":[{"target":"extrude1","field":"distance"}]}}"#, Json::obj().with("source", site(l0, 9)))
    );
    assert_eq!(
        node(&g, "extrude1").text(),
        with_meta(r#"{"id":"extrude1","type":"extrude","label":"Extrude1","inputs":{"profile":{"sketch":"sketch1"},"distance":6,"operation":"new"}}"#, Json::obj().with("source", site(l0 + 13, 9)))
    );
    let fillet = node(&g, "fillet1").clone().with("meta", Json::Null);
    assert_eq!(fillet.text(), r#"{"id":"fillet1","type":"fillet","label":"Fillet1","disabled":true,"inputs":{"radius":1,"edges":[{"kind":"edge","signature":{"point":[20,0,6]}}],"consumes":["extrude1"]},"meta":null}"#);
    let sketch1 = node(&g, "sketch1");
    let sk = sketch1.get("inputs").and_then(|i| i.get("sketch")).unwrap();
    assert_eq!(sk.get("pointOrder").unwrap().text(), r#"["p1","p2","p3"]"#);
    assert_eq!(sk.get("segments").and_then(|s| s.get("l1")).unwrap().text(), r#"{"id":"l1","type":"line","a":"p1","b":"p2"}"#);
    assert_eq!(sk.get("constraints").and_then(|s| s.get("c1")).unwrap().text(), r#"{"id":"c1","kind":"horizontal","entities":["l1"]}"#);
    let sources = Json::obj()
        .with("point:p1", site(l0 + 2, 13))
        .with("point:p2", site(l0 + 3, 13))
        .with("point:p3", site(l0 + 4, 13))
        .with("segment:l1", site(l0 + 5, 13))
        .with("segment:l2", site(l0 + 6, 13))
        .with("segment:l3", site(l0 + 7, 13))
        .with("constraint:c1", site(l0 + 8, 13));
    assert_eq!(sketch1.get("meta").unwrap().text(), Json::obj().with("source", site(l0 + 1, 9)).with("sources", sources).text());
    assert_eq!(node(&g, "plane_xy").get("inputs").unwrap().text(), r#"{"origin":[0,0,0],"normal":[0,0,1],"xAxis":[1,0,0],"builtin":"XY"}"#);
    assert_eq!(
        node(&g, "bodyMeta").text(),
        with_meta(r#"{"id":"bodyMeta","type":"3d.bodyMeta","inputs":{"extrude1":{"material":"aluminium-6061"}}}"#, Json::obj().with("sources", Json::obj().with("extrude1", site(l0 + 15, 9))))
    );
    assert_eq!(node(&g, "environment").get("inputs").unwrap().text(), r#"{"gravity":[0,0,-9.81]}"#);
}

#[test]
fn an_assembly_opens_as_one_and_what_a_3d_document_cannot_say_is_refused_by_name() {
    assert_eq!(graph(assembly([]).name("A")).get("meta").and_then(|m| m.get("isAssembly")), Some(&Json::Bool(true)));
    let doc = |children: Vec<El>| bad(part(children).name("P"));
    assert_eq!(doc(vec![El::new("widget").with_attr("id", "w")]), "widget() is not read in a 3D document (see commandagi::design::threed)");
    assert_eq!(doc(vec![box_().name("B")]), r#"box_().name("B") needs an id"#);
    assert_eq!(doc(vec![box_().id("a"), plane().id("a")]), r#"the id "a" is both plane "a" and feature "a"; a node id is used once"#);
    assert!(doc(vec![sketch([line().id("l").a("p1").b("p2")]).id("s")]).contains("segment l's a names no point"));
    assert!(doc(vec![sketch([box_().id("b")]).id("s")]).contains("box_() is not read in a sketch"));
    assert!(doc(vec![slot().name("features").value(json("{}"))]).contains("features is not a slot"));
    assert!(doc(vec![extrude().id("e").distance(f64::INFINITY)]).ends_with("distance is Infinity, not a finite number"));
    assert_eq!(doc(vec![parameter().name("w").value("6")]), r#"parameter().name("w"): value is a number"#);
    assert!(doc(vec![feature().type_("extrude").id("e")]).ends_with("a extrude is written extrude(…)"));
    assert!(doc(vec![box_().id("a//b")]).contains("an id is letters, digits, _ . - with / between them"));
    assert!(bad(part([]).name("P").builtin_planes(["XY"])).contains("builtin_planes lists built-in planes"));
    assert!(bad(part([]).name("P").with_attr("colour", "red")).ends_with("colour is not read on a 3D document"));
}

#[test]
fn a_subset_of_the_built_in_planes_an_id_with_a_slash_and_a_feature_of_an_unknown_type() {
    let planes = |g: &Json| {
        let Some(Json::Obj(nodes)) = g.get("nodes") else { panic!() };
        let mut ids: Vec<String> = nodes.iter().filter(|(_, n)| n.get("type") == Some(&Json::from("plane"))).map(|(id, _)| id.clone()).collect();
        ids.sort();
        ids
    };
    assert_eq!(planes(&graph(part([]).name("P"))), ["plane_xy", "plane_xz", "plane_yz"]);
    assert!(planes(&graph(part([]).name("P").builtin_planes(Vec::<&str>::new()))).is_empty());
    let xy = plane().id("XY").name("XY").origin([0, 0, 0]).normal([0, 0, 1]).x_axis([1, 0, 0]).builtin("XY");
    assert_eq!(planes(&graph(part([xy]).name("P").builtin_planes(["plane_xz"]))), ["XY", "plane_xz"]);
    let g = graph(part([cylinder().id("fan/bore").center([0, 0, 0]).axis([0, 0, 1]).radius(2).height(5).operation("cut"), feature().type_("rotate").id("r1").name("rotate_y_30").axis([0, 1, 0]).angle(30)]).name("P"));
    assert_eq!(node(&g, "fan/bore").get("inputs").unwrap().text(), r#"{"center":[0,0,0],"axis":[0,0,1],"radius":2,"height":5,"operation":"cut"}"#);
    let mut r1 = node(&g, "r1").clone();
    r1.remove("meta");
    assert_eq!(r1.text(), r#"{"id":"r1","type":"rotate","label":"rotate_y_30","inputs":{"axis":[0,1,0],"angle":30}}"#);
    assert_eq!(g.get("meta").and_then(|m| m.get("presentation")).unwrap().text(), r#"{"order":["fan/bore","r1"]}"#);
}

#[test]
fn a_code_features_inputs_are_ports_of_its_node() {
    let g = graph(part([code().id("bracket").source("bracket.3d.py").inputs(json(r#"{"width": 40}"#)).consumes(["base"])]).name("P"));
    assert_eq!(node(&g, "bracket").get("inputs").unwrap().text(), r#"{"source":"bracket.3d.py","consumes":["base"],"width":40}"#);
    assert_eq!(bad(part([code().id("c").source("a.py").distance(3)]).name("P")), r#"code().id("c"): distance: is not one of its fields"#);
}

#[test]
fn a_feature_element_whose_fields_do_not_fit_its_type_is_refused_with_the_op_panels_sentence_for_each() {
    let read = |children: Vec<El>| bad(part(children).name("P"));
    let profile = || json(r#"{"sketch": "s1"}"#);
    assert_eq!(read(vec![extrude().id("e1").profile(profile()).distance(-1).operation("new")]), r#"extrude().id("e1"): distance: must be greater than 0 (got -1)"#);
    assert_eq!(
        read(vec![extrude().id("e1").profile(profile()).distance(5).operation("glue").with_attr("colour", "red")]),
        r#"extrude().id("e1"): operation: must be one of new, add, cut, intersect (got "glue"); colour: is not one of its fields"#
    );
    assert_eq!(read(vec![extrude().id("e1").distance(5).operation("new")]), r#"extrude().id("e1"): profile: is required"#);
    let s1 = sketch([circle().id("c").center("p").radius("big"), point().id("p").x(0).y(0)]).id("s1").plane(json(r#"{"type": "datum", "plane": "plane_xy"}"#));
    assert_eq!(read(vec![s1]), r#"sketch().id("s1"): sketch/segments/c/radius: must be number (got "string")"#);
    // A large array kept in a file beside the document stands for the array.
    let positions = json(r#"{"$file": "P.assets/m/positions.f64", "mime": "application/octet-stream", "encoding": "f64le", "len": 9}"#);
    assert!(declare(part([point_cloud().id("pc").positions(positions)]).name("P")).is_ok());
    assert!(THREED_FEATURES.contains(&"makeHuman") && !THREED_FEATURES.contains(&"makehuman"));
}

#[test]
fn a_required_field_with_a_declared_default_may_be_left_out() {
    // The TypeScript and Python SDKs read the same assembly to the same inputs.
    let read = |children: Vec<El>| declare(assembly(children).name("A")).map(|d| d.graph().unwrap());
    let g = read(vec![
        external_part().id("bare").path("bracket.3d.tsx"),
        external_part().id("moved").path("bracket.3d.tsx").placement(json(r#"{"position": [10, 0, 5]}"#)).operation("add"),
    ])
    .unwrap();
    let identity = r#"{"position":[0,0,0],"quaternion":[0,0,0,1],"scale":1}"#;
    assert_eq!(node(&g, "bare").get("inputs").unwrap().text(), format!(r#"{{"path":"bracket.3d.tsx","placement":{identity},"operation":"new"}}"#));
    // A placement that says only where: its orientation and scale are the identity's. The author's values stay.
    assert_eq!(
        node(&g, "moved").get("inputs").unwrap().text(),
        r#"{"path":"bracket.3d.tsx","placement":{"position":[10,0,5],"quaternion":[0,0,0,1],"scale":1},"operation":"add"}"#
    );
    // An optional field stays absent: its absence is a value the kernel reads.
    assert!(node(&g, "bare").get("inputs").unwrap().get("importer").is_none());
    // A defaulted field the author wrote wrong is still refused, and a field with no default is still required.
    assert_eq!(
        read(vec![external_part().id("p").path("b.3d.tsx").placement(json(r#"{"scale": "big"}"#))]).unwrap_err(),
        r#"external_part().id("p"): placement/scale: must be number (got "string")"#
    );
    assert_eq!(read(vec![import().id("i").format("step")]).unwrap_err(), r#"import().id("i"): data: is required"#);
}

#[test]
fn a_world_body_written_by_tsx_reads_with_its_declared_fields() {
    // The element an ingest writes in a .3d.tsx (sdk/typescript threed.test.ts reads the same one).
    let world_body = |collider: &str, restitution: f64| {
        declare(
            part([mesh_body()
                .id("world/t")
                .name("t")
                .positions(json("[0, 0, 0, 1000, 0, 0, 0, 1000, 0, 1000, 1000, 0]"))
                .indices(json("[0, 1, 2, 1, 3, 2]"))
                .with_attr("worldCollider", json(collider))
                .with_attr("worldCenter", json("[0, 0, 100]"))
                .with_attr("worldFriction", 0.8)
                .with_attr("worldRestitution", restitution)
                .with_attr("worldMeta", json(r#"{"tile": "12/1/2", "res": 30}"#))])
            .name("W"),
        )
    };
    let g = world_body(r#"{"type": "heightfield", "nx": 2, "ny": 2, "sizeX": 1000, "sizeY": 1000}"#, 0.1).unwrap().graph().unwrap();
    assert_eq!(node(&g, "world/t").get("inputs").unwrap().get("worldMeta").unwrap().text(), r#"{"tile":"12/1/2","res":30}"#);
    assert!(world_body(r#"{"type": "trimesh"}"#, 0.1).is_ok());
    assert_eq!(
        world_body(r#"{"type": "heightfield", "nx": 1, "ny": 2, "sizeX": 1000, "sizeY": 1000}"#, 2.0).unwrap_err(),
        r#"mesh_body().id("world/t"): worldCollider/nx: must be at least 2 (got 1); worldRestitution: must be at most 1 (got 2)"#
    );
}

#[test]
fn the_tags_are_the_schemas_roots_and_its_typed_props_make_the_elements_the_reader_takes() {
    use commandagi::design::threed_props::*;
    let schema = threed_schema();
    let Some(Json::Obj(roots)) = schema.get("roots") else { panic!("no roots") };
    assert_eq!(roots.iter().map(|(t, _)| t.as_str()).collect::<Vec<_>>(), THREED_FEATURES);
    // This crate reads no `pattern` (it has no regex engine): the schema it carries has none.
    assert!(!include_str!("../src/design/threed_schema.json").contains(r#""pattern""#));
    let s = SketchProps { id: "s1".into(), name: None, plane: PlaneRef::Datum(PlaneRefDatum { plane: "plane_xy".into() }), suppressed: None, consumes: None }
        .el([SketchPointProps { pin: None, id: "p".into(), x: 0.0, y: 0.0, construction: None, fixed: None, from: None }.el()]);
    let props = ExtrudeProps {
        id: "e1".into(),
        name: Some("Extrude1".into()),
        profile: ProfileRef { sketch: "s1".into(), regions: None },
        distance: Some(6.0),
        symmetric: None,
        reverse: None,
        distance2: None,
        draft_angle: None,
        through: None,
        up_to_face: None,
        operation: Some(BooleanMode::New),
        suppressed: None,
        consumes: None,
    };
    let (e, line) = (props.el(), line!());
    let g = graph(part([s, e]).name("P"));
    let mut n = node(&g, "e1").clone();
    // The element's site is the `.el()` call's.
    assert_eq!(n.remove("meta").unwrap().text(), Json::obj().with("source", site(line, 28)).text());
    assert_eq!(n.text(), r#"{"id":"e1","type":"extrude","label":"Extrude1","inputs":{"profile":{"sketch":"s1"},"distance":6,"operation":"new"}}"#);
}
