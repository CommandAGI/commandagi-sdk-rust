//! THE ONTOLOGY'S OWN FILES IN RUST — worlds, device definitions, dashboards, geo projects and node graphs, written as
//! the elements their editors edit. The same elements, and the same documents, as the TypeScript SDK's `ontology.ts`
//! and the Python SDK's `ontology.py`:
//!
//! ```
//! use commandagi::design::ontology::*;
//!
//! fn document() -> El {
//!     world([
//!         unit().uid("arm").name("arm").device("../../devices/so-101/definition.tsx").position([0, 0, 0]).rotation(0),
//!     ])
//!     .name("Shop")
//!     .kind("physical")
//! }
//! # commandagi::design::declare(document()).unwrap();
//! ```
//!
//! A record is an element and its fields are the element's attributes, verbatim; a list of records is the parent's
//! children (`units` → `unit()`, `channels` → `channel()`, a scene's `bodies` → `body()`). The tags:
//!
//! ```text
//! world([…]).name.kind.description?.from?.model?     world.rs             space().origin_mm.size_mm, unit().uid …,
//!                                                                         view(), scene([body().id …])
//! device([…]).name …                                 definition.rs        channel().id.dir.medium.format.transport …
//! dashboard([…]).name …                              <name>.dashboard.rs  param().id.type_ …, one layout: split([a, b])
//!                                                                         .axis.ratio of two split|pane().id.kind …, or
//!                                                                         one pane(); region().side.tab …
//! geoproject([…]).id.name …                          <name>.geo.rs        date_range().start.end, camera(), reference()
//! opgraph([…]).name                                  <name>.opgraph.rs    node().id.type_.x.y …ports, wire().from.to
//! ```
//!
//! A field whose own name has an underscore (a world's `origin_mm`, `size_mm`, a definition's `record_hz`) is its
//! method as it is. A node graph is the editor's op graph itself: `node().id("blur").type_("blur").x(300).y(80)
//! .radius(6)` is the node `{"id": "blur", "type": "blur", "inputs": {"radius": 6}, "meta": {"x": 300, "y": 80}}`, and
//! `wire().from("gradient:out").to("blur:in")` the wire into its `in` port; `.output(true)` marks a terminal. A port
//! whose name is not an attribute name (`layers.3`) is written in `.inputs(json(r#"{"layers.3": null}"#))`.
//!
//! [`OntologyAttrs`] has a method for each field the ontology's files and the shipped definitions use. A field it has
//! no method for (a port of a node type of your own) is a method of a trait of the file's own, which sets the field by
//! its TypeScript name: `fn cutoff(self, v: impl Into<Value>) -> El { self.with_attr("cutoff", v) }`.
//!
//! A run gives back what the TypeScript SDK gives: for a world, a definition, a dashboard and a geo project, the
//! document (`{format, document, sources}`, each element's site keyed by its tree path) beside an empty graph; for a
//! node graph, the editor's own op graph, each node with `meta.source` and each wire's site in its target's
//! `meta.sources["wire:<port>"]`. A code form declares; it never enforces: the host reads a definition's bounds.

use super::documents::{declare_document, js_string, tree_of_element, DocTree, TagRule, Vocabulary};
use super::element::{attributes, elements, rust_name};
use super::{Declared, Family};
pub use super::{fragment, json, El, Json, Value};

