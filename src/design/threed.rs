//! A 3D DOCUMENT IN RUST — the document's own graph, declared element by element, so the 3D editor opens a `.3d.rs`
//! with all its tools and writes each edit back into the file. The same elements, node for node, as the TypeScript
//! SDK's JSX (`threed.ts`) and the Python SDK's (`threed.py`):
//!
//! ```
//! use commandagi::design::threed::*;
//!
//! fn document() -> El {
//!     part([
//!         parameter().name("depth").value(6).unit("mm").bindings(json(r#"[{"target": "extrude1", "field": "distance"}]"#)),
//!         sketch([
//!             point().id("p1").x(-30).y(-20),
//!             point().id("p2").x(30).y(-20),
//!             line().id("l1").a("p1").b("p2"),
//!             constraint().id("c1").kind("horizontal").entities(["l1"]),
//!         ])
//!         .id("sketch1")
//!         .name("Sketch1")
//!         .plane(json(r#"{"type": "datum", "plane": "plane_xy"}"#)),
//!         extrude().id("extrude1").name("Extrude1").profile(json(r#"{"sketch": "sketch1"}"#)).distance(6).operation("new"),
//!         body().id("extrude1").material("aluminium-6061"),
//!     ])
//!     .name("Mounting plate")
//! }
//! # commandagi::design::declare(document()).unwrap();
//! ```
//!
//! The calls (each method is the field of the same name, in snake case: `.x_axis(…)` is `xAxis`):
//!
//! ```text
//! part([…]).name(…) | assembly([…]).name(…)     the document; an assembly opens in the assembly mode
//! part([…]).builtin_planes([…])                 the built-in planes the document has (absent: all three; []: none)
//! parameter().name().value().unit().comment().min().max().step().bindings(json(r#"[{"target", "field"}]"#))
//! plane().id().name().origin().normal().x_axis()                      a datum plane
//! FEATURE().id().name() …fields                  a feature: the function is its type (extrude, revolve, fillet,
//!                                                chamfer, hole, linear_pattern, box_, import, …); `.suppressed(…)`,
//!                                                `.consumes([…])` as stored
//! feature().type_(…).id().name() …               a feature of a type the kernel does not know
//! sketch([…]).id().name().plane(…)              a sketch: point().id().x().y(), a segment by its type (line,
//!                                                circle, arc, spline, ellipse, ellipse_arc), constraint().id()
//!                                                .kind().entities([…]), projection().id().ref_().feature()
//! body().id().name().material(…).visible(…)     a body's own data (`bodyMeta[id]`)
//! slot().name(…).value(…)                       any other field of the document, whole
//! ```
//!
//! The declaration is the document's graph exactly: a feature is a node of its type with its fields as ports, a
//! plane a `plane` node, a parameter an `input` node whose `drives` are its bindings, a slot a `3d.<field>` node, the
//! bodies one `3d.bodyMeta` node. The order of the feature elements is the order of the left-hand list
//! (`presentation.order`). Every node carries the call it came from in `meta.source`; a sketch's entities and the
//! bodies are listed by key in `meta.sources`. Units are millimetres and radians, as stored. A field the methods do not
//! name is written `.with_attr("field", value)`. Anything else is refused by name, never guessed.

use super::element::{attributes, elements, fn_name, rust_name};
use super::ir::slug;
use super::source::Sources;
use super::{Declared, Family};
pub use super::{fragment, json, El, Json, Value};

