//! STUDIES IN RUST — a study (`<name>.study.rs`) holds the question of an analysis of a part, never the answer: the
//! same elements, and the same document, as the TypeScript SDK's JSX (`study.ts`) and the Python SDK's
//! (`commandagi.design.study`), one call each:
//!
//! ```
//! use commandagi::design::study::*;
//!
//! fn document() -> El {
//!     study([
//!         material().grade("PETG").poisson(0.38),
//!         fixture().id("mount").type_("fixed").faces(json(r#"{"kind": "face", "select": {"normal": [0, 0, -1]}}"#)),
//!         load_case([
//!             force().id("grip").faces(json(r#"{"kind": "face", "select": {"farthest": [1, 0, 0]}}"#)).total([-40, 0, 0]),
//!         ])
//!         .id("clamp"),
//!         mesh().order(2).size(2).refine([1.0, 0.7, 0.5]),
//!         criterion().id("safety").quantity("vonMises.max").limit("yield / 2"),
//!         revise().parameter("thickness").min(3).max(8),
//!     ])
//!     .kind("structural")
//!     .part("Jaw.3d.tsx")
//!     .units("mm-N-MPa-t")
//! }
//! # commandagi::design::declare(document()).unwrap();
//! ```
//!
//! One root, `study([…]).kind(…).part(…).units(…)`, for every kind of study. A kind has its own tags: this module
//! knows `structural` (FEA) and `flow` (CFD: `fluid()`, `inlet()`, `outlet()`, `flow_mesh()`, in `units("m-kg-s")`). `criterion()` and `revise()` are every kind's. A record is an element and its fields are
//! its attributes (methods in snake case of the native names: `.load_case("clamp")` on a criterion is `loadCase`). A
//! load's tag is its `type`. A face pick is the 3D document's pick, written as `json(r#"…"#)`. Nothing adds a
//! default: the runner says what it assumes. A tag of one kind is refused in a study of another, by name.
//!
//! The document a study declares:
//!
//! ```text
//! { kind, part, units, title?, material?, fixtures?: [{ id, type, faces, region? }], loadCases?: [{ id, loads: [{ type:
//!   "force" | "pressure", id, faces, total | value }] }], mesh?, solver?, criteria?: [{ id, quantity, limit, region?,
//!   loadCase?, note? }], revise?: [{ parameter, min, max }] }
//! ```

use super::documents::{declare_document, DocTree, TagRule, Vocabulary};
use super::element::{attributes, elements, fn_name};
use super::{Declared, Family};
pub use super::{fragment, json, El, Json, Value};

elements! {
    /// A study: its records. `.kind(…).part(…).units(…)`.
    study: children;
    /// What the part is made of: `material().grade("PETG").poisson(0.38)`.
    material: leaf;
    /// What holds the part: `fixture().id(…).type_("fixed").faces(json(…))`.
    fixture: leaf;
    /// A load case: its loads, each solved alone. `load_case([…]).id("clamp")`.
    load_case = "loadCase": children;
    /// A force spread over faces by area: `force().id(…).faces(json(…)).total([0, 0, -100])` (N).
    force: leaf;
    /// A pressure on faces: `pressure().id(…).faces(json(…)).value(0.1)` (MPa, positive pushes into the part).
    pressure: leaf;
    /// The mesh levels: `mesh().order(2).size(5).refine([1.0, 0.7, 0.5])`.
    mesh: leaf;
    /// The solver: `solver().kind("calculix").analysis("static")`.
    solver: leaf;
    /// A quantity that must stay within a limit: `criterion().id(…).quantity("vonMises.max").limit("yield / 2")`.
    criterion: leaf;
    /// A parameter of the part the planner may change: `revise().parameter("thickness").min(3).max(8)`.
    revise: leaf;
    /// A flow study's fluid: `fluid().nu(1e-6).body("body_0")` (kinematic viscosity, m²/s).
    fluid: leaf;
    /// Where the fluid enters: `inlet().faces(json(…)).velocity(0.5)` (m/s, into the body).
    inlet: leaf;
    /// Where the fluid leaves (pressure 0): `outlet().faces(json(…))`.
    outlet: leaf;
    /// A flow study's mesh: `flow_mesh().cells([20, 3, 3]).levels([1, 2, 4])`.
    flow_mesh = "flowMesh": leaf;
}

