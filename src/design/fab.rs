//! MACHINE JOBS IN RUST — a machining setup (`.cam.rs`) and a slicing setup (`.slice.rs`) written as the elements the
//! 3D app's CAM and slicing modes edit: the same elements as the TypeScript SDK's JSX (`fab.ts`) and the Python SDK's
//! (`fab.py`), one call each:
//!
//! ```
//! use commandagi::design::fab::*;
//!
//! fn document() -> El {
//!     cam([
//!         stock().material_id("plywood").thickness_mm(6).x_mm(90).y_mm(70),
//!         machine().post("grbl").max_spindle_rpm(10000),
//!         fixture().name("left clamp").x_mm(-14).y_mm(25).w_mm(20).d_mm(20).z_mm(4),
//!         operation().id("op-1").op("mill_adaptive").profile("adaptive_wood").params(json(r#"{"toolDiameterMm": 3.175}"#)),
//!         runs_on().unit("cloud://global/worlds/fab-cell/world.tsx#cnc-1").channel("gcode").name("cnc 3018 / 01"),
//!     ])
//! }
//! # commandagi::design::declare(document()).unwrap();
//! ```
//!
//! The rule of the ontology's files: a record is an element and its fields are its attributes (methods in snake case
//! of the native names: `.thickness_mm(6)` is `thicknessMm`; millimetres). A setup's single records (`source`,
//! `design`, `stock`, `part`, `machine`, `runs_on`, `spool`) are one element each, absent when the native field is
//! null. A CAM operation is an `operation().id(…)`; the order of the elements is the order the cuts run. An operation's
//! `params`, `tabs` and `region` and a slicing setup's `params` are the native object, written as `json(r#"…"#)`.
//! Nothing adds a default. Each record's call is in the document's `sources` by its path (`operation#op-1`, `stock`).

use super::documents::{declare_document, DocTree, TagRule, Vocabulary};
use super::element::{attributes, elements};
use super::{Declared, Family};
pub use super::{fragment, json, El, Json, Value};

elements! {
    /// A machining setup: its records, its operations in the order the cuts run.
    cam: children;
    /// A slicing setup: `slice([…]).profile("fdm_pla_0.20_draft")`.
    slice: children;
    /// The file a setup starts from: `.file_id(…).name(…)`.
    source: leaf;
    /// The design a setup machines: `.file_id(…).name(…)`.
    design: leaf;
    /// The stock: `.material_id(…).thickness_mm(…).x_mm(…).y_mm(…)`.
    stock: leaf;
    /// The part's footprint and where it sits on the stock.
    part: leaf;
    /// A machining setup's machine (`.post(…)` …), or a slicing setup's (`.unit(…).channel(…).name(…)`).
    machine: leaf;
    /// A fixture: `.name(…).x_mm(…).y_mm(…).w_mm(…).d_mm(…).z_mm(…)`.
    fixture: leaf;
    /// A CAM operation: `.id(…).op(…).profile(…)`.
    operation: leaf;
    /// The unit's channel a machining setup runs on.
    runs_on = "runsOn": leaf;
    /// A slicing setup's filament.
    spool: leaf;
}

attributes! {
    /// The fields of a machine job's records, chained on its elements.
    pub trait FabAttrs {
        /// A file's id (its path).
        file_id;
        /// A record's name.
        name;
        /// A stock's or a spool's material.
        material_id;
        /// The stock's thickness (mm).
        thickness_mm;
        /// Across (mm).
        x_mm;
        /// Along (mm).
        y_mm;
        /// The part's footprint across (mm).
        footprint_x_mm;
        /// The part's footprint along (mm).
        footprint_y_mm;
        /// Where the part sits on the stock, across (mm).
        at_x_mm;
        /// Where the part sits on the stock, along (mm).
        at_y_mm;
        /// The machine's post processor (`"grbl"`).
        post;
        /// The machine's spindle limit.
        max_spindle_rpm;
        /// The machine's feed limit.
        max_feed_mm_per_min;
        /// The spindle's power.
        spindle_power_kw;
        /// A fixture's width (mm).
        w_mm;
        /// A fixture's depth (mm).
        d_mm;
        /// A fixture's height (mm).
        z_mm;
        /// An operation's id.
        id;
        /// An operation's kind (`"mill_contour"`).
        op;
        /// An operation's profile, or a slicing setup's.
        profile;
        /// An operation is enabled unless it says `false`.
        enabled;
        /// An operation's or a slicing setup's overrides: `json(r#"{"toolDiameterMm": 3.175}"#)`.
        params;
        /// An operation's tabs: `json(r#"{"count": 4, "lengthMm": 5, "heightMm": 1.5}"#)`.
        tabs;
        /// An operation's region: `json(r#"{"boundary": [[27, 25], [63, 25]], "depthMm": 3}"#)`.
        region;
        /// The unit a job runs on, by its address.
        unit;
        /// The unit's channel (`"gcode"`).
        channel;
        /// A spool's filament diameter (mm).
        diameter_mm;
    }
}