elements! {
    /// A part: the document, its elements.
    part: children;
    /// An assembly: the document, opened in the assembly mode.
    assembly: children;
    /// A parameter: `parameter().name("depth").value(6).unit("mm")`.
    parameter: leaf;
    /// A datum plane: `plane().id(…).origin([0, 0, 0]).normal([0, 0, 1]).x_axis([1, 0, 0])`.
    plane: leaf;
    /// A body's own data: `body().id("extrude1").material(…)`.
    body: leaf;
    /// Any other field of the document, whole: `slot().name("environment").value(json(…))`.
    slot: leaf;
    /// A feature of a type the kernel does not know: `feature().type_("rotate").id(…)`.
    feature: leaf;
    /// A sketch: its points, segments, constraints and projections.
    sketch: children;
    extrude: leaf;
    revolve: leaf;
    sweep: leaf;
    loft: leaf;
    fillet: leaf;
    chamfer: leaf;
    shell: leaf;
    box_: leaf;
    import: leaf;
    external_part = "externalPart": leaf;
    cylinder: leaf;
    sphere: leaf;
    cone: leaf;
    makehuman: leaf;
    linear_pattern = "linearPattern": leaf;
    circular_pattern = "circularPattern": leaf;
    path_pattern = "pathPattern": leaf;
    mirror: leaf;
    datum_plane = "datumPlane": leaf;
    draft: leaf;
    hole: leaf;
    thread: leaf;
    rib: leaf;
    coil: leaf;
    combine: leaf;
    offset_faces = "offsetFaces": leaf;
    thicken: leaf;
    split: leaf;
    scale: leaf;
    dome: leaf;
    wrap_text = "wrapText": leaf;
    delete_face = "deleteFace": leaf;
    full_round = "fullRound": leaf;
    sheet_flange = "sheetFlange": leaf;
    unfold: leaf;
    copy_body = "copyBody": leaf;
    transform: leaf;
    subdiv_body = "subdivBody": leaf;
    mesh_body = "meshBody": leaf;
    mesh_modifier = "meshModifier": leaf;
    point_cloud = "pointCloud": leaf;
    gaussian_splat = "gaussianSplat": leaf;
    skeleton: leaf;
    volume: leaf;
    molecule: leaf;
    graph: leaf;
    generate: leaf;
    pcb_trace = "pcbTrace": leaf;
    copper_pour = "copperPour": leaf;
    generate_via = "generateVia": leaf;
    plated_hole = "platedHole": leaf;
    /// A part authored as code: `code().id(…).source("bracket.3d.py").inputs(json(…))`.
    code: leaf;
    /// A sketch point: `point().id("p1").x(0).y(0)`.
    point: leaf;
    /// A line between two points: `line().id("l1").a("p1").b("p2")`.
    line: leaf;
    /// A circle: `circle().id(…).center("p1").radius(3)`.
    circle: leaf;
    /// An arc: `arc().id(…).center(…).start(…).end(…).radius(…)`.
    arc: leaf;
    /// A spline through (or controlled by) its points.
    spline: leaf;
    /// An ellipse: `.center(…).focus(…).minor_radius(…)`.
    ellipse: leaf;
    /// An ellipse arc: `.center(…).focus(…).minor_radius(…).start(…).end(…)`.
    ellipse_arc = "ellipseArc": leaf;
    /// A constraint: `constraint().id("c1").kind("horizontal").entities(["l1"])`.
    constraint: leaf;
    /// A projection of a feature's edge into the sketch.
    projection: leaf;
}

