//! RECORDS IN RUST — a market contract (`.contract.rs`) and a product instance (`.instance.rs`), which are signed, a
//! hardware release (`.release.rs`) and a unit's calibration (`.calibration.rs`: `calibration([joint()…,
//! rezero()…]).device(…).serial(…)`), written as elements. The same elements, and the same documents, as the TypeScript SDK's `records.ts` and the Python SDK's
//! `records.py`:
//!
//! ```
//! use commandagi::design::records::*;
//!
//! fn document() -> El {
//!     instance([
//!         event().seq(0).kind("manufacture").at(1772409600000_i64).prev(json("null")).by("org-acme").sig("ed25519:…"),
//!     ])
//!     .serial("ACME-FU-0031")
//!     .product_id("fleet-unit")
//! }
//! # commandagi::design::declare(document()).unwrap();
//! ```
//!
//! The rule of the ontology's files: a record is an element and its fields are the element's attributes. These
//! records are SIGNED, so nothing is rewritten on the way: a contract's `terms` are ONE attribute (the exact object
//! both parties sign, `json(r#"{…}"#)`, nulls and empty strings kept), times stay epoch milliseconds (an integer past
//! `i32` is written with its type: `1780765664000_i64`), signatures are written as they are. A contract's parties are
//! `party()` children in order; an instance's provenance chain is `event()` children, oldest first, each named by its
//! `seq`. Nothing adds a default. A run gives back `{format, document, sources}` beside an empty graph; `sources`
//! names each element's site by its path (`""`, `party@0`, `event#0`).

use super::documents::{declare_document, js_string, DocTree, TagRule, Vocabulary};
use super::element::{attributes, elements};
use super::{Declared, Family};
pub use super::{fragment, json, El, Json, Value};

elements! {
    /// A contract record: its parties.
    contract: children;
    /// A party of a contract, its fields as stored: `.principal("u-1").role("offeror")`.
    party: leaf;
    /// A product instance: its provenance chain.
    instance: children;
    /// An event of an instance's chain: `.seq(0).kind("manufacture").at(…).by("org-acme")`.
    event: leaf;
    /// A hardware release: its frozen files, its check verdicts and its outputs.
    release: children;
    /// A frozen file of a release: `.path("cad/Plate.3d.tsx").digest("<sha256>")`.
    file: leaf;
    /// A check's verdict at a frozen file's digest: `.file(…).id("interference").kind("interference").status("passed")`.
    check: leaf;
    /// An output a release made: `.path("outputs/Plate.step").digest(…).from(…).kind("step")`.
    output: leaf;
    /// A unit's calibration: its joints and its re-zeros. `.device("../../devices/arm/definition.tsx").serial(…)`.
    calibration: children;
    /// A calibrated joint: `.name("shoulder_lift").zero(0.012).kp(12).kv(0.6).run("runs/lift/run.json")`.
    joint: leaf;
    /// A re-zero, appended, never edited: `.joint("shoulder_lift").zero(0.015).at(…).by("person:owner").reason(…)`.
    rezero: leaf;
    /// A calibrated camera: `.channel("view").intrinsics(json(…)).extrinsics(json(…)).residual(json(…)).run(…).reality("simulation")`.
    camera: leaf;
}

attributes! {
    /// The fields of a contract record, a party, a product instance, an event, a release and a calibration, chained on
    /// their elements.
    pub trait RecordsAttrs {
        id; terms; created_at;
        principal; role; acted_by; agent; sig; signed_at;
        serial; product_id; model; release;
        seq; kind; at; prev; by; to; contract_ref; attestation_ref; data;
        rev; of; frozen; product; note; previous; path; digest; file; status; accepted; reason; from;
        device; name; zero; kp; kv; run; joint;
        channel; robot; intrinsics; extrinsics; marker; residual; reality;
    }
}

pub(crate) const FAMILY: Family = Family { module: "records", roots: &["contract", "instance", "release", "calibration"], declare };

