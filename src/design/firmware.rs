//! FIRMWARE RELEASES AND ROLLOUTS IN RUST — the same elements, and the same documents, as the TypeScript SDK's JSX
//! (`firmware.ts`) and the Python SDK's (`commandagi.design.firmware`), one call each.
//!
//! A firmware release (`<name>.firmware.rs`) names one built image by ref and states what it is: its SHA-256 (the
//! digest a unit reports when it runs it), its SBOM, the device definition it is for and the hardware revisions it
//! supports. It holds no bytes: the image and the SBOM are files beside it.
//!
//! ```
//! use commandagi::design::firmware::*;
//!
//! fn document() -> El {
//!     firmware([supports().revision("B"), supports().revision("C")])
//!         .version("1.4.0")
//!         .image("build/cam-002-1.4.0.bin")
//!         .sha256("9f2c000000000000000000000000000000000000000000000000000000000000")
//!         .size(1203456)
//!         .sbom("build/cam-002-1.4.0.cdx.json")
//!         .device("../devices/commandagi-cam-002/definition.tsx")
//!         .security_version(2)
//! }
//! # commandagi::design::declare(document()).unwrap();
//! ```
//!
//! A rollout (`<name>.rollout.rs`) pushes one release to the units of one world in stages: a canary share first, then
//! waves (cumulative percentages, the last one 100). A failure halts the rollout. Nothing retries an update: a retry is
//! a principal's own recorded command.
//!
//! ```
//! use commandagi::design::firmware::*;
//!
//! fn document() -> El {
//!     rollout([
//!         health().id("no-crash").channel("state").command(json(r#"{"verb": "read"}"#)).read("crashes").max(0),
//!         target().unit("cam-1"),
//!     ])
//!     .firmware("cam-002 1.4.0.firmware.tsx")
//!     .world("worlds/fleet/world.tsx")
//!     .channel("firmware")
//!     .canary(10)
//!     .waves([50, 100])
//!     .settle_ms(60000)
//! }
//! # commandagi::design::declare(document()).unwrap();
//! ```
//!
//! A firmware's project folder holds `firmware.rs`, the same element without a built image: what the firmware gives,
//! its channels as a definition writes them (the ontology's `channel()`), and the revisions it supports. A definition
//! names that folder (or a release by `sha256:<hex>`) as its `firmware`; `commandagi check <definition>` holds the two to
//! each other.
//!
//! ```
//! use commandagi::design::firmware::*;
//!
//! mod channels {
//!     use commandagi::design::ontology::*;
//!     pub fn cam() -> El { channel().id("cam").dir("out").medium("video").format("video") }
//! }
//!
//! fn document() -> El {
//!     firmware([supports().revision("A"), channels::cam()]).name("ESP32 camera")
//! }
//! # commandagi::design::declare(document()).unwrap();
//! ```
//!
//! `target().unit(…)`: the units it updates, in that world; none: every unit of the world whose definition is the
//! release's device. A record is an element and its fields are its attributes.
//!
//! ```text
//! firmware: { name?, version?, image?, sha256?, sbom?, device?, size?, securityVersion?, title?, note?,
//!             supports?: [{ revision }], channels?: [{ id, … }] }: a release states all of version, image, sha256,
//!             sbom and device; a project's declaration states its name
//! rollout:  { firmware, world, channel, canary, waves, settleMs?, title?, note?, health?: [{ id, channel, read,
//!             command?, min?, max?, equals?, unit?, timeoutMs? }], targets?: [{ unit }] }
//! ```

use super::documents::{declare_document, DocTree, TagRule, Vocabulary};
use super::element::{attributes, elements};
use super::{Declared, Family};
pub use super::{fragment, json, El, Json, Value};

elements! {
    /// A firmware: the hardware revisions it supports and the channels it gives. A release:
    /// `.version(…).image(…).sha256(…).sbom(…).device(…)`; a project's declaration: `.name(…)`.
    firmware: children;
    /// A hardware revision the release supports: `supports().revision("B")`.
    supports: leaf;
    /// A rollout: its health checks and its targets. `.firmware(…).world(…).channel(…).canary(10).waves([50, 100])`.
    rollout: children;
    /// A check every updated unit must pass before the next wave: `health().id(…).channel(…).read(…).max(0)`.
    health: leaf;
    /// A unit the rollout updates: `target().unit("cam-1")`.
    target: leaf;
}