attributes! {
    /// The fields of a 3D document's entities, chained on its elements (each the field of the same name).
    pub trait ThreedAttrs {
        a; african; age; align_to_path; anchor; angle; animation; annotations; area; armatures; asian; assembly;
        at; axis; axis_b; b; base; bend_radius; bias; bindings; body_id; bond_orders; bonds; both_sides;
        break_after; builtin; builtin_planes; cap; caucasian; cell; center; centreline; chains; closed; collapsed;
        colormap; colors; comment; component; conductor; construction; consumes; control_points; count; count2;
        cupsize; data; decals; depth; diameter; dims; dir; directed; direction; direction2; distance; distance2;
        draft_angle; drill_mm; dynamics; edge; edge_weights; edges; electrical; electron; elements; end; entities;
        environment; fabrication; face; faces; factor; feature; file_id; file_name; firmness; fixed; focus;
        footprint; format; from; gender; graph; groups; head_depth; head_diameter; height; id; importer; indices;
        input; inputs; internal; iso; join; k_factor; keep_original; kind; knots; label; layer; layers; length;
        level; lib_id; local_placement; material; material_groups; mates; max; members; min; minor_radius; mode;
        model_id; modifier; muscle; name; node_colors; node_groups; node_positions; node_sizes; node_types; normal;
        normals; nucleus; nurbs; offset; operation; optimization; order; origin; outward; pad; pad_diameter_mm;
        pads; paint; params; parents; part; path; pin; pitch; placement; planar; plane; point; point_size; points;
        polygons; position; positions; profile; profiles; proportions; provenance; pull_direction; quaternion;
        radii; radius; radius1; radius2; ref_; reference; refs; regions; render; representation; reverse;
        right_handed; rings; rotate_angle; rotate_axis; rotation_deg; rotations; rule; ruled; scale; scale_x_y_z;
        scales; scene_constraints; seed_feature; semantic_visibility; sh; sh_degree; side; signature; size; sketch;
        skins; source; source_materials; source_uv; spacing; spacing2; standard_parts; start; step; style;
        subdivision; suppressed; symmetric; target; terminal; thickness; through; token; transform; translate;
        turns; type_; u; unit; unit_scale_mm; up_to_face; uv; v; value; visible; weight; width; x; x_axis; y;
    }
}

pub(crate) const FAMILY: Family = Family { module: "threed", roots: &["part", "assembly"], declare };

/// The feature types a 3D document holds, by tag (`packages/domain/3d-core/types.ts` Feature).
pub const THREED_FEATURES: &[&str] = &[
    "sketch", "extrude", "revolve", "sweep", "loft", "fillet", "chamfer", "shell", "box", "import", "externalPart",
    "cylinder", "sphere", "cone", "makehuman", "linearPattern", "circularPattern", "pathPattern", "mirror",
    "datumPlane", "draft", "hole", "thread", "rib", "coil", "combine", "offsetFaces", "thicken", "split", "scale",
    "dome", "wrapText", "deleteFace", "fullRound", "sheetFlange", "unfold", "copyBody", "transform", "subdivBody",
    "meshBody", "meshModifier", "pointCloud", "gaussianSplat", "skeleton", "volume", "molecule", "graph", "generate",
    "pcbTrace", "copperPour", "generateVia", "platedHole", "code",
];
/// Tags with a meaning of their own: never a `feature().type_(…)`.
const NOT_FEATURES: &[&str] = &[
    "feature", "part", "assembly", "parameter", "plane", "body", "slot", "point", "constraint", "projection", "line", "circle", "arc",
    "spline", "ellipse", "ellipseArc", "input",
];
/// A sketch's segment kinds, by tag.
pub const SKETCH_SEGMENTS: &[&str] = &["line", "circle", "arc", "spline", "ellipse", "ellipseArc"];
/// The datum planes every 3D document has (`3d-core/document.ts` basePlanes): id, name, origin, normal, xAxis, builtin.
pub const BUILTIN_PLANES: &[(&str, &str, [f64; 3], [f64; 3], [f64; 3], &str)] = &[
    ("plane_xy", "Front (XY)", [0.0, 0.0, 0.0], [0.0, 0.0, 1.0], [1.0, 0.0, 0.0], "XY"),
    ("plane_xz", "Top (XZ)", [0.0, 0.0, 0.0], [0.0, 1.0, 0.0], [1.0, 0.0, 0.0], "XZ"),
    ("plane_yz", "Right (YZ)", [0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], "YZ"),
];
/// The document's view fields a root element may hold (the graph's `meta`), beside `name`.
const VIEW_ATTRS: &[&str] = &["semanticVisibility"];
const PARAM_FIELDS: &[&str] = &["value", "unit", "comment", "min", "max", "step"];
/// Document fields a slot may not hold: they have elements of their own, or are the graph itself.
const NOT_SLOTS: &[&str] = &["id", "name", "units", "planes", "features", "parameters", "presentation", "isAssembly", "bodyMeta", "semanticVisibility"];
/// Slots whose value is an object: each field is a port of the slot's node; any other slot holds its value whole.
const OBJECT_SLOTS: &[&str] = &["environment", "assembly", "sceneConstraints", "dynamics", "animation", "optimization", "standardParts"];