/// The fields of a contract record (packages/domain/core/src/contract-document.ts `ContractDoc`), but its parties.
pub const CONTRACT_FIELDS: &[&str] = &["id", "terms", "createdAt"];
/// The fields of a product instance (`ProductInstance`), but its events. `release`: the release it was built from.
pub const INSTANCE_FIELDS: &[&str] = &["serial", "productId", "model", "release"];
/// The fields of a hardware release, but its lists.
pub const RELEASE_FIELDS: &[&str] = &["id", "rev", "of", "frozen", "product", "createdAt", "by", "note", "previous"];
/// The fields of one frozen file of a release.
pub const RELEASE_FILE_FIELDS: &[&str] = &["path", "digest"];
/// The fields of one check verdict of a release.
pub const RELEASE_CHECK_FIELDS: &[&str] = &["file", "id", "kind", "status", "accepted", "reason"];
/// The fields of one output of a release.
pub const RELEASE_OUTPUT_FIELDS: &[&str] = &["path", "digest", "from", "kind"];
/// What a release's check says.
pub const RELEASE_CHECK_STATUSES: &[&str] = &["passed", "failed", "skipped", "accepted"];
/// The fields of a provenance event (`InstanceEvent`).
pub const INSTANCE_EVENT_FIELDS: &[&str] = &["seq", "kind", "at", "prev", "by", "to", "contractRef", "attestationRef", "data", "sig"];
/// The fields of a unit's calibration: the definition it calibrates (a ref), the unit's serial, a note.
pub const CALIBRATION_FIELDS: &[&str] = &["device", "serial", "note"];
/// The fields of one calibrated joint: its zero (rad, what its encoder reads at the chain's zero), its servo's gains,
/// the run that measured them.
pub const CALIBRATION_JOINT_FIELDS: &[&str] = &["name", "zero", "kp", "kv", "run"];
/// The fields of one re-zero: the joint, its new zero (rad), when (epoch ms), who (person:<id> | agent:<id>), why.
pub const REZERO_FIELDS: &[&str] = &["joint", "zero", "at", "by", "reason"];
/// The fields of one calibrated camera: its video channel, the arm it was measured against, its intrinsics (pixels),
/// its extrinsics (the camera in the world: mm and a unit quaternion, x right, y down, z forward), the marker in that
/// arm's tool frame, the residual (reprojection px, and the position mm and rotation degrees it bounds, three sigma),
/// the run that measured it and that run's reality, when and by whom.
pub const CALIBRATION_CAMERA_FIELDS: &[&str] = &["channel", "robot", "intrinsics", "extrinsics", "marker", "residual", "run", "reality", "at", "by"];

const CONTRACT: Vocabulary = Vocabulary {
    format: "contract",
    noun: "a contract record",
    root: "contract",
    tags: &[
        TagRule::new("contract", &[]).required(&["id", "terms", "createdAt"]).attrs(CONTRACT_FIELDS),
        // A party's fields are the record's as stored (principal, role, actedBy, agent, sig, signedAt, …): any.
        TagRule::new("party", &["contract"]).required(&["principal", "role"]),
    ],
    from_tree: contract_of,
};

const INSTANCE: Vocabulary = Vocabulary {
    format: "instance",
    noun: "a product instance",
    root: "instance",
    tags: &[
        TagRule::new("instance", &[]).required(&["serial", "productId"]).attrs(INSTANCE_FIELDS),
        TagRule::new("event", &["instance"]).key("seq").required(&["seq", "kind", "at", "by"]).attrs(INSTANCE_EVENT_FIELDS),
    ],
    from_tree: instance_of,
};

const RELEASE: Vocabulary = Vocabulary {
    format: "release",
    noun: "a hardware release",
    root: "release",
    tags: &[
        TagRule::new("release", &[]).required(&["id", "rev", "of", "frozen"]).attrs(RELEASE_FIELDS),
        TagRule::new("file", &["release"]).key("path").required(&["path", "digest"]).attrs(RELEASE_FILE_FIELDS),
        TagRule::new("check", &["release"]).required(&["file", "id", "kind", "status"]).attrs(RELEASE_CHECK_FIELDS),
        TagRule::new("output", &["release"]).key("path").required(&["path", "digest"]).attrs(RELEASE_OUTPUT_FIELDS),
    ],
    from_tree: release_of,
};

/// A unit's calibration (`<name>.calibration.rs`): what one physical unit measures off its definition. A unit of a
/// world names it (`unit().calibration("arm.calibration.rs")`). Each `joint()` holds the joint's zero and its servo's
/// gains as this unit has them; each `rezero()` is an event of the unit, oldest first. A joint's zero is its newest
/// re-zero's, else its `joint()`'s. A re-zero is appended, never edited. A camera unit's calibration holds a `camera()`
/// per video channel: its lens and where it is, measured by a run that moved a marker through its view, with the
/// residual that says how far to trust them; a new run replaces it.
const CALIBRATION: Vocabulary = Vocabulary {
    format: "calibration",
    noun: "a unit's calibration",
    root: "calibration",
    tags: &[
        TagRule::new("calibration", &[]).required(&["device"]).attrs(CALIBRATION_FIELDS),
        TagRule::new("joint", &["calibration"]).key("name").required(&["name"]).attrs(CALIBRATION_JOINT_FIELDS),
        TagRule::new("rezero", &["calibration"]).required(&["joint", "zero", "at", "by"]).attrs(REZERO_FIELDS),
        TagRule::new("camera", &["calibration"]).key("channel").required(&["channel", "intrinsics", "extrinsics", "residual"]).attrs(CALIBRATION_CAMERA_FIELDS),
    ],
    from_tree: calibration_of,
};