attributes! {
    /// The fields of a release's and a rollout's records, chained on their elements.
    pub trait FirmwareAttrs {
        /// A firmware's name, in its project's declaration.
        name;
        version; image;
        /// The image's SHA-256, 64 lowercase hex digits.
        sha256;
        size; sbom; device; security_version; title; note; revision;
        /// A rollout's release, by its path from this file.
        firmware;
        world; channel; canary; waves; settle_ms;
        id; command; read; min; max; equals; unit; timeout_ms;
    }
}

pub(crate) const FAMILY: Family = Family { module: "firmware", roots: &["firmware", "rollout"], declare };

pub const FIRMWARE_FIELDS: &[&str] = &["name", "version", "image", "sha256", "size", "sbom", "device", "securityVersion", "title", "note"];
/// The fields a release states, all of them (a firmware's declaration in its project states none of them).
pub const FIRMWARE_RELEASE_FIELDS: &[&str] = &["version", "image", "sha256", "sbom", "device"];
pub const ROLLOUT_FIELDS: &[&str] = &["firmware", "world", "channel", "canary", "waves", "settleMs", "title", "note"];
pub const HEALTH_FIELDS: &[&str] = &["id", "channel", "command", "read", "min", "max", "equals", "unit", "timeoutMs"];

const FIRMWARE: Vocabulary = Vocabulary {
    format: "firmware",
    noun: "a firmware",
    root: "firmware",
    tags: &[
        TagRule::new("firmware", &[]).attrs(FIRMWARE_FIELDS),
        TagRule::new("supports", &["firmware"]).key("revision").required(&["revision"]).attrs(&["revision"]),
        TagRule::new("channel", &["firmware"]).key("id").required(&["id"]),
    ],
    from_tree: firmware_of,
};

const ROLLOUT: Vocabulary = Vocabulary {
    format: "rollout",
    noun: "a firmware rollout",
    root: "rollout",
    tags: &[
        TagRule::new("rollout", &[]).required(&["firmware", "world", "channel", "canary", "waves"]).attrs(ROLLOUT_FIELDS),
        TagRule::new("health", &["rollout"]).key("id").required(&["id", "channel", "read"]).attrs(HEALTH_FIELDS),
        TagRule::new("target", &["rollout"]).key("unit").required(&["unit"]).attrs(&["unit"]),
    ],
    from_tree: rollout_of,
};

fn declare(root: El) -> Result<Declared, String> {
    declare_document(&root, if root.tag == "firmware" { &FIRMWARE } else { &ROLLOUT })
}

fn list_of(t: &DocTree, tag: &str) -> Vec<Json> {
    t.children.iter().filter(|c| c.tag == tag).map(DocTree::record).collect()
}

fn stated(v: Option<&Json>) -> bool {
    !matches!(v, None | Some(Json::Null)) && !matches!(v, Some(Json::Str(s)) if s.is_empty()) && !matches!(v, Some(Json::Arr(a)) if a.is_empty())
}

fn firmware_of(t: &DocTree) -> Result<Json, String> {
    if !FIRMWARE_RELEASE_FIELDS.iter().any(|k| stated(t.attr(k))) {
        if !stated(t.attr("name")) {
            return Err("firmware(): a firmware's declaration states its name; a release states version, image, sha256, sbom and device".into());
        }
        return Ok(with_children(t));
    }
    let missing: Vec<&str> = FIRMWARE_RELEASE_FIELDS.iter().copied().filter(|k| !stated(t.attr(k))).collect();
    if !missing.is_empty() {
        return Err(format!("firmware() needs {}: a release states all of {}", missing.join(", "), FIRMWARE_RELEASE_FIELDS.join(", ")));
    }
    let sha = t.attr("sha256");
    if !matches!(sha, Some(Json::Str(s)) if s.len() == 64 && s.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))) {
        let said = sha.map(Json::text).unwrap_or_else(|| "undefined".into());
        return Err(format!("firmware() sha256 is the image's SHA-256 in 64 lowercase hex digits, not {said}"));
    }
    Ok(with_children(t))
}

fn with_children(t: &DocTree) -> Json {
    let mut out = t.record();
    let (supports, channels) = (list_of(t, "supports"), list_of(t, "channel"));
    if !supports.is_empty() {
        out.set("supports", Json::Arr(supports));
    }
    if !channels.is_empty() {
        out.set("channels", Json::Arr(channels));
    }
    out
}

fn rollout_of(t: &DocTree) -> Result<Json, String> {
    let mut out = t.record();
    let (health, targets) = (list_of(t, "health"), list_of(t, "target"));
    if !health.is_empty() {
        out.set("health", Json::Arr(health));
    }
    if !targets.is_empty() {
        out.set("targets", Json::Arr(targets));
    }
    Ok(out)
}