/// Whether a root is a 3D document: one `part` or `assembly`.
pub fn is_threed(root: &El) -> bool {
    root.tag == "part" || root.tag == "assembly"
}

/// How a message names an element: `sketch().id("sketch1")`, `slot().name("environment")`, `box_()`.
fn at(el: &El) -> String {
    for k in ["id", "name"] {
        if let Some(v @ Json::Str(_)) = el.attr(k) {
            return format!("{}().{k}({})", fn_name(el.tag), v.text());
        }
    }
    format!("{}()", fn_name(el.tag))
}

/// A number as JavaScript writes it in a message.
fn js_number(n: f64) -> String {
    if n.is_nan() {
        "NaN".into()
    } else if n.is_infinite() {
        if n > 0.0 { "Infinity".into() } else { "-Infinity".into() }
    } else {
        Json::Num(n).text()
    }
}

/// A value is plain data with finite numbers.
fn plain(v: &Json, what: &str) -> Result<(), String> {
    match v {
        Json::Num(n) if !n.is_finite() => Err(format!("{what} is {}, not a finite number", js_number(*n))),
        Json::Arr(items) => items.iter().enumerate().try_for_each(|(i, x)| plain(x, &format!("{what}[{i}]"))),
        Json::Obj(entries) => entries.iter().try_for_each(|(k, x)| plain(x, &format!("{what}.{k}"))),
        _ => Ok(()),
    }
}

/// The element's attributes as fields, without the names in `skip`, each plain data.
fn fields(el: &El, skip: &[&str]) -> Result<Vec<(String, Json)>, String> {
    let mut out = Vec::new();
    for (k, v) in &el.attrs {
        if skip.contains(&k.as_str()) {
            continue;
        }
        plain(v, &format!("{} {}", at(el), rust_name(k)))?;
        out.push((k.clone(), v.clone()));
    }
    Ok(out)
}

fn take(fields: &mut Vec<(String, Json)>, name: &str) -> Option<Json> {
    fields.iter().position(|(k, _)| k == name).map(|i| fields.remove(i).1)
}

fn word_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

/// An id: letters, digits, `_ . -`, with `/` between them (a merged code part's features are `<code id>/<id>`).
fn id_ok(id: &str) -> bool {
    id.split('/').all(|part| !part.is_empty() && part.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '-')))
}

fn id_of(el: &El) -> Result<String, String> {
    match el.attr("id") {
        Some(Json::Str(id)) if !id.is_empty() => {
            if !id_ok(id) {
                return Err(format!("{}: an id is letters, digits, _ . - with / between them ({})", at(el), Json::from(id.as_str()).text()));
            }
            Ok(id.clone())
        }
        _ => Err(format!("{} needs an id", at(el))),
    }
}

fn no_children(el: &El) -> Result<(), String> {
    if el.child_elements().next().is_some() {
        return Err(format!("{} has no child elements", at(el)));
    }
    Ok(())
}

fn source_meta(el: &El) -> Option<Json> {
    el.source_json().map(|s| Json::obj().with("source", s))
}

fn keys(entries: &[(String, Json)]) -> Json {
    Json::Arr(entries.iter().map(|(k, _)| Json::from(k.as_str())).collect())
}