fn declare(root: El) -> Result<Declared, String> {
    declare_document(&root, match &*root.tag {
        "contract" => &CONTRACT,
        "release" => &RELEASE,
        "calibration" => &CALIBRATION,
        _ => &INSTANCE,
    })
}

fn finite(v: Option<&Json>) -> Option<f64> {
    v.and_then(Json::as_f64).filter(|n| n.is_finite())
}

fn said(v: Option<&Json>) -> String {
    v.map(Json::text).unwrap_or_else(|| "undefined".into())
}

fn calibration_of(t: &DocTree) -> Result<Json, String> {
    let list = |tag: &str| -> Vec<Json> { t.children.iter().filter(|c| c.tag == tag).map(DocTree::record).collect() };
    let (joints, rezeros, cameras) = (list("joint"), list("rezero"), list("camera"));
    for j in &joints {
        let at = format!("joint().name({})", said(j.get("name")));
        if j.get("zero").is_some() && finite(j.get("zero")).is_none() {
            return Err(format!("{at}: zero is radians, not {}", said(j.get("zero"))));
        }
        if j.get("kp").is_some() && !finite(j.get("kp")).is_some_and(|n| n > 0.0) {
            return Err(format!("{at}: kp is a positive number (N·m/rad), not {}", said(j.get("kp"))));
        }
        if j.get("kv").is_some() && !finite(j.get("kv")).is_some_and(|n| n >= 0.0) {
            return Err(format!("{at}: kv is a number of at least 0 (N·m·s/rad), not {}", said(j.get("kv"))));
        }
    }
    let mut last = f64::NEG_INFINITY;
    for r in &rezeros {
        let at = format!("rezero().joint({})", said(r.get("joint")));
        if finite(r.get("zero")).is_none() {
            return Err(format!("{at}: zero is radians, not {}", said(r.get("zero"))));
        }
        let Some(when) = finite(r.get("at")).filter(|n| n.fract() == 0.0) else {
            return Err(format!("{at}: at is epoch milliseconds, not {}", said(r.get("at"))));
        };
        let by_ok = matches!(r.get("by"), Some(Json::Str(b)) if (b.starts_with("person:") && b.len() > 7) || (b.starts_with("agent:") && b.len() > 6));
        if !by_ok {
            return Err(format!("{at}: by is person:<id> or agent:<id>, not {}", said(r.get("by"))));
        }
        if when < last {
            return Err(format!("{at}: the re-zeros are oldest first, and this one is older than the one before it"));
        }
        last = when;
    }
    for c in &cameras {
        camera_checked(c)?;
    }
    let mut out = Json::obj();
    for k in CALIBRATION_FIELDS {
        if let Some(v) = t.attr_of(k) {
            out.set(k, v.clone());
        }
    }
    let out = out.with("joints", joints).with("rezeros", rezeros);
    Ok(if cameras.is_empty() { out } else { out.with("cameras", cameras) })
}

/// Whether `v` is an array of `n` finite numbers.
fn numbers(v: Option<&Json>, n: usize) -> bool {
    matches!(v, Some(Json::Arr(a)) if a.len() == n && a.iter().all(|x| finite(Some(x)).is_some()))
}

/// A `camera()` as a calibration holds it, or the reason it is not one (the TypeScript SDK's `cameraChecked`).
fn camera_checked(c: &Json) -> Result<(), String> {
    let at = format!("camera().channel({})", said(c.get("channel")));
    let i = c.get("intrinsics");
    let positive = |k: &str| finite(i.and_then(|i| i.get(k))).is_some_and(|n| n > 0.0);
    let lens = matches!(i, Some(Json::Obj(_))) && ["width", "height", "fx", "fy"].iter().all(|k| positive(k)) && ["cx", "cy"].iter().all(|k| finite(i.and_then(|i| i.get(k))).is_some());
    if !lens {
        return Err(format!("{at}: intrinsics are {{ width, height, fx, fy, cx, cy }} in pixels (width, height, fx and fy positive), not {}", said(i)));
    }
    let e = c.get("extrinsics");
    if !(matches!(e, Some(Json::Obj(_))) && numbers(e.and_then(|e| e.get("position_mm")), 3) && numbers(e.and_then(|e| e.get("quaternion")), 4)) {
        return Err(format!("{at}: extrinsics are {{ position_mm: [x, y, z], quaternion: [x, y, z, w] }}, not {}", said(e)));
    }
    if let Some(m) = c.get("marker") {
        if !(matches!(m, Json::Obj(_)) && numbers(m.get("position_mm"), 3) && numbers(m.get("quaternion"), 4)) {
            return Err(format!("{at}: marker is {{ position_mm: [x, y, z], quaternion: [x, y, z, w] }}, not {}", m.text()));
        }
    }
    let r = c.get("residual");
    if !(matches!(r, Some(Json::Obj(_))) && finite(r.and_then(|r| r.get("rms_px"))).is_some_and(|n| n >= 0.0)) {
        return Err(format!("{at}: residual has rms_px, the reprojection error in pixels, not {}", said(r)));
    }
    match c.get("reality") {
        None if c.get("run").is_some() => return Err(format!("{at}: a camera measured by a run says its reality (a simulated run is never physical evidence)")),
        Some(Json::Str(x)) if x != "physical" && x != "simulation" => return Err(format!("{at}: reality is physical or simulation, not {}", said(c.get("reality")))),
        Some(v) if !matches!(v, Json::Str(_)) => return Err(format!("{at}: reality is physical or simulation, not {}", v.text())),
        _ => {}
    }
    Ok(())
}

