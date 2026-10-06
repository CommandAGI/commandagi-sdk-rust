// The ontology's files in Rust against the documents the TypeScript SDK's ontology.test.ts pins for the same elements:
// a world, a definition, a dashboard and a geo project leave as the one shape of a document, each site by its tree
// path; a node graph is the editor's own op graph, wires bound to ports with the literal kept and each wire's site in
// its target's meta.sources; what a vocabulary has no words for is refused by name.

use commandagi::design::ontology::*;
use commandagi::design::{declare, Declared};

fn document(root: El) -> (String, Json, Json) {
    match declare(root).unwrap() {
        Declared::Document { format, document, sources } => (format, document, sources),
        Declared::Graph(_) => panic!("a document, not a graph"),
    }
}

fn bad(root: El) -> String {
    declare(root).unwrap_err()
}

fn j(text: &str) -> Json {
    Json::parse(text).unwrap()
}

fn site(el: &El) -> Json {
    el.source_json().unwrap()
}

#[test]
fn a_world_declares_its_units_space_and_a_scenes_bodies_each_site_by_its_path() {
    let arm = unit().uid("arm").name("arm").device("../../devices/so-101/definition.tsx").position([100, 0, 0]).rotation(90);
    let arm_site = site(&arm);
    let file = world([
        space().origin_mm([0, 0, 0]).size_mm([4000, 3000, 2500]),
        arm,
        scene([body().id("cube").shape(json(r#"{"type": "box", "hx": 0.02}"#)).at([0, 0, 1])]).hz(240),
    ])
    .name("Shop")
    .kind("simulation");
    let root_site = site(&file);
    let (format, doc, sources) = document(file);
    assert_eq!(format, "world");
    assert_eq!(
        doc,
        j(r#"{"name": "Shop", "kind": "simulation", "space": {"origin_mm": [0, 0, 0], "size_mm": [4000, 3000, 2500]},
              "units": [{"uid": "arm", "name": "arm", "device": "../../devices/so-101/definition.tsx", "position": [100, 0, 0], "rotation": 90}],
              "scene": {"hz": 240, "bodies": [{"id": "cube", "shape": {"type": "box", "hx": 0.02}, "at": [0, 0, 1]}]}}"#)
    );
    assert_eq!(sources.get(""), Some(&root_site));
    assert_eq!(sources.get("unit#arm"), Some(&arm_site));
    assert!(sources.get("space").is_some() && sources.get("scene/body#cube").is_some(), "every element by its path: {}", sources.text());

    let w = |children: Vec<El>| world(children).name("W").kind("physical");
    assert!(bad(world([]).name("W")).contains("world() needs kind"));
    assert!(bad(world([]).name("W").kind("real")).contains(r#"physical" or "simulation", said explicitly, not "real""#));
    assert!(bad(w(vec![unit().uid("a").name("a").device("d.json").speed(3)])).contains(r#"unit().uid("a"): speed is not read"#));
    assert!(bad(w(vec![unit().uid("a").name("a").device("d"), unit().uid("a").name("b").device("d")])).contains(r#"two unit() in world() have uid "a""#));
    assert!(bad(w(vec![body().id("b")])).contains("body() stands in scene(), not in world()"));
    assert!(bad(w(vec![node().id("n")])).contains("node() is not an element of a world"));
}

#[test]
fn a_device_definitions_channels_are_channel_elements() {
    let def = device([channel()
        .id("joints")
        .dir("duplex")
        .medium("records")
        .format("servo-bus")
        .transport("serial")
        .limits(json(r#"{"j1": [-90, 90]}"#))
        .min_interval_ms(20)])
    .name("Arm")
    .interface("USB serial")
    .model(json(r#"{"file": "model.glb"}"#));
    let (format, doc, _) = document(def);
    assert_eq!(format, "device");
    assert_eq!(
        doc,
        j(r#"{"name": "Arm", "interface": "USB serial", "model": {"file": "model.glb"},
              "channels": [{"id": "joints", "dir": "duplex", "medium": "records", "format": "servo-bus", "transport": "serial", "limits": {"j1": [-90, 90]}, "minIntervalMs": 20}]}"#)
    );
    assert!(bad(device([]).name("A").channels(json("[]"))).contains("each channel as a channel() element"));
}

#[test]
fn a_dashboards_layout_is_nested_splits_of_panes_its_params_and_regions_elements() {
    let file = dashboard([
        param().id("unit").type_("unit"),
        split([pane().id("a").kind("world").world("${unit.world}"), pane().id("b").kind("devices").theater(true)]).axis("x").ratio(0.5),
        region().side("left").tab("nav"),
    ])
    .name("cockpit")
    .focus("b");
    let (format, doc, sources) = document(file);
    assert_eq!(format, "dashboard");
    assert_eq!(
        doc,
        j(r#"{"format": "commandagi-dashboard", "name": "cockpit", "focus": "b", "params": [{"id": "unit", "type": "unit"}],
              "layout": {"kind": "split", "axis": "x", "ratio": 0.5, "children": [{"kind": "leaf", "paneId": "a"}, {"kind": "leaf", "paneId": "b"}]},
              "panes": {"a": {"kind": "world", "world": "${unit.world}"}, "b": {"kind": "devices", "theater": true}},
              "regions": {"left": {"tab": "nav"}}}"#)
    );
    let keys: Vec<&str> = match &sources {
        Json::Obj(e) => e.iter().map(|(k, _)| k.as_str()).collect(),
        _ => vec![],
    };
    assert_eq!(keys, ["", "param#unit", "split@0", "split@0/pane#a", "split@0/pane#b", "region#left"]);
    assert!(bad(dashboard([split([pane().id("a")]).axis("x").ratio(0.5)]).name("x")).contains("a split() has two sides"));
    assert!(bad(dashboard([pane().id("a"), pane().id("b")]).name("x")).contains("has one layout"));
}

/// A tag the trait has no method for: a method of the file's own sets the field by its TypeScript name.
trait Theater {
    fn theater(self, v: impl Into<Value>) -> El;
}
impl Theater for El {
    fn theater(self, v: impl Into<Value>) -> El {
        self.with_attr("theater", v)
    }
}

#[test]
fn a_geo_projects_date_range_and_camera_are_elements() {
    let (format, doc, _) =
        document(geoproject([date_range().start("2015-01").end("2030-01"), camera().center_lng(40).center_lat(25).scale(320)]).id("p1").name("Chokepoints"));
    assert_eq!(format, "geo");
    assert_eq!(
        doc,
        j(r#"{"type": "geoeconomics/project", "schemaVersion": "1.0", "id": "p1", "name": "Chokepoints",
              "data": {"defaultDateRange": {"start": "2015-01", "end": "2030-01"}, "defaultCamera": {"centerLng": 40, "centerLat": 25, "scale": 320}}}"#)
    );
}

#[test]
fn a_node_graph_is_the_editors_op_graph_with_wires_into_ports_and_the_literal_kept() {
    let g = node().id("g").type_("gradient").x(40).y(80).shade_a(40);
    let into_b = wire().from("g:out").to("b:in");
    let (g_site, w_site) = (site(&g), site(&into_b));
    let file = opgraph([
        g,
        node().id("b").type_("blur").x(300).y(80).radius(6).output(true),
        node().id("m").type_("mix").inputs(json(r#"{"layers.2": null}"#)),
        into_b,
        wire().from("g:out").to("b:radius"),
        wire().from("b:out").to("m:layers.1"),
    ])
    .name("Mix");
    let Declared::Graph(graph) = declare(file).unwrap() else { panic!("a node graph is a graph") };
    let want = j(&format!(
        r#"{{"id": "nodegraph", "nodes": {{
              "g": {{"id": "g", "type": "gradient", "inputs": {{"shadeA": 40}}, "meta": {{"x": 40, "y": 80, "source": {g}}}}},
              "b": {{"id": "b", "type": "blur", "inputs": {{"radius": {{"wire": {{"node": "g", "port": "out"}}, "value": 6}}, "in": {{"wire": {{"node": "g", "port": "out"}}}}}},
                    "meta": {{"x": 300, "y": 80, "source": {b}, "sources": {{"wire:in": {w}, "wire:radius": {r}}}}}}},
              "m": {{"id": "m", "type": "mix", "inputs": {{"layers.2": null, "layers.1": {{"wire": {{"node": "b", "port": "out"}}}}}}, "meta": {{"source": {m}, "sources": {{"wire:layers.1": {l}}}}}}}}},
            "outputs": ["b"], "meta": {{"name": "Mix", "domain": "nodegraph"}}}}"#,
        g = g_site.text(),
        w = w_site.text(),
        b = graph.get("nodes").and_then(|n| n.get("b")).and_then(|n| n.get("meta")).and_then(|m| m.get("source")).unwrap().text(),
        r = graph.get("nodes").and_then(|n| n.get("b")).and_then(|n| n.get("meta")).and_then(|m| m.get("sources")).and_then(|s| s.get("wire:radius")).unwrap().text(),
        m = graph.get("nodes").and_then(|n| n.get("m")).and_then(|n| n.get("meta")).and_then(|m| m.get("source")).unwrap().text(),
        l = graph.get("nodes").and_then(|n| n.get("m")).and_then(|n| n.get("meta")).and_then(|m| m.get("sources")).and_then(|s| s.get("wire:layers.1")).unwrap().text(),
    ));
    assert_eq!(graph, want);

    assert!(bad(opgraph([node().id("a").type_("t").inputs(json(r#"{"radius": 1}"#))])).contains("write radius as a method"));
    assert!(bad(opgraph([node().id("a").type_("t"), wire().from("a:out").to("z:in")])).contains(r#"no node().id("z")"#));
    assert!(bad(opgraph([node().id("a").type_("t"), wire().from("a").to("a:in")])).contains(r#""node:port""#));
    assert!(bad(opgraph([node().id("a").type_("t"), wire().from("a:out").to("a:in"), wire().from("a:out").to("a:in")])).contains("have to"));
    assert!(bad(opgraph([node().id("a:b").type_("t")])).contains("an id has no colon"));
}