/// The sketch a sketch element's children declare, and where each entity was written.
fn sketch_of(el: &El) -> Result<(Json, Sources), String> {
    let (mut points, mut segments, mut constraints, mut projections) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    let mut sources = Sources::new();
    let mut taken: Vec<String> = Vec::new();
    for child in el.child_elements() {
        let mut claim = |kind: &str| -> Result<String, String> {
            let id = id_of(child)?;
            if taken.contains(&id) {
                return Err(format!("{}: two entities are called {id}", at(el)));
            }
            taken.push(id.clone());
            sources.put(format!("{kind}:{id}"), child);
            no_children(child)?;
            Ok(id)
        };
        if child.tag == "point" {
            let id = claim("point")?;
            let f = fields(child, &[])?;
            let num = |k: &str| matches!(f.iter().find(|(n, _)| n == k), Some((_, Json::Num(_))));
            if !num("x") || !num("y") {
                return Err(format!("{} {}: a point has x and y (mm)", at(el), at(child)));
            }
            points.push((id, Json::Obj(f)));
        } else if SKETCH_SEGMENTS.contains(&child.tag) {
            let id = claim("segment")?;
            if child.attr("type").is_some() {
                return Err(format!("{} {}: the tag is the segment's type", at(el), at(child)));
            }
            let mut s = vec![("id".to_string(), Json::from(id.as_str())), ("type".to_string(), Json::from(child.tag))];
            s.extend(fields(child, &["id"])?);
            segments.push((id, Json::Obj(s)));
        } else if child.tag == "constraint" {
            let id = claim("constraint")?;
            let f = fields(child, &[])?;
            let get = |k: &str| f.iter().find(|(n, _)| n == k).map(|(_, v)| v);
            if !matches!(get("kind"), Some(Json::Str(_))) || !matches!(get("entities"), Some(Json::Arr(_))) {
                return Err(format!("{} {}: a constraint has a kind and its entities", at(el), at(child)));
            }
            constraints.push((id, Json::Obj(f)));
        } else if child.tag == "projection" {
            let id = claim("projection")?;
            projections.push((id, Json::Obj(fields(child, &[])?)));
        } else {
            let segs: Vec<String> = SKETCH_SEGMENTS.iter().map(|s| fn_name(s)).collect();
            return Err(format!("{}: {}() is not read in a sketch (point, {}, constraint, projection)", at(el), fn_name(child.tag), segs.join(", ")));
        }
    }
    for (id, s) in &segments {
        for k in ["a", "b", "center", "start", "end", "focus"] {
            if let Some(p) = s.get(k) {
                if !p.as_str().is_some_and(|p| points.iter().any(|(q, _)| q == p)) {
                    return Err(format!("{}: segment {id}'s {k} names no point of the sketch ({})", at(el), p.text()));
                }
            }
        }
    }
    let mut sketch = Json::obj()
        .with("points", Json::Obj(points.clone()))
        .with("segments", Json::Obj(segments.clone()))
        .with("constraints", Json::Obj(constraints.clone()))
        .with("pointOrder", keys(&points))
        .with("segmentOrder", keys(&segments))
        .with("constraintOrder", keys(&constraints));
    if !projections.is_empty() {
        sketch.set("projectionOrder", keys(&projections));
        // `projections` stands before `projectionOrder`, as TypeScript sets them.
        if let Json::Obj(entries) = &mut sketch {
            let at = entries.len() - 1;
            entries.insert(at, ("projections".into(), Json::Obj(projections)));
        }
    }
    Ok((sketch, sources))
}

/// The built-in planes a root says the document has (`builtin_planes`); absent: all three.
fn builtin_planes_of(root: &El) -> Result<Vec<&'static str>, String> {
    let all: Vec<&'static str> = BUILTIN_PLANES.iter().map(|p| p.0).collect();
    let Some(v) = root.attr("builtinPlanes") else { return Ok(all) };
    let refused = || format!("{}: builtin_planes lists built-in planes ({})", at(root), all.join(", "));
    let Json::Arr(items) = v else { return Err(refused()) };
    let mut out = Vec::new();
    for x in items {
        let p = x.as_str().and_then(|s| all.iter().find(|a| **a == s).copied()).ok_or_else(refused)?;
        if out.contains(&p) {
            return Err(format!("{}: builtin_planes names a plane twice", at(root)));
        }
        out.push(p);
    }
    Ok(out)
}

