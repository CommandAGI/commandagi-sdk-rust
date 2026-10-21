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
//!         body().id("extrude1").engineering(json(r#"{"material": "6061-T6", "process": "cnc-milling"}"#)),
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
//!                                                `.consumes([…])` as stored. THE ORDER OF THE ELEMENTS IS THE ORDER
//!                                                THE PART IS BUILT IN: a feature that acts on bodies (a cut, a hole,
//!                                                a fillet, a pattern) and names no `consumes` follows every
//!                                                body-making feature written before it
//! faces and edges                                where a feature takes them (`.edges()`, `.faces()`, `.face()`,
//!                                                `.edge()`): one picked by where it was, `{"kind", "signature":
//!                                                {"point", …}}`, or the ones a query chooses when the part is built,
//!                                                `{"kind", "select": {…}}`: normal, parallel, perpendicular, axis,
//!                                                geometry, length, radius, area, ofFace, createdBy, at, inBox, farthest,
//!                                                any, not. Every test present must hold
//! hole().face(…).positions(…) | .sketch(…)       where the holes are: points in model coordinates, or a sketch of
//!                                                points; with neither, the face's centre. A face that chooses
//!                                                several drills each place in the one it lies on
//! feature().type_(…).id().name() …               a feature of a type the kernel does not know
//! sketch([…]).id().name().plane(…)              a sketch: point().id().x().y(), a segment by its type (line,
//!                                                circle, arc, spline, ellipse, ellipse_arc), constraint().id()
//!                                                .kind().entities([…]), projection().id().ref_().feature()
//! body().id().name().material(…).engineering(…) a body's own data (`bodyMeta[id]`): its name, its look (`material`)
//!                                                and what it is (`engineering`: the material grade by its id in a
//!                                                materials table, the process and how it is made, the finish, its
//!                                                suppliers and their terms, its inspection, compliance, shipping and
//!                                                lifecycle; docs/formats.md § logistics)
//! item().id().name().qty().ref_().engineering(…) a part of the product with no geometry here (`items`): a cable in
//!                                                the box, or another design by `ref` (a board's .pcb.rs)
//! assemble().id().order().parts([…]).fasteners([…]).tools([…]).fixture().by().minutes().rate_per_hour()
//!          .currency().instruction()             one assembly step (`assemblySteps`)
//! joint().id().kind().a().b().anchor().axis().limits() …   a joint between two bodies (`dynamics.joints`): fixed,
//!                                                revolute, prismatic, cylindrical, planar, screw, spherical, universal
//! mate().id().kind().a().b().axis() …            a mate between two components of an assembly (`assembly.mates`)
//! sheet().id().size().scale().projection().views(…) …      an engineering drawing sheet of the document (`drawings`)
//! slot().name(…).value(…)                       any other field of the document, whole
//! ```
//!
//! The declaration is the document's graph exactly: a feature is a node of its type with its fields as ports, a
//! plane a `plane` node, a parameter an `input` node whose `drives` are its bindings, a slot a `3d.<field>` node, the
//! bodies one `3d.bodyMeta` node. The order of the feature elements is the order of the left-hand list
//! (`presentation.order`). Every node carries the call it came from in `meta.source`; a sketch's entities and the
//! bodies are listed by key in `meta.sources`. Units are millimetres and radians, as stored. A field the methods do not
//! name is written `.with_attr("field", value)`. Anything else is refused by name, never guessed.
//!
//! A feature element is checked against its type's schema (`threed_schema.json`, generated from the `Feature` types;
//! the same fields are typed in [`super::threed_props`], `ExtrudeProps` …): the feature `{type: <tag>, id, name: <its
//! id when absent>, suppressed, …its fields}`, a sketch with the `sketch` its children make. A wrong field is refused
//! with the sentence the TypeScript reader and the op panel give: `extrude().id("e1"): distance: must be greater than 0
//! (got -1)`. A `feature().type_(…)` of an unknown type is not checked. A required field that declares a default may
//! be left out: it reads as that default (an `external_part()` without `placement` sits at the identity placement),
//! and the node's inputs hold it.

