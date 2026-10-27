//! TEST PLANS IN RUST — a test plan (`<name>.testplan.rs`) holds the steps that test one serial-numbered unit, never
//! the results: the same elements, and the same document, as the TypeScript SDK's JSX (`testplans.ts`) and the Python
//! SDK's (`commandagi.design.testplans`), one call each:
//!
//! ```
//! use commandagi::design::testplans::*;
//!
//! fn document() -> El {
//!     testplan([
//!         condition().id("ambient").quantity("temperature").min(15).max(35).unit("C"),
//!         fixture().id("supply").kind("instrument").unit("psu-1").channel("output").note("5 V, current limit 1 A"),
//!         step().id("rail-3v3").channel("power").command(json(r#"{"verb": "read"}"#)).read("v3v3").min(3.2).max(3.4).unit("V")
//!             .verifies(["requirements/power.req.tsx#rail-3v3"]),
//!         step().id("supply-current").fixture("supply").command(json(r#"{"verb": "read"}"#)).read("current").max(0.4).unit("A"),
//!     ])
//!     .title("Cam-001 power rail")
//!     .device("../devices/commandagi-cam-001/definition.tsx")
//! }
//! # commandagi::design::declare(document()).unwrap();
//! ```
//!
//! A record is an element and its fields are its attributes (methods in snake case of the native names: `.settle_ms`
//! is `settleMs`). A step reads the unit's channel (`channel`) or an instrument's (`fixture`). `command` is one record
//! in the channel format's shape, sent first; `read` is a value of the state the channel reports after it. Steps keep
//! their order. Nothing adds a default: the runner says what it assumes.
//!
//! The document a test plan declares:
//!
//! ```text
//! { title?, device?, note?, conditions?: [{ id, quantity, value?, min?, max?, unit?, note? }], fixtures?: [{ id, kind,
//!   unit?, channel?, note? }], steps?: [{ id, channel?, fixture?, command?, read?, min?, max?, equals?, unit?,
//!   settleMs?, timeoutMs?, condition?, verifies?, note? }] }
//! ```

use super::documents::{declare_document, DocTree, TagRule, Vocabulary};
use super::element::{attributes, elements};
use super::{Declared, Family};
pub use super::{fragment, json, El, Json, Value};

elements! {
    /// A test plan: its conditions, fixtures and steps. `.title(…).device(…)`.
    testplan: children;
    /// A condition the test needs: `condition().id("ambient").quantity("temperature").min(15).max(35).unit("C")`.
    condition: leaf;
    /// A jig or an instrument: `fixture().id("supply").kind("instrument").unit("psu-1").channel("output")`.
    fixture: leaf;
    /// One step: what to command, what to read, the limits and the requirements it verifies.
    step: leaf;
}

attributes! {
    /// The fields of a test plan's records, chained on its elements.
    pub trait TestPlanAttrs {
        title; device; note;
        id; quantity; value; min; max; unit; kind; channel;
        /// A step's instrument: the id of a fixture that names a unit of the same world.
        fixture;
        /// One record in the channel format's shape, sent first: `json(r#"{"verb": "read"}"#)`.
        command;
        /// A value of the state the channel reports.
        read;
        equals; settle_ms; timeout_ms;
        /// The id of the condition the step holds to.
        condition;
        /// Refs to the requirements the step verifies: `["requirements/power.req.tsx#rail-3v3"]`.
        verifies;
    }
}

pub(crate) const FAMILY: Family = Family { module: "testplans", roots: &["testplan"], declare };

pub const TESTPLAN_FIELDS: &[&str] = &["title", "device", "note"];
pub const CONDITION_FIELDS: &[&str] = &["id", "quantity", "value", "min", "max", "unit", "note"];
pub const FIXTURE_FIELDS: &[&str] = &["id", "kind", "unit", "channel", "note"];
pub const STEP_FIELDS: &[&str] =
    &["id", "channel", "fixture", "command", "read", "min", "max", "equals", "unit", "settleMs", "timeoutMs", "condition", "verifies", "note"];

const LISTS: &[(&str, &str)] = &[("condition", "conditions"), ("fixture", "fixtures"), ("step", "steps")];

const TESTPLAN: Vocabulary = Vocabulary {
    format: "testplan",
    noun: "a test plan",
    root: "testplan",
    tags: &[
        TagRule::new("testplan", &[]).attrs(TESTPLAN_FIELDS),
        TagRule::new("condition", &["testplan"]).key("id").required(&["id", "quantity"]).attrs(CONDITION_FIELDS),
        TagRule::new("fixture", &["testplan"]).key("id").required(&["id", "kind"]).attrs(FIXTURE_FIELDS),
        TagRule::new("step", &["testplan"]).key("id").required(&["id"]).attrs(STEP_FIELDS),
    ],
    from_tree: testplan_of,
};

fn declare(root: El) -> Result<Declared, String> {
    declare_document(&root, &TESTPLAN)
}

fn testplan_of(t: &DocTree) -> Result<Json, String> {
    let mut out = t.record();
    for (tag, field) in LISTS {
        let items: Vec<Json> = t.children.iter().filter(|c| c.tag == *tag).map(DocTree::record).collect();
        if !items.is_empty() {
            out.set(field, Json::Arr(items));
        }
    }
    for s in t.children.iter().filter(|c| c.tag == "step") {
        let at = match s.attr_of("id") {
            Some(id) => format!("step().id({})", id.text()),
            None => "step()".into(),
        };
        if let Some(v) = s.attr_of("verifies") {
            let refs = matches!(v, Json::Arr(items) if items.iter().all(|r| matches!(r, Json::Str(s) if !s.is_empty())));
            if !refs {
                return Err(format!("{at} verifies is a list of refs to requirements, not {}", v.text()));
            }
        }
        if let Some(v) = s.attr_of("command") {
            if !matches!(v, Json::Obj(_)) {
                return Err(format!("{at} command is one record in the channel format's shape ({{ verb: … }}), not {}", v.text()));
            }
        }
    }
    Ok(out)
}