fn declare(root: El) -> Result<Declared, String> {
    declare_threed(&root, "Part").map(Declared::Graph)
}

/// The graph a part or assembly element declares: an op graph whose nodes are its parameters, planes, features and
/// slots (docs/formats.md § the 3D document's body).
pub fn declare_threed(root: &El, fallback_name: &str) -> Result<Json, String> {
    if !is_threed(root) {
        return Err("a 3D document is one part() or assembly() element".into());
    }
    for (k, _) in &root.attrs {
        if !["name", "id", "builtinPlanes"].contains(&k.as_str()) && !VIEW_ATTRS.contains(&k.as_str()) {
            return Err(format!("{}: {} is not read on a 3D document", at(root), rust_name(k)));
        }
    }
    let builtins = builtin_planes_of(root)?;
    let name = match root.attr("name") {
        Some(Json::Str(n)) if !n.is_empty() => n.clone(),
        _ => fallback_name.to_string(),
    };
    let mut nodes: Vec<(String, Json)> = Vec::new();
    let mut owner: Vec<(String, String)> = Vec::new();
    fn claim(owner: &mut Vec<(String, String)>, id: &str, what: String) -> Result<(), String> {
        if let Some((_, prev)) = owner.iter().find(|(o, _)| o == id) {
            return Err(format!("the id \"{id}\" is both {prev} and {what}; a node id is used once"));
        }
        owner.push((id.to_string(), what));
        Ok(())
    }
    let node = |id: &str, kind: &str, label: Option<&str>, inputs: Json, meta: Option<Json>| {
        let mut n = Json::obj().with("id", id).with("type", kind);
        if let Some(l) = label {
            n.set("label", l);
        }
        n.set("inputs", inputs);
        if let Some(m) = meta {
            n.set("meta", m);
        }
        n
    };
    let mut order: Vec<Json> = Vec::new();
    let mut bodies: Vec<(String, Json)> = Vec::new();
    let mut body_sources = Sources::new();
    let mut declared: Vec<&El> = Vec::new();
    for el in root.child_elements() {
        match el.tag {
            "parameter" => {
                no_children(el)?;
                let pname = match el.attr("name") {
                    Some(Json::Str(p)) if !p.is_empty() && slug(p) == *p => p.clone(),
                    _ => return Err(format!("{} needs a name (letters, digits, _ . -)", at(el))),
                };
                for (k, _) in &el.attrs {
                    if !["name", "bindings"].contains(&k.as_str()) && !PARAM_FIELDS.contains(&k.as_str()) {
                        return Err(format!("{}: {} is not read on a parameter", at(el), rust_name(k)));
                    }
                }
                if !matches!(el.attr("value"), Some(Json::Num(_))) {
                    return Err(format!("{}: value is a number", at(el)));
                }
                claim(&mut owner, &pname, format!("parameter \"{pname}\""))?;
                let mut inputs = fields(el, &["name", "bindings"])?;
                let bindings = match el.attr("bindings") {
                    None => Json::Arr(Vec::new()),
                    Some(b) => {
                        plain(b, &format!("{} bindings", at(el)))?;
                        b.clone()
                    }
                };
                let ok = match &bindings {
                    Json::Arr(list) => list.iter().all(|b| matches!(b.get("target"), Some(Json::Str(_))) && matches!(b.get("field"), Some(Json::Str(_)))),
                    _ => false,
                };
                if !ok {
                    return Err(format!("{}: bindings are [{{ target, field }}]", at(el)));
                }
                if matches!(&bindings, Json::Arr(l) if !l.is_empty()) {
                    inputs.retain(|(k, _)| k != "drives");
                    inputs.push(("drives".into(), bindings));
                }
                nodes.push((pname.clone(), node(&pname, "input", Some(&pname), Json::Obj(inputs), source_meta(el))));
            }
            "plane" => {
                no_children(el)?;
                let id = id_of(el)?;
                claim(&mut owner, &id, format!("plane \"{id}\""))?;
                let mut rest = fields(el, &["id"])?;
                let label = take(&mut rest, "name");
                nodes.push((id.clone(), node(&id, "plane", label.as_ref().and_then(Json::as_str), Json::Obj(rest), source_meta(el))));
            }
            "body" => {
                no_children(el)?;
                let id = id_of(el)?;
                if bodies.iter().any(|(b, _)| *b == id) {
                    return Err(format!("two body() elements are called {id}"));
                }
                bodies.push((id.clone(), Json::Obj(fields(el, &["id"])?)));
                body_sources.put(id, el);
            }
            "slot" => {
                no_children(el)?;
                let field = match el.attr("name") {
                    Some(Json::Str(f)) if f.chars().next().is_some_and(|c| c.is_ascii_alphabetic() || c == '_') && f.chars().all(word_char) => f.clone(),
                    _ => return Err(format!("{} needs the name of a document field", at(el))),
                };
                if NOT_SLOTS.contains(&field.as_str()) {
                    return Err(format!("{}: {field} is not a slot (it has its own element or attribute)", at(el)));
                }
                for (k, _) in &el.attrs {
                    if k != "name" && k != "value" {
                        return Err(format!("{}: {} is not read on a slot (its value is value)", at(el), rust_name(k)));
                    }
                }
                claim(&mut owner, &field, format!("the document's {field}"))?;
                // An absent value is `undefined` in TypeScript, which the node's JSON leaves out.
                let value = el.attr("value").cloned();
                if let Some(v) = &value {
                    plain(v, &format!("{} value", at(el)))?;
                }
                let inputs = if OBJECT_SLOTS.contains(&field.as_str()) {
                    match &value {
                        Some(v @ Json::Obj(_)) => v.clone(),
                        _ => return Err(format!("{}: the document's {field} is an object", at(el))),
                    }
                } else {
                    match value {
                        Some(v) => Json::obj().with("value", v),
                        None => Json::obj(),
                    }
                };
                nodes.push((field.clone(), node(&field, &format!("3d.{field}"), None, inputs, source_meta(el))));
            }
            t if THREED_FEATURES.contains(&t) => declared.push(el),
            "feature" => {
                let t = match el.attr("type") {
                    Some(Json::Str(t))
                        if t.chars().next().is_some_and(|c| c.is_ascii_alphabetic() || c == '_') && t.chars().all(|c| word_char(c) || c == '.' || c == '-') =>
                    {
                        t.clone()
                    }
                    _ => return Err(format!("{} needs the feature's type", at(el))),
                };
                if THREED_FEATURES.contains(&t.as_str()) || NOT_FEATURES.contains(&t.as_str()) {
                    return Err(format!("{}: a {t} is written {}(…)", at(el), fn_name(&t)));
                }
                declared.push(el);
            }
            _ => return Err(format!("{}() is not read in a 3D document (see commandagi::design::threed)", fn_name(el.tag))),
        }
    }
    for el in declared {
        let id = id_of(el)?;
        claim(&mut owner, &id, format!("feature \"{id}\""))?;
        let generic = el.tag == "feature";
        if !generic && el.attr("type").is_some() {
            return Err(format!("{}: the tag is the feature's type", at(el)));
        }
        let kind = if generic { el.attr("type").and_then(Json::as_str).unwrap_or_default().to_string() } else { el.tag.to_string() };
        let mut rest = fields(el, if generic { &["id", "type"] } else { &["id"] })?;
        let label = take(&mut rest, "name");
        let suppressed = take(&mut rest, "suppressed");
        if suppressed.as_ref().is_some_and(|s| !matches!(s, Json::Bool(_))) {
            return Err(format!("{}: suppressed is true or false", at(el)));
        }
        let mut inputs = rest;
        let mut sources = Sources::new();
        if el.tag == "sketch" {
            if inputs.iter().any(|(k, _)| k == "sketch") {
                return Err(format!("{}: a sketch's points, segments and constraints are its child elements", at(el)));
            }
            let (sketch, s) = sketch_of(el)?;
            inputs.push(("sketch".into(), sketch));
            sources = s;
        } else {
            no_children(el)?;
            if el.tag == "code" {
                // A code feature's `inputs` are ports of its node, beside `source` and `consumes` (the document graph's rule).
                let code_inputs = take(&mut inputs, "inputs");
                if code_inputs.as_ref().is_some_and(|c| !matches!(c, Json::Obj(_))) {
                    return Err(format!("{}: inputs is an object", at(el)));
                }
                if let Some((k, _)) = inputs.iter().find(|(k, _)| k != "source" && k != "consumes") {
                    return Err(format!("{}: a code feature has source, inputs and consumes, not {}", at(el), rust_name(k)));
                }
                if let Some(Json::Obj(entries)) = code_inputs {
                    for (k, v) in entries {
                        inputs.retain(|(n, _)| *n != k);
                        inputs.push((k, v));
                    }
                }
            }
        }
        let mut n = Json::obj().with("id", id.as_str()).with("type", kind.as_str());
        n.set("label", match &label {
            Some(Json::Str(l)) => l.as_str(),
            _ => id.as_str(),
        });
        if let Some(s) = suppressed {
            n.set("disabled", s);
        }
        n.set("inputs", Json::Obj(inputs));
        let mut meta = Json::obj();
        if let Some(s) = el.source_json() {
            meta.set("source", s);
        }
        if !sources.is_empty() {
            meta.set("sources", sources.json());
        }
        if meta != Json::obj() {
            n.set("meta", meta);
        }
        nodes.push((id.clone(), n));
        order.push(Json::from(id.as_str()));
    }
    for (id, label, origin, normal, x_axis, builtin) in BUILTIN_PLANES {
        if owner.iter().any(|(o, _)| o == id) || !builtins.contains(id) {
            continue;
        }
        let v3 = |v: &[f64; 3]| Json::Arr(v.iter().map(|n| Json::Num(*n)).collect());
        let inputs = Json::obj().with("origin", v3(origin)).with("normal", v3(normal)).with("xAxis", v3(x_axis)).with("builtin", *builtin);
        nodes.push((id.to_string(), node(id, "plane", Some(label), inputs, None)));
    }
    if !bodies.is_empty() {
        claim(&mut owner, "bodyMeta", "the bodies (body())".into())?;
        let meta = (!body_sources.is_empty()).then(|| Json::obj().with("sources", body_sources.json()));
        nodes.push(("bodyMeta".into(), node("bodyMeta", "3d.bodyMeta", None, Json::Obj(bodies), meta)));
    }
    let mut view = Json::obj().with("name", name.as_str()).with("units", "mm");
    for k in VIEW_ATTRS {
        if let Some(v) = root.attr(k) {
            plain(v, &format!("{} {}", at(root), rust_name(k)))?;
            view.set(k, v.clone());
        }
    }
    if root.tag == "assembly" {
        view.set("isAssembly", true);
    }
    if !order.is_empty() {
        view.set("presentation", Json::obj().with("order", order));
    }
    let id = match root.attr("id") {
        Some(Json::Str(i)) if !i.is_empty() => i.clone(),
        _ => format!("3d-{}", slug(&name).to_lowercase()),
    };
    Ok(Json::obj().with("id", id.as_str()).with("nodes", Json::Obj(nodes)).with("meta", view))
}