use super::element::{attributes, elements, fn_name, rust_name};
use super::ir::slug;
use super::schema::{describe_issue, validate, with_defaults};
use super::source::Sources;
use super::standards::sized_hole;
use super::{Declared, Family};
pub use super::threed_props::THREED_FEATURES;
pub use super::{fragment, json, El, Json, Value};
use std::sync::OnceLock;

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
    /// A joint between two bodies: `joint().id("j1").kind("revolute").a("base").b("arm").anchor([0, 0, 40]).axis([0, 0, 1])`.
    joint: leaf;
    /// A mate between two components of an assembly: `mate().id("m1").kind("concentric").a("p1").b("p2").axis("Z")`.
    mate: leaf;
    /// An engineering drawing sheet: `sheet().id("sheet1").views(json(r#"[{"id": "front", "view": "front"}]"#))`.
    sheet: leaf;
    /// A part of the product with no geometry here: `item().id("tape").name("Foam tape").qty(2).engineering(…)`.
    item: leaf;
    /// One assembly step: `assemble().id("s1").order(1).parts(["b", "tape"]).by("in-house").minutes(2)`.
    assemble: leaf;
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
    make_human = "makeHuman": leaf;
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
        a; actuator; african; age; align_to_path; anchor; angle; animation; annotations; area; armatures; asian; assembly;
        at; axis; axis_b; b; base; bend_radius; bias; bindings; bodies; body_id; bond_orders; bonds; both_sides;
        break_after; builtin; builtin_planes; by; cap; caucasian; cell; center; centreline; chains; closed; collapsed;
        colormap; colors; comment; component; conductor; construction; consumes; control_points; count; count2;
        cupsize; currency; data; decals; depth; diameter; dims; dir; directed; direction; direction2; distance; distance2;
        draft_angle; drill_mm; dynamics; edge; edge_weights; edges; electrical; electron; elements; end; engineering; entities;
        environment; fabrication; face; faces; factor; fasteners; feature; file_id; file_name; firmness; fixed; fixture; focus;
        footprint; format; from; gender; graph; groups; head_depth; head_diameter; height; hidden_lines; hole_callouts; id; importer; indices;
        input; inputs; instruction; internal; iso; join; k_factor; keep_original; kind; knots; label; layer; layers; length;
        level; lib_id; limits; local_placement; material; material_groups; mates; max; members; min; minor_radius; minutes; mode;
        model_id; modifier; muscle; name; node_colors; node_groups; node_positions; node_sizes; node_types; normal;
        normals; nucleus; nurbs; offset; operation; optimization; order; origin; outward; overall_dimensions; pad; pad_diameter_mm;
        pads; paint; params; parents; part; parts; path; pin; pitch; placement; planar; plane; point; point_size; points;
        polygons; position; positions; profile; profiles; projection; proportions; provenance; pull_direction; qty; quaternion;
        radii; radius; radius1; radius2; rate_per_hour; ref_; ref_a; ref_b; reference; refs; regions; render; representation; reverse;
        right_handed; rings; rotate_angle; rotate_axis; rotation_deg; rotations; rule; ruled; scale; scale_x_y_z;
        scales; scene_constraints; seed_feature; semantic_visibility; sh; sh_degree; side; signature; size; sketch;
        skins; source; source_materials; source_uv; spacing; spacing2; standard_parts; start; step; style;
        subdivision; suppressed; symmetric; target; terminal; thickness; through; title; token; tools; transform; translate;
        travel; turns; type_; u; unit; unit_scale_mm; up_to_face; uv; v; value; views; visible; weight; width; x; x_axis; y;
        /// A size's tolerance: `json(r#"{"class": "H7"}"#)`, `{"upper": 0.05, "lower": -0.05}` or `{"basic": true}`.
        tolerance;
        /// A fit check's hole, shaft and intended fit (clearance, transition, interference).
        hole; shaft; intent;
        /// A stack-up check's chain (`["+pocket", "-width"]`), limit (`{"min": 0.05, "max": 0.5}`), method, samples and seed.
        chain; limit; method; samples; seed;
    }
}

pub(crate) const FAMILY: Family = Family { module: "threed", roots: &["part", "assembly"], declare };

/// The schema of the 3D features (`roots`: tag → definition, `$defs`), generated from the `Feature` types.
pub fn threed_schema() -> &'static Json {
    static SCHEMA: OnceLock<Json> = OnceLock::new();
    SCHEMA.get_or_init(|| Json::parse(include_str!("threed_schema.json")).expect("threed_schema.json is JSON"))
}