fn release_of(t: &DocTree) -> Result<Json, String> {
    let at = format!("release().rev({})", Json::from(js_string(t.attr_of("rev"))).text());
    let list = |tag: &str| -> Vec<Json> { t.children.iter().filter(|c| c.tag == tag).map(DocTree::record).collect() };
    let (files, checks, outputs) = (list("file"), list("check"), list("output"));
    for f in files.iter().chain(outputs.iter()) {
        let ok = matches!(f.get("digest"), Some(Json::Str(d)) if d.len() == 64 && d.chars().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()));
        if !ok {
            let said = f.get("digest").map(Json::text).unwrap_or_else(|| "undefined".into());
            let path = f.get("path").map(Json::text).unwrap_or_default();
            return Err(format!("{at}: {path}'s digest is its content's SHA-256 in hex (64 characters), not {said}"));
        }
    }
    for c in &checks {
        if !matches!(c.get("status"), Some(Json::Str(s)) if RELEASE_CHECK_STATUSES.contains(&s.as_str())) {
            let said = c.get("status").map(Json::text).unwrap_or_else(|| "undefined".into());
            return Err(format!("{at}: a check's status is {}, not {said}", RELEASE_CHECK_STATUSES.join(", ")));
        }
    }
    let frozen_ok = matches!(t.attr_of("frozen"), Some(Json::Str(f)) if !f.is_empty() && !f.starts_with('/') && f.split('/').all(|p| !p.is_empty() && p != "." && p != ".."));
    if !frozen_ok {
        let said = t.attr_of("frozen").map(Json::text).unwrap_or_else(|| "undefined".into());
        return Err(format!("{at}: frozen is a folder beside the record (files, files/<the folder's names>), not {said}"));
    }
    match t.attr_of("createdAt") {
        None => {}
        Some(Json::Num(n)) if n.is_finite() && n.fract() == 0.0 => {}
        Some(other) => return Err(format!("{at}: created_at is epoch milliseconds, not {}", other.text())),
    }
    let mut out = Json::obj();
    for k in RELEASE_FIELDS {
        if let Some(v) = t.attr_of(k) {
            out.set(k, v.clone());
        }
    }
    Ok(out.with("files", files).with("checks", checks).with("outputs", outputs))
}

fn field(t: &DocTree, name: &str) -> Json {
    t.attr_of(name).cloned().unwrap_or(Json::Null)
}

fn contract_of(t: &DocTree) -> Result<Json, String> {
    let at = format!("contract().id({})", Json::from(js_string(t.attr_of("id"))).text());
    if !matches!(t.attr_of("terms"), Some(Json::Obj(_))) {
        return Err(format!("{at}: terms is the object the parties sign"));
    }
    match t.attr_of("createdAt") {
        Some(Json::Num(n)) if n.is_finite() && n.fract() == 0.0 => {}
        other => {
            let said = other.map(Json::text).unwrap_or_else(|| "undefined".into());
            return Err(format!("{at}: created_at is epoch milliseconds, not {said}"));
        }
    }
    let parties: Vec<Json> = t.children.iter().map(DocTree::record).collect();
    Ok(Json::obj()
        .with("id", field(t, "id"))
        .with("terms", field(t, "terms"))
        .with("parties", parties)
        .with("createdAt", field(t, "createdAt")))
}

fn instance_of(t: &DocTree) -> Result<Json, String> {
    let events: Vec<Json> = t.children.iter().map(DocTree::record).collect();
    for (i, e) in events.iter().enumerate() {
        if e.get("seq") != Some(&Json::Num(i as f64)) {
            let said = e.get("seq").map(Json::text).unwrap_or_else(|| "undefined".into());
            return Err(format!("event().seq({said}) is event {i} of the chain: events are written oldest first, seq 0, 1, 2 …"));
        }
    }
    let mut out = Json::obj();
    for k in INSTANCE_FIELDS {
        if let Some(v) = t.attr_of(k) {
            out.set(k, v.clone());
        }
    }
    Ok(out.with("events", events))
}