attributes! {
    /// The fields of a study's records, chained on its elements.
    pub trait StudyAttrs {
        /// The study's kind (`"structural"`), or a solver's (`"calculix"`).
        kind;
        /// The part it studies, by its path from this file.
        part;
        /// The unit set: `"mm-N-MPa-t"`, the only one.
        units;
        title;
        /// A row id of the materials tables along the folder's search path.
        grade;
        /// The one body exported for the study.
        body;
        /// Poisson's ratio ν, when the study states it; else the materials table's.
        poisson;
        id;
        /// A fixture's type: `"fixed"` or `"roller"`.
        type_;
        /// A face pick: `json(r#"{"kind": "face", "select": {"normal": [1, 0, 0]}}"#)`.
        faces;
        /// A force's total, [Fx, Fy, Fz] in N.
        total;
        /// A pressure, MPa.
        value;
        /// The elements' order: 2 (quadratic tetrahedra).
        order;
        /// The coarsest element size (mm).
        size;
        /// Decreasing factors of the size, one mesh level each.
        refine;
        /// The analysis: `"static"`.
        analysis;
        /// Seconds one solve may take (CalculiX); the kernel's own limit (900 s) when not given.
        timeout;
        /// What a criterion reads: `"vonMises.max"`, `"displacement.max"`, `"displacement.z.min"`.
        quantity;
        /// A number, or an expression of the material's `yield`, `tensile`, `E`: `"yield / 2"`.
        limit;
        /// Where a criterion reads: `json(r#"{"faces": …}"#)`, `{"inBox": {"min", "max"}}`, `{"not": …}`; the part of a
        /// fixture's faces it holds: `{"inBox": {"min", "max"}}`.
        region;
        /// The load case a criterion reads.
        load_case;
        /// A fluid's kinematic viscosity (m²/s).
        nu;
        /// The inlet's mean velocity (m/s).
        velocity;
        /// The background block's cells along x, y and z at the coarsest level.
        cells;
        /// The refinement multipliers of the cells, one level each: `[1, 2, 4]`.
        levels;
        /// The solver's iterations at most.
        iterations;
        /// `"wall-functions"` or `"resolved"`.
        wall_model;
        note;
        /// A named parameter of the part.
        parameter;
        min;
        max;
    }
}

pub(crate) const FAMILY: Family = Family { module: "study", roots: &["study"], declare };

/// The one unit set a study may declare: millimetres, newtons, megapascals, tonnes.
pub const STUDY_UNITS: &str = "mm-N-MPa-t";
pub const STUDY_ROOT_FIELDS: &[&str] = &["kind", "part", "units", "title"];
/// The kinds a study may name.
pub const STUDY_KINDS: &[&str] = &["structural", "flow"];

/// A tag of a study: the field of its parent record it fills, whether that field is a list, whether the tag is the
/// record's `type` (a load is `force()` or `pressure()`, both in `loads`), and its kind (none: every kind's).
pub struct StudyTag {
    pub tag: &'static str,
    pub field: &'static str,
    pub list: bool,
    pub typed: bool,
    pub kind: Option<&'static str>,
}

const fn tag(tag: &'static str, field: &'static str, list: bool, typed: bool, kind: Option<&'static str>) -> StudyTag {
    StudyTag { tag, field, list, typed, kind }
}

const S: Option<&str> = Some("structural");
const F: Option<&str> = Some("flow");
pub const STUDY_TAGS: &[StudyTag] = &[
    tag("criterion", "criteria", true, false, None),
    tag("revise", "revise", true, false, None),
    tag("material", "material", false, false, S),
    tag("fixture", "fixtures", true, false, S),
    tag("loadCase", "loadCases", true, false, S),
    tag("force", "loads", true, true, S),
    tag("pressure", "loads", true, true, S),
    tag("mesh", "mesh", false, false, S),
    tag("solver", "solver", false, false, S),
    tag("fluid", "fluid", false, false, F),
    tag("inlet", "inlet", false, false, F),
    tag("outlet", "outlet", false, false, F),
    tag("flowMesh", "flowMesh", false, false, F),
];