/// The feature with each required field it leaves out filled with its declared default ([`with_defaults`]), checked
/// against its type's schema; refused with every issue when it does not fit.
fn check_feature(el: &El, feature: Json) -> Result<Json, String> {
    let schema = threed_schema();
    let Some(def) = feature.get("type").and_then(|t| schema.get("roots")?.get(t.as_str()?)).and_then(Json::as_str) else { return Ok(feature) };
    let defs = schema.get("$defs").cloned().unwrap_or_else(Json::obj);
    let root = Json::obj().with("$ref", format!("#/$defs/{def}"));
    let filled = with_defaults(&root, &feature, &defs)?;
    let issues = validate(&root, &filled, &defs, "")?;
    if issues.is_empty() {
        return Ok(filled);
    }
    Err(format!("{}: {}", at(el), issues.iter().map(describe_issue).collect::<Vec<_>>().join("; ")))
}
/// A value checked against a definition of the schema (a body's engineering data, a joint, a mate, a sheet), with each
/// required field it leaves out filled with its declared default; refused with every issue when it does not fit.
fn check_value(el: &El, def: &str, value: &Json, path: &str) -> Result<Json, String> {
    let schema = threed_schema();
    let defs = schema.get("$defs").cloned().unwrap_or_else(Json::obj);
    let root = Json::obj().with("$ref", format!("#/$defs/{def}"));
    let filled = with_defaults(&root, value, &defs)?;
    let issues = validate(&root, &filled, &defs, path)?;
    if issues.is_empty() {
        return Ok(filled);
    }
    Err(format!("{}: {}", at(el), issues.iter().map(describe_issue).collect::<Vec<_>>().join("; ")))
}
/// What is wrong with a sheet's section and detail views, one sentence each, as the 3D core's `sheetViewProblems`
/// says it (packages/domain/3d-core/drawing-sheet.ts); the TypeScript and Python SDKs say the same sentences.
fn sheet_view_problems(sheet: &Json) -> Vec<String> {
    let Some(Json::Arr(views)) = sheet.get("views") else { return Vec::new() };
    let text = |v: &Json, k: &str| v.get(k).and_then(Json::as_str).unwrap_or_default().to_string();
    let given = |v: &Json, k: &str| v.get(k).is_some_and(|x| !matches!(x, Json::Null));
    let mut out = Vec::new();
    for v in views {
        let (id, view) = (text(v, "id"), text(v, "view"));
        if view == "section" && !given(v, "section") {
            out.push(format!("view {id} is a section view with no section: give its cutting plane (point and normal)"));
        }
        if view != "detail" {
            continue;
        }
        let Some(detail) = v.get("detail").filter(|_| given(v, "detail")) else {
            out.push(format!("view {id} is a detail view with no detail: give the view it enlarges, its centre and its radius"));
            continue;
        };
        let of = text(detail, "of");
        match views.iter().find(|w| text(w, "id") == of) {
            None => out.push(format!("view {id} is a detail of {of}, which is no view of this sheet")),
            Some(w) if text(w, "view") == "detail" => out.push(format!("view {id} is a detail of {of}, which is a detail itself")),
            Some(_) => {}
        }
    }
    out
}
/// Elements that are the items of a list the document holds: the tag, the document field (its node), the port of that
/// node that holds the list, and the definition each item is checked against (`threed_schema()` `$defs`).
pub const THREED_LISTS: &[(&str, &str, &str, &str)] =
    &[
        ("joint", "dynamics", "joints", "DynamicsJoint"),
        ("mate", "assembly", "mates", "Mate"),
        ("sheet", "drawings", "value", "DrawingSheet"),
        ("check", "checks", "value", "DesignCheck"),
        ("accept", "accepted", "value", "AcceptedFinding"),
        ("permitted", "permitted", "value", "PermittedFinding"),
        ("fastener", "fasteners", "value", "FastenerStack"),
        ("item", "items", "value", "ProductItem"),
        ("assemble", "assemblySteps", "value", "AssemblyStep"),
    ];