elements! {
    /// A world: its space, its units, its view and its scene.
    world: children;
    /// A world's space: `.origin_mm([…]).size_mm([…])`.
    space: leaf;
    /// A unit placed in a world: `.uid("arm").name("arm").device("…/definition.tsx")`.
    unit: leaf;
    /// A world's view.
    view: leaf;
    /// A world's scene: its bodies.
    scene: children;
    /// A body of a scene: `.id("cube").shape(json(…))`.
    body: leaf;
    /// A device definition: its channels.
    device: children;
    /// A channel of a device: `.id("joints").dir("duplex").medium("records")`.
    channel: leaf;
    /// A dashboard: its params, its one layout, its regions.
    dashboard: children;
    /// A dashboard's param: `.id("log").type_("file")`.
    param: leaf;
    /// A split of a dashboard's layout: two sides, each a split or a pane; `.axis("x").ratio(0.5)`.
    split: children;
    /// A pane of a dashboard's layout: `.id("scene").kind("world")`.
    pane: leaf;
    /// A region of a dashboard: `.side("left").tab("explorer")`.
    region: leaf;
    /// A geo project's manifest.
    geoproject: children;
    /// A geo project's date range: `.start("2015-01").end("2030-01")`.
    date_range = "dateRange": leaf;
    /// A geo project's camera.
    camera: leaf;
    /// A file a geo project names: `.role("layer").path("…")`.
    reference: leaf;
    /// A node graph: its nodes and wires.
    opgraph: children;
    /// A node of a node graph: `.id("blur").type_("blur").x(300).y(80)`, its ports' values beside.
    node: leaf;
    /// A wire of a node graph: `.from("gradient:out").to("blur:in")`.
    wire: leaf;
}

attributes! {
    /// The fields of the ontology's records, chained on their elements.
    pub trait OntologyAttrs {
        // A world's, a unit's and a body's.
        name; kind; description; from; model; uid; device; domain; support; channels; manual_url; position; rotation;
        origin_mm = "origin_mm"; size_mm = "size_mm"; dimensions_mm = "dimensions_mm";
        hz; at; rot; vel; shape; mass; friction; restitution; angular_damping = "angular_damping"; gravity; scale;
        backdrop; walls; grids; bodies; forces; particles; entities; relations; systems; robots; joints; links; kinematics;
        dynamics; constraints; fidelity; motion; color; blend; balance; coupled; seals; release; optics; root;
        // A definition's and a channel's.
        id; type_; label; dir; medium; format; transport; interface; manufacturer; model_number = "model_number";
        product_url = "product_url"; price_usd = "price_usd"; record_hz = "record_hz"; spec_check = "spec_check";
        limits; min_interval_ms; verbs; units; unit; rate; baud; dialect; encoding; conversion; parameters; params; editor;
        notes; status; suggest; speed; source; title; value; rank;
        // A dashboard's, a param's, a pane's and a region's.
        focus; accept; default; required; axis; ratio; side; tab; open; collapsed; selected; path; view; world; param;
        file; group; pad;
        // A geo project's.
        created_at; updated_at; default_workspace; metadata; start; end; center_lat; center_lng; zoom; pitch; bearing;
        role; hash;
        // A node graph's and a node's (the ports of the node-graph editor's own node types are fields too).
        meta; inputs; output; disabled; x; y; to; shade_a; shade_b; angle; seed; density; radius; freq; octave;
    }
}

pub(crate) const FAMILY: Family =
    Family { module: "ontology", roots: &["world", "device", "dashboard", "geoproject", "opgraph"], declare };

/// The graph id and domain a node graph has unless its file says another (the node-graph editor's new graph).
pub const NODE_GRAPH_ID: &str = "nodegraph";
/// The fields of a unit in a world's file (packages/domain/world/worlds.js).
pub const UNIT_FIELDS: &[&str] = &["uid", "name", "device", "domain", "size_mm", "support", "channels", "manualUrl", "position", "rotation"];
const WORLD_FIELDS: &[&str] = &["name", "kind", "description", "from", "model"];
const GEO_FIELDS: &[&str] = &["id", "name", "createdAt", "updatedAt", "description", "defaultWorkspace", "metadata"];
/// A node's attributes that are not its ports.
const NODE_ATTRS: &[&str] = &["id", "type", "label", "x", "y", "disabled", "output", "meta", "inputs"];
const DASHBOARD_STRUCTURE: &[&str] = &["format", "layout", "panes", "params", "regions"];