pub(crate) const FAMILY: Family = Family { module: "fab", roots: &["cam", "slice"], declare };

pub const CAM_REF_FIELDS: &[&str] = &["fileId", "name"];
pub const CAM_STOCK_FIELDS: &[&str] = &["materialId", "thicknessMm", "xMm", "yMm"];
pub const CAM_PART_FIELDS: &[&str] = &["footprintXMm", "footprintYMm", "atXMm", "atYMm"];
pub const CAM_MACHINE_FIELDS: &[&str] = &["post", "maxSpindleRpm", "maxFeedMmPerMin", "spindlePowerKw"];
pub const CAM_FIXTURE_FIELDS: &[&str] = &["name", "xMm", "yMm", "wMm", "dMm", "zMm"];
pub const CAM_OPERATION_FIELDS: &[&str] = &["id", "op", "profile", "enabled", "params", "tabs", "region"];
/// A unit's channel a job runs on (`runsOn` of a machining setup, `machine` of a slicing setup).
pub const TARGET_FIELDS: &[&str] = &["unit", "channel", "name"];
pub const SPOOL_FIELDS: &[&str] = &["materialId", "diameterMm"];

const CAM_PARENT: &[&str] = &["cam"];
const SLICE_PARENT: &[&str] = &["slice"];
const REF_REQUIRED: &[&str] = &["fileId", "name"];

/// A machining setup.
pub const CAM: Vocabulary = Vocabulary {
    format: "cam",
    noun: "a machining setup",
    root: "cam",
    tags: &[
        TagRule::new("cam", &[]).attrs(&[]),
        TagRule::new("source", CAM_PARENT).single().required(REF_REQUIRED).attrs(CAM_REF_FIELDS),
        TagRule::new("design", CAM_PARENT).single().required(REF_REQUIRED).attrs(CAM_REF_FIELDS),
        TagRule::new("stock", CAM_PARENT).single().attrs(CAM_STOCK_FIELDS),
        TagRule::new("part", CAM_PARENT).single().required(&["footprintXMm", "footprintYMm"]).attrs(CAM_PART_FIELDS),
        TagRule::new("machine", CAM_PARENT).single().attrs(CAM_MACHINE_FIELDS),
        TagRule::new("fixture", CAM_PARENT).required(CAM_FIXTURE_FIELDS).attrs(CAM_FIXTURE_FIELDS),
        TagRule::new("operation", CAM_PARENT).key("id").required(&["id", "op", "profile"]).attrs(CAM_OPERATION_FIELDS),
        TagRule::new("runsOn", CAM_PARENT).single().required(TARGET_FIELDS).attrs(TARGET_FIELDS),
    ],
    from_tree: cam_from_tree,
};

/// A slicing setup.
pub const SLICE: Vocabulary = Vocabulary {
    format: "slice",
    noun: "a slicing setup",
    root: "slice",
    tags: &[
        TagRule::new("slice", &[]).required(&["profile"]).attrs(&["profile", "params"]),
        TagRule::new("source", SLICE_PARENT).single().required(REF_REQUIRED).attrs(CAM_REF_FIELDS),
        TagRule::new("design", SLICE_PARENT).single().required(REF_REQUIRED).attrs(CAM_REF_FIELDS),
        TagRule::new("spool", SLICE_PARENT).single().attrs(SPOOL_FIELDS),
        TagRule::new("machine", SLICE_PARENT).single().required(TARGET_FIELDS).attrs(TARGET_FIELDS),
    ],
    from_tree: slice_from_tree,
};

fn cam_from_tree(t: &DocTree) -> Result<Json, String> {
    let mut doc = Json::obj().with("operations", Json::Arr(Vec::new()));
    for tag in ["source", "design", "stock", "part", "machine"] {
        if let Some(c) = t.single(tag) {
            doc.set(tag, c.record());
        }
    }
    let fixtures: Vec<Json> = t.children.iter().filter(|c| c.tag == "fixture").map(DocTree::record).collect();
    if !fixtures.is_empty() {
        doc.set("fixtures", fixtures);
    }
    // `operations` keeps its first place (TypeScript's key order); its value is set now.
    doc.set("operations", t.children.iter().filter(|c| c.tag == "operation").map(DocTree::record).collect::<Vec<_>>());
    if let Some(target) = t.single("runsOn") {
        doc.set("runsOn", target.record());
    }
    Ok(doc)
}

fn slice_from_tree(t: &DocTree) -> Result<Json, String> {
    let mut doc = t.record();
    for tag in ["source", "design", "spool", "machine"] {
        if let Some(c) = t.single(tag) {
            doc.set(tag, c.record());
        }
    }
    Ok(doc)
}

fn declare(root: El) -> Result<Declared, String> {
    declare_document(&root, if root.tag == "cam" { &CAM } else { &SLICE })
}