/// Tags with a meaning of their own: never a `feature().type_(…)`.
const NOT_FEATURES: &[&str] = &[
    "feature", "part", "assembly", "parameter", "plane", "body", "slot", "joint", "mate", "sheet", "check", "accept", "permitted", "fastener", "item", "assemble", "point", "constraint",
    "projection", "line", "circle", "arc", "spline", "ellipse", "ellipseArc", "input",
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
const PARAM_FIELDS: &[&str] = &["value", "unit", "comment", "min", "max", "step", "tolerance"];
/// Document fields a slot may not hold: they have elements of their own, or are the graph itself.
const NOT_SLOTS: &[&str] = &[
    "id", "name", "units", "planes", "features", "parameters", "presentation", "isAssembly", "bodyMeta", "semanticVisibility", "drawings", "checks",
    "accepted", "permitted", "fasteners", "items", "assemblySteps",
];
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
    declare_threed(&root, &super::source::stem().unwrap_or_else(|| "Part".into())).map(Declared::Graph)
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
    let mut lists: Vec<(usize, Vec<Json>, Sources)> = Vec::new();
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
                if let Some(tol) = el.attr("tolerance") {
                    if let Some(problem) = super::tolerances::tolerance_problem(tol) {
                        return Err(format!("{}: tolerance: {problem}", at(el)));
                    }
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
                let mut body = fields(el, &["id"])?;
                for (k, v) in body.iter_mut() {
                    if k == "engineering" {
                        *v = check_value(el, "PartEngineering", v, "engineering")?;
                    }
                }
                bodies.push((id.clone(), Json::Obj(body)));
                body_sources.put(id, el);
            }
            t if THREED_LISTS.iter().any(|l| l.0 == t) => {
                no_children(el)?;
                let id = id_of(el)?;
                let kind = THREED_LISTS.iter().position(|l| l.0 == t).unwrap_or_default();
                if !lists.iter().any(|l| l.0 == kind) {
                    lists.push((kind, Vec::new(), Sources::new()));
                }
                let Some(held) = lists.iter_mut().find(|l| l.0 == kind) else { continue };
                if held.1.iter().any(|x| x.get("id").and_then(Json::as_str) == Some(id.as_str())) {
                    return Err(format!("two {}() elements are called {id}", fn_name(t)));
                }
                let item = check_value(el, THREED_LISTS[kind].3, &Json::Obj(fields(el, &[])?), "")?;
                if t == "sheet" {
                    let problems = sheet_view_problems(&item);
                    if !problems.is_empty() {
                        return Err(format!("{}: {}", at(el), problems.join("; ")));
                    }
                }
                held.1.push(item);
                held.2.put(format!("{t}:{id}"), el);
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
                        Some(v @ Json::Obj(_)) => {
                            if let Some(l) = THREED_LISTS.iter().find(|l| l.1 == field && v.get(l.2).is_some()) {
                                return Err(format!("{}: {} are written as {}() elements", at(el), l.2, fn_name(l.0)));
                            }
                            v.clone()
                        }
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
        }
        // A hole that names a standard size ("M4 close fit") has that size's diameter and head (standards.rs).
        if el.tag == "hole" && inputs.iter().any(|(k, v)| k == "size" && matches!(v, Json::Str(_))) {
            if let Json::Obj(sized) = sized_hole(&Json::Obj(inputs.clone())).map_err(|e| format!("{}: {e}", at(el)))? {
                inputs = sized;
            }
        }
        if !generic {
            let mut feature = vec![("type".to_string(), Json::from(kind.as_str())), ("id".to_string(), Json::from(id.as_str()))];
            feature.push(("name".into(), label.clone().unwrap_or_else(|| Json::from(id.as_str()))));
            if let Some(s) = &suppressed {
                feature.push(("suppressed".into(), s.clone()));
            }
            feature.extend(inputs.iter().cloned());
            if let Json::Obj(filled) = check_feature(el, Json::Obj(feature))? {
                inputs = filled.into_iter().filter(|(k, _)| !matches!(k.as_str(), "type" | "id" | "name" | "suppressed")).collect();
            }
        }
        if el.tag != "sketch" {
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
    for (kind, items, sources) in lists {
        let (tag, field, port, _) = THREED_LISTS[kind];
        if !nodes.iter().any(|(id, _)| id == field) {
            claim(&mut owner, field, format!("the document's {field} ({}())", fn_name(tag)))?;
            nodes.push((field.to_string(), node(field, &format!("3d.{field}"), None, Json::obj(), None)));
        }
        let Some((_, n)) = nodes.iter_mut().find(|(id, _)| id == field) else { continue };
        let mut inputs = n.get("inputs").cloned().unwrap_or_else(Json::obj);
        inputs.set(port, Json::Arr(items));
        n.set("inputs", inputs);
        if !sources.is_empty() {
            let mut meta = n.get("meta").cloned().unwrap_or_else(Json::obj);
            let mut all = meta.get("sources").cloned().unwrap_or_else(Json::obj);
            if let Json::Obj(entries) = sources.json() {
                for (k, v) in entries {
                    all.set(&k, v);
                }
            }
            meta.set("sources", all);
            n.set("meta", meta);
        }
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
