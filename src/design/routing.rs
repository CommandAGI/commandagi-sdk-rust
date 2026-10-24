//! Routing knowledge: priors, hard rules, prices and measured observations.
use super::documents::{declare_document, DocTree, TagRule, Vocabulary};
use super::element::{attributes, elements};
use super::{Declared, Family};
pub use super::{fragment, json, El, Json, Value};

elements! { routing: children; prior: leaf; rule: leaf; price: leaf; observation: leaf; }
attributes! { pub trait RoutingAttrs { id; region; text; config; harness; model; quality; importance; cost; currency; channel; billing; weight; count; date; source; evidence; reported; checked; task; uncensored; sensitive; deny; reason; input_per_million; output_per_million; tokens_per_task; budget; tasks_per_usd; lines_per_usd; done; blocked; lines_merged; spend; } }

pub const ROUTING_FIELDS: &[&str] = &["id", "region", "text", "config", "harness", "model", "quality", "importance", "cost", "currency", "channel", "billing", "weight", "count", "date", "source", "evidence", "reported", "checked", "task", "uncensored", "sensitive", "deny", "reason", "inputPerMillion", "outputPerMillion", "tokensPerTask", "budget", "tasksPerUsd", "linesPerUsd", "done", "blocked", "linesMerged", "spend"];
const TAGS: &[TagRule] = &[
    TagRule::new("routing", &[]).attrs(&[]),
    TagRule::new("prior", &["routing"]).key("id").required(&["id"]).attrs(ROUTING_FIELDS),
    TagRule::new("rule", &["routing"]).key("id").required(&["id"]).attrs(ROUTING_FIELDS),
    TagRule::new("price", &["routing"]).key("id").required(&["id"]).attrs(ROUTING_FIELDS),
    TagRule::new("observation", &["routing"]).key("id").required(&["id"]).attrs(ROUTING_FIELDS),
];
const VOCABULARY: Vocabulary = Vocabulary { format: "routing", noun: "routing knowledge", root: "routing", tags: TAGS, from_tree: routing_of };
pub(crate) const FAMILY: Family = Family { module: "routing", roots: &["routing"], declare };

fn declare(root: El) -> Result<Declared, String> { declare_document(&root, &VOCABULARY) }
fn routing_of(t: &DocTree) -> Result<Json, String> {
    let mut out = Json::obj();
    for (tag, field) in [("prior", "priors"), ("rule", "rules"), ("price", "prices"), ("observation", "observations")] {
        let mut rows = Vec::new();
        let mut seen = Vec::<String>::new();
        for child in t.children.iter().filter(|c| c.tag == tag) {
            let Some(Json::Str(id)) = child.attr("id") else { return Err(format!("<{tag}> id is required")); };
            if id.trim().is_empty() || seen.contains(id) { return Err(format!("duplicate or empty <{tag}> id {id:?}")); }
            seen.push(id.clone());
            rows.push(child.record());
        }
        out.set(field, Json::Arr(rows));
    }
    Ok(out)
}