const WORLD: Vocabulary = Vocabulary {
    format: "world",
    noun: "a world",
    root: "world",
    tags: &[
        TagRule::new("world", &[]).required(&["name", "kind"]).attrs(WORLD_FIELDS),
        TagRule::new("space", &["world"]).single().required(&["origin_mm", "size_mm"]).attrs(&["origin_mm", "size_mm"]),
        TagRule::new("unit", &["world"]).key("uid").required(&["uid", "name", "device"]).attrs(UNIT_FIELDS),
        TagRule::new("view", &["world"]).single(),
        TagRule::new("scene", &["world"]).single(),
        TagRule::new("body", &["scene"]).key("id").required(&["id"]),
    ],
    from_tree: world_of,
};

const DEVICE: Vocabulary = Vocabulary {
    format: "device",
    noun: "a device definition",
    root: "device",
    tags: &[TagRule::new("device", &[]).required(&["name"]), TagRule::new("channel", &["device"]).key("id").required(&["id"])],
    from_tree: device_of,
};

const DASHBOARD: Vocabulary = Vocabulary {
    format: "dashboard",
    noun: "a dashboard",
    root: "dashboard",
    tags: &[
        TagRule::new("dashboard", &[]).required(&["name"]),
        TagRule::new("param", &["dashboard"]).key("id").required(&["id", "type"]),
        TagRule::new("split", &["dashboard", "split"]).required(&["axis", "ratio"]).attrs(&["axis", "ratio"]),
        TagRule::new("pane", &["dashboard", "split"]).key("id").required(&["id"]),
        TagRule::new("region", &["dashboard"]).key("side").required(&["side"]),
    ],
    from_tree: dashboard_of,
};

const GEO: Vocabulary = Vocabulary {
    format: "geo",
    noun: "a geo project",
    root: "geoproject",
    tags: &[
        TagRule::new("geoproject", &[]).required(&["id", "name"]).attrs(GEO_FIELDS),
        TagRule::new("dateRange", &["geoproject"]).single().required(&["start", "end"]),
        TagRule::new("camera", &["geoproject"]).single(),
        TagRule::new("reference", &["geoproject"]).key("path").required(&["role", "path"]).attrs(&["role", "path", "hash"]),
    ],
    from_tree: geo_of,
};

const OPGRAPH: Vocabulary = Vocabulary {
    format: "opgraph",
    noun: "a node graph",
    root: "opgraph",
    tags: &[
        TagRule::new("opgraph", &[]).attrs(&["id", "name", "domain", "meta"]),
        TagRule::new("node", &["opgraph"]).key("id").required(&["id", "type"]),
        TagRule::new("wire", &["opgraph"]).key("to").required(&["from", "to"]).attrs(&["from", "to"]),
    ],
    from_tree: opgraph_of,
};

/// The vocabularies of the ontology's files.
pub const ONTOLOGY: &[&Vocabulary] = &[&WORLD, &DEVICE, &DASHBOARD, &GEO, &OPGRAPH];

/// Declare a world, a definition, a dashboard or a geo project (a document), or a node graph (a graph).
fn declare(root: El) -> Result<Declared, String> {
    if root.tag == "opgraph" {
        return Ok(Declared::Graph(opgraph_of(&tree_of_element(&root, &OPGRAPH)?)?));
    }
    let v = ONTOLOGY.iter().find(|v| v.root == root.tag).expect("a root of this family");
    declare_document(&root, v)
}

// ── Records ────────────────────────────────────────────────────────────────────────────────────────────────────────

/// The fields of `o` named in `keys`, in the order of `keys`.
fn own(o: &[(String, Json)], keys: &[&str]) -> Vec<(String, Json)> {
    keys.iter().filter_map(|k| o.iter().find(|(n, _)| n == k).cloned()).collect()
}

/// The fields of `o` but those named in `keys`.
fn omit(o: &[(String, Json)], keys: &[&str]) -> Vec<(String, Json)> {
    o.iter().filter(|(k, _)| !keys.contains(&k.as_str())).cloned().collect()
}

fn kids<'a>(t: &'a DocTree, tag: &'a str) -> impl Iterator<Item = &'a DocTree> {
    t.children.iter().filter(move |c| c.tag == tag)
}

/// A value as a message shows it: its JSON text (JavaScript's `JSON.stringify`).
fn text(v: Option<&Json>) -> String {
    v.map(Json::text).unwrap_or_else(|| "undefined".into())
}

