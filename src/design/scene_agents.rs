//! AGENTS IN A SCENE — the `agents` a design's `slot().name("scene")` holds (docs/formats.md § simulations, "Agents in
//! a scene", in the CommandAGI repository): a model agent attached to robots of the scene, or to some joints of one (its
//! actuators). Each acts as `agent:<id>` through the simulation's own commands, and only on its `units`: that list is
//! its grant in the simulation, and it grants nothing on hardware. The same as the TypeScript SDK's (`scene-agents.ts`)
//! and the Python SDK's (`commandagi.design.scene_agents`):
//!
//! ```
//! use commandagi::design::scene_agents::*;
//! use commandagi::design::Json;
//!
//! let scene = Json::parse(r#"{ "robots": [{ "id": "left" }, { "id": "right" }],
//!   "agents": [{ "id": "pilot", "units": ["left"] }, { "id": "wrist", "units": ["right.wrist_flex"] }] }"#).unwrap();
//! check_scene_agents(&scene).unwrap();
//! ```
//!
//! An agent's `model` is `<route>/<model>` on a route of the agent runtime (openrouter, deepinfra, deepseek, dashscope);
//! absent, `deepseek/deepseek-flash`.
//!
//! `check_scene_agents` refuses what the engine refuses, in its words: an unknown field, a malformed or repeated id, an
//! id that is a policy's principal, a unit that is no robot of the scene nor a joint of one, an empty text.

use super::Json;

/// An entry's fields: the ones it needs, then the ones it may have. One table, the same in the three SDKs.
pub const SCENE_AGENT_FIELDS: (&[&str], &[&str]) = (&["id", "units"], &["model", "instructions", "label"]);

fn ids(scene: &Json, key: &str) -> Vec<String> {
    match scene.get(key) {
        Some(Json::Arr(items)) => items.iter().filter_map(|x| x.get("id").and_then(Json::as_str).map(String::from)).collect(),
        _ => Vec::new(),
    }
}
fn good_id(s: &str) -> bool {
    let mut c = s.chars();
    c.next().is_some_and(|f| f.is_ascii_lowercase() || f.is_ascii_digit()) && s.len() <= 48 && s.chars().all(|x| x.is_ascii_lowercase() || x.is_ascii_digit() || x == '-')
}

/// Refuses what the engine would refuse, with its words. It reads the scene's `robots`, `policies` and `agents`.
pub fn check_scene_agents(scene: &Json) -> Result<(), String> {
    let agents = match scene.get("agents") {
        None => return Ok(()),
        Some(Json::Arr(items)) => items,
        Some(_) => return Err("simulation: agents is a list".into()),
    };
    let (robots, policies) = (ids(scene, "robots"), ids(scene, "policies"));
    let (needs, may) = SCENE_AGENT_FIELDS;
    let fields = [needs, may].concat();
    let mut seen: Vec<String> = Vec::new();
    for a in agents {
        let id = a.get("id").and_then(Json::as_str).unwrap_or("");
        if !good_id(id) || seen.iter().any(|x| x == id) {
            let named = match a.get("id") { Some(Json::Str(s)) => s.clone(), Some(other) => other.text(), None => "undefined".into() };
            return Err(format!("agent {named}: missing, malformed or duplicate id (lower case letters, digits and -)"));
        }
        seen.push(id.to_string());
        let no = |why: String| Err(format!("agent {id}: {why}"));
        let Json::Obj(entries) = a else { return no("an agent is an object".into()) };
        for (k, _) in entries {
            if !fields.contains(&k.as_str()) {
                return no(format!("unknown field {k} (it has {})", fields.join(", ")));
            }
        }
        if policies.iter().any(|p| format!("policy-{p}") == id) {
            return no(format!("agent:{id} is the principal of a policy of this scene"));
        }
        let units = match a.get("units") {
            Some(Json::Arr(items)) if !items.is_empty() => items,
            _ => return no(r#"units is a list of the robots it drives ("arm") or their joints ("arm.j1")"#.into()),
        };
        for u in units {
            let ok = u.as_str().is_some_and(|s| {
                let mut parts = s.splitn(2, '.');
                let robot = parts.next().unwrap_or("");
                !robot.is_empty() && robots.iter().any(|r| r == robot) && parts.next().is_none_or(|joint| !joint.is_empty())
            });
            if !ok {
                return no(format!("unit {} is not a robot of this scene or a joint of one", u.text()));
            }
        }
        for k in ["model", "instructions", "label"] {
            if let Some(v) = a.get(k) {
                if v.as_str().is_none_or(str::is_empty) {
                    return no(format!("{k} is a text"));
                }
            }
        }
    }
    Ok(())
}