const IN_STUDY: &[&str] = &["study"];
const FACES_LOAD: &[&str] = &["id", "faces", "total"];
const PRESSURE: &[&str] = &["id", "faces", "value"];
const FIXTURE: &[&str] = &["id", "type", "faces"];

/// A study.
pub const STUDY: Vocabulary = Vocabulary {
    format: "study",
    noun: "a study",
    root: "study",
    tags: &[
        TagRule::new("study", &[]).required(&["kind", "part"]).attrs(STUDY_ROOT_FIELDS),
        TagRule::new("criterion", IN_STUDY).key("id").required(&["id", "quantity", "limit"]).attrs(&["id", "quantity", "limit", "region", "loadCase", "note"]),
        TagRule::new("revise", IN_STUDY).key("parameter").required(&["parameter", "min", "max"]).attrs(&["parameter", "min", "max"]),
        TagRule::new("material", IN_STUDY).single().required(&["grade"]).attrs(&["grade", "body", "poisson"]),
        TagRule::new("fixture", IN_STUDY).key("id").required(FIXTURE).attrs(&["id", "type", "faces", "region"]),
        TagRule::new("loadCase", IN_STUDY).key("id").required(&["id"]).attrs(&["id"]),
        TagRule::new("force", &["loadCase"]).key("id").required(FACES_LOAD).attrs(FACES_LOAD),
        TagRule::new("pressure", &["loadCase"]).key("id").required(PRESSURE).attrs(PRESSURE),
        TagRule::new("mesh", IN_STUDY).single().attrs(&["order", "size", "refine"]),
        TagRule::new("solver", IN_STUDY).single().attrs(&["kind", "analysis", "timeout"]),
        TagRule::new("fluid", IN_STUDY).single().required(&["nu"]).attrs(&["nu", "body"]),
        TagRule::new("inlet", IN_STUDY).single().required(&["faces", "velocity"]).attrs(&["faces", "velocity"]),
        TagRule::new("outlet", IN_STUDY).single().required(&["faces"]).attrs(&["faces"]),
        TagRule::new("flowMesh", IN_STUDY).single().required(&["cells"]).attrs(&["cells", "levels", "iterations", "wallModel"]),
    ],
    from_tree: study_from_tree,
};

fn study_tag(tag: &str) -> Option<&'static StudyTag> {
    STUDY_TAGS.iter().find(|t| t.tag == tag)
}

/// Each child's record into the field its tag fills (a list's items in order).
fn records_of(children: &[DocTree], into: &mut Json) {
    for c in children {
        let Some(t) = study_tag(c.tag) else { continue };
        let mut rec = if t.typed { Json::obj().with("type", c.tag) } else { Json::obj() };
        for (k, v) in &c.attrs {
            rec.set(k, v.clone());
        }
        records_of(&c.children, &mut rec);
        if t.list {
            match into.get(t.field) {
                Some(Json::Arr(_)) => {}
                _ => {
                    into.set(t.field, Json::Arr(Vec::new()));
                }
            }
            if let Json::Obj(entries) = into {
                if let Some((_, Json::Arr(items))) = entries.iter_mut().find(|(k, _)| k == t.field) {
                    items.push(rec);
                }
            }
        } else {
            into.set(t.field, rec);
        }
    }
}

fn study_from_tree(t: &DocTree) -> Result<Json, String> {
    let mut doc = t.record();
    let kind = doc.get("kind").cloned();
    let Some(kind) = kind.as_ref().and_then(Json::as_str).filter(|k| STUDY_KINDS.contains(k)) else {
        return Err(format!("study() kind is one of {}, not {}", STUDY_KINDS.join(", "), kind.map(|k| k.text()).unwrap_or_else(|| "undefined".into())));
    };
    for c in &t.children {
        if let Some(k) = study_tag(c.tag).and_then(|s| s.kind) {
            if k != kind {
                return Err(format!("{}() belongs to a {k} study, not to a {kind} one", fn_name(c.tag)));
            }
        }
    }
    records_of(&t.children, &mut doc);
    Ok(doc)
}

fn declare(root: El) -> Result<Declared, String> {
    declare_document(&root, &STUDY)
}