fn is_record(v: Option<&Json>) -> bool {
    matches!(v, Some(Json::Obj(_)))
}

/// How a message names an element: its call with the attribute that names it (`node().id("a")`).
fn call(t: &DocTree, key: &str) -> String {
    let f = super::element::fn_name(t.tag);
    match t.attr(key) {
        Some(v @ Json::Str(_)) => format!("{f}().{}({})", rust_name(key), v.text()),
        _ => format!("{f}()"),
    }
}

// ── world.rs ───────────────────────────────────────────────────────────────────────────────────────────────────────

fn world_of(t: &DocTree) -> Result<Json, String> {
    match t.attr("kind") {
        Some(Json::Str(k)) if k == "physical" || k == "simulation" => {}
        other => {
            return Err(format!("world(): kind is \"physical\" or \"simulation\", said explicitly, not {}", text(other)));
        }
    }
    let mut out = Json::Obj(own(&t.attrs, WORLD_FIELDS));
    if let Some(space) = t.single("space") {
        out.set("space", space.record());
    }
    out.set("units", Json::Arr(kids(t, "unit").map(DocTree::record).collect()));
    if let Some(view) = t.single("view") {
        out.set("view", view.record());
    }
    if let Some(scene) = t.single("scene") {
        let bodies: Vec<Json> = kids(scene, "body").map(DocTree::record).collect();
        let mut s = scene.record();
        if !bodies.is_empty() {
            s.set("bodies", Json::Arr(bodies));
        }
        out.set("scene", s);
    }
    Ok(out)
}

// ── definition.rs ──────────────────────────────────────────────────────────────────────────────────────────────────

fn device_of(t: &DocTree) -> Result<Json, String> {
    if t.attr("channels").is_some() {
        return Err("device(): write each channel as a channel() element, not a channels attribute".into());
    }
    let channels: Vec<Json> = kids(t, "channel").map(DocTree::record).collect();
    let mut out = t.record();
    if !channels.is_empty() {
        out.set("channels", Json::Arr(channels));
    }
    Ok(out)
}

// ── <name>.dashboard.rs ────────────────────────────────────────────────────────────────────────────────────────────

fn dashboard_of(t: &DocTree) -> Result<Json, String> {
    if let Some(k) = DASHBOARD_STRUCTURE.iter().find(|k| t.attr(k).is_some()) {
        return Err(format!("dashboard(): {k} is written as elements (param(), split(), pane(), region()), not an attribute"));
    }
    let is_layout = |c: &&DocTree| c.tag == "split" || c.tag == "pane";
    let layouts: Vec<&DocTree> = t.children.iter().filter(is_layout).collect();
    if layouts.len() != 1 {
        return Err(format!("dashboard() has one layout: a split() or one pane() (it has {})", layouts.len()));
    }
    fn layout(n: &DocTree, panes: &mut Vec<(String, Json)>) -> Result<Json, String> {
        if n.tag == "pane" {
            let id = js_string(n.attr("id"));
            if panes.iter().any(|(k, _)| *k == id) {
                return Err(format!("two pane() have id {:?}", id));
            }
            panes.push((id.clone(), Json::Obj(omit(&n.attrs, &["id"]))));
            return Ok(Json::obj().with("kind", "leaf").with("paneId", id));
        }
        let sides: Vec<&DocTree> = n.children.iter().filter(|c| c.tag == "split" || c.tag == "pane").collect();
        if sides.len() != 2 {
            return Err(format!("a split() has two sides (this one has {})", sides.len()));
        }
        let children = sides.into_iter().map(|c| layout(c, panes)).collect::<Result<Vec<_>, _>>()?;
        let attr = |k: &str| n.attr(k).cloned().unwrap_or(Json::Null);
        Ok(Json::obj().with("kind", "split").with("axis", attr("axis")).with("ratio", attr("ratio")).with("children", children))
    }
    let mut panes = Vec::new();
    let root = layout(layouts[0], &mut panes)?;
    let params: Vec<Json> = kids(t, "param").map(DocTree::record).collect();
    let mut regions = Json::obj();
    for r in kids(t, "region") {
        regions.set(&js_string(r.attr("side")), Json::Obj(omit(&r.attrs, &["side"])));
    }
    let mut out = Json::obj().with("format", "commandagi-dashboard");
    for (k, v) in &t.attrs {
        out.set(k, v.clone());
    }
    if !params.is_empty() {
        out.set("params", Json::Arr(params));
    }
    out.set("layout", root);
    out.set("panes", Json::Obj(panes));
    if regions != Json::obj() {
        out.set("regions", regions);
    }
    Ok(out)
}

// ── <name>.geo.rs (the geo project's manifest) ─────────────────────────────────────────────────────────────────────

fn geo_of(t: &DocTree) -> Result<Json, String> {
    let mut top = Json::Obj(own(&t.attrs, GEO_FIELDS));
    let description = top.remove("description");
    let workspace = top.remove("defaultWorkspace");
    let mut data = Json::obj();
    if let Some(w) = workspace {
        data.set("defaultWorkspace", w);
    }
    if let Some(range) = t.single("dateRange") {
        data.set("defaultDateRange", range.record());
    }
    if let Some(camera) = t.single("camera") {
        data.set("defaultCamera", camera.record());
    }
    if let Some(d) = description {
        data.set("description", d);
    }
    // The geo project's manifest says which shape it is in; this is the one the studio reads.
    let mut out = Json::obj().with("type", "geoeconomics/project").with("schemaVersion", "1.0");
    if let Json::Obj(fields) = top {
        for (k, v) in fields {
            out.set(&k, v);
        }
    }
    let refs: Vec<Json> = kids(t, "reference").map(DocTree::record).collect();
    if !refs.is_empty() {
        out.set("references", Json::Arr(refs));
    }
    Ok(out.with("data", data))
}

// ── <name>.opgraph.rs: the node graph ──────────────────────────────────────────────────────────────────────────────

/// Whether a port is written as a method of its node (else in `.inputs(json(…))`): `[A-Za-z_$][\w$-]*`, not a field.
pub fn port_is_attribute(port: &str) -> bool {
    let mut chars = port.chars();
    let first = chars.next().is_some_and(|c| c.is_ascii_alphabetic() || c == '_' || c == '$');
    first && chars.all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '$' | '-')) && !NODE_ATTRS.contains(&port) && port != "key" && port != "children"
}

fn is_wire(v: Option<&Json>) -> bool {
    match v.and_then(|v| v.get("wire")) {
        Some(w @ Json::Obj(_)) => matches!(w.get("node"), Some(Json::Str(_))) && matches!(w.get("port"), Some(Json::Str(_))),
        _ => false,
    }
}

/// `"gradient:out"` → (node, port): a node id has no colon; a port may have dots (`mix:layers.2`).
fn end_of(text: Option<&Json>, what: &str) -> Result<(String, String), String> {
    let split = match text {
        Some(Json::Str(s)) => s.split_once(':').filter(|(n, p)| !n.is_empty() && !p.is_empty() && !p.contains(['\n', '\r', '\u{2028}', '\u{2029}'])),
        _ => None,
    };
    match split {
        Some((node, port)) => Ok((node.to_string(), port.to_string())),
        None => Err(format!("wire(): {what} is \"node:port\", not {}", self::text(text))),
    }
}

fn opgraph_of(t: &DocTree) -> Result<Json, String> {
    let mut nodes: Vec<(String, Json)> = Vec::new();
    let mut outputs: Vec<Json> = Vec::new();
    for n in kids(t, "node") {
        let id = js_string(n.attr("id"));
        let at = call(n, "id");
        if !matches!(n.attr("type"), Some(Json::Str(s)) if !s.is_empty()) {
            return Err(format!("{at} needs a type_"));
        }
        if id.contains(':') {
            return Err(format!("{at}: an id has no colon (a wire names \"node:port\")"));
        }
        let mut inputs = Json::Obj(n.attrs.iter().filter(|(k, _)| !NODE_ATTRS.contains(&k.as_str())).cloned().collect());
        if let Some(given) = n.attr("inputs") {
            let Json::Obj(ports) = given else { return Err(format!("{at} inputs is an object of ports")) };
            for (k, v) in ports {
                if port_is_attribute(k) {
                    return Err(format!("{at}: write {k} as a method (.{}(…)), not in inputs", rust_name(k)));
                }
                inputs.set(k, v.clone());
            }
        }
        let mut meta = match n.attr("meta") {
            None => Json::obj(),
            Some(m @ Json::Obj(_)) => m.clone(),
            Some(_) => return Err(format!("{at} meta is an object")),
        };
        for k in ["x", "y"] {
            if let Some(v) = n.attr(k) {
                meta.set(k, v.clone());
            }
        }
        if let Some(s) = &n.source {
            meta.set("source", s.clone());
        }
        let mut node = Json::obj().with("id", id.as_str()).with("type", n.attr("type").cloned().unwrap_or(Json::Null));
        if let Some(label @ Json::Str(_)) = n.attr("label") {
            node.set("label", label.clone());
        }
        node.set("inputs", inputs);
        if n.attr("disabled") == Some(&Json::Bool(true)) {
            node.set("disabled", true);
        }
        if meta != Json::obj() {
            node.set("meta", meta);
        }
        if n.attr("output") == Some(&Json::Bool(true)) {
            outputs.push(Json::from(id.as_str()));
        }
        nodes.push((id, node));
    }
    for w in kids(t, "wire") {
        let (from_node, from_port) = end_of(w.attr("from"), "from")?;
        let (to_node, to_port) = end_of(w.attr("to"), "to")?;
        if !nodes.iter().any(|(k, _)| *k == to_node) {
            return Err(format!("wire().to({}): no node().id({})", text(w.attr("to")), Json::from(to_node).text()));
        }
        if !nodes.iter().any(|(k, _)| *k == from_node) {
            return Err(format!("wire().from({}): no node().id({})", text(w.attr("from")), Json::from(from_node).text()));
        }
        let target = &mut nodes.iter_mut().find(|(k, _)| *k == to_node).expect("checked").1;
        let literal = target.get("inputs").and_then(|i| i.get(&to_port)).cloned();
        if is_wire(literal.as_ref()) {
            return Err(format!("two wire() go into {to_node}:{to_port} (a port takes one wire)"));
        }
        let mut bound = Json::obj().with("wire", Json::obj().with("node", from_node).with("port", from_port));
        if let Some(v) = literal.filter(|v| *v != Json::Null) {
            bound.set("value", v);
        }
        if let Json::Obj(fields) = target {
            if let Some((_, inputs)) = fields.iter_mut().find(|(k, _)| k == "inputs") {
                inputs.set(&to_port, bound);
            }
        }
        // Where the wire was written (`meta.sources`, by `wire:<port>`): the editor finds the element again to remove it.
        if let Some(s) = &w.source {
            let mut meta = target.get("meta").cloned().unwrap_or_else(Json::obj);
            let mut sources = match meta.get("sources") {
                Some(o @ Json::Obj(_)) => o.clone(),
                _ => Json::obj(),
            };
            sources.set(&format!("wire:{to_port}"), s.clone());
            meta.set("sources", sources);
            target.set("meta", meta);
        }
    }
    let mut meta = if is_record(t.attr("meta")) { t.attr("meta").cloned().unwrap_or_else(Json::obj) } else { Json::obj() };
    if let Some(name) = t.attr("name") {
        meta.set("name", name.clone());
    }
    meta.set("domain", t.attr("domain").filter(|d| **d != Json::Null).cloned().unwrap_or_else(|| NODE_GRAPH_ID.into()));
    let id = match t.attr("id") {
        Some(Json::Str(s)) => s.clone(),
        _ => NODE_GRAPH_ID.to_string(),
    };
    Ok(Json::obj().with("id", id).with("nodes", Json::Obj(nodes)).with("outputs", outputs).with("meta", meta))
}
