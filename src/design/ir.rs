//! The op-graph IR every `commandagi::design` declaration produces: plain JSON, the op graph every CommandAGI editor
//! stores. The same declarations give the same JSON as the TypeScript SDK's `commandagi/design` and the Python SDK's
//! `commandagi.design`.
//!
//! ```text
//! {"id", "nodes": {<id>: {"id", "type", "inputs", "label"?, "meta"?}}, "outputs"?, "meta"?}
//! ```
//!
//! A node has one map of ports, `inputs`: a literal, or one wire `{"wire": {"node", "port"}}`. Structure only.

use super::json::Json;
use std::collections::{HashMap, HashSet};

/// An id from any text: runs of characters other than `A-Z a-z 0-9 _ . -` become `_`, and `_` is trimmed.
pub fn slug(raw: &str) -> String {
    let mut out = String::new();
    let mut gap = false;
    for c in raw.chars() {
        if c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '-') {
            out.push(c);
            gap = false;
        } else if !gap {
            out.push('_');
            gap = true;
        }
    }
    let s = out.trim_matches('_');
    if s.is_empty() { "node".to_string() } else { s.to_string() }
}

/// FNV-1a (32 bits) over the UTF-16 code units of `s`, as eight hex digits (the TypeScript engine's rule).
pub fn fnv1a(s: &str) -> String {
    let mut h: u32 = 0x811c_9dc5;
    for unit in s.encode_utf16() {
        h ^= unit as u32;
        h = h.wrapping_mul(0x0100_0193);
    }
    format!("{h:08x}")
}

/// A wire to `port` of `node`.
pub fn wire(node: &str, port: &str) -> Json {
    Json::obj().with("wire", Json::obj().with("node", node).with("port", port))
}

/// Numbered channels of one set: `channels("ends", [a, b])` → `{"ends.1": a, "ends.2": b}`.
pub fn channels(set: &str, values: Vec<Json>) -> Vec<(String, Json)> {
    values.into_iter().enumerate().map(|(i, v)| (format!("{set}.{}", i + 1), v)).collect()
}

/// One graph being declared. Ids are deterministic (from the label or type, numbered on collision).
pub struct Scope {
    pub id: String,
    pub meta: Json,
    nodes: Vec<(String, Json)>,
    taken: HashSet<String>,
    next: HashMap<String, usize>,
}

impl Scope {
    pub fn new(id: &str, meta: Json) -> Scope {
        Scope { id: id.to_string(), meta, nodes: Vec::new(), taken: HashSet::new(), next: HashMap::new() }
    }

    pub fn has(&self, id: &str) -> bool {
        self.taken.contains(id)
    }

    pub fn unique_id(&mut self, base: &str) -> String {
        let b = slug(base);
        let mut n = *self.next.get(&b).unwrap_or(&2);
        let mut i = b.clone();
        while self.taken.contains(&i) {
            i = format!("{b}_{n}");
            n += 1;
        }
        self.next.insert(b, n);
        self.taken.insert(i.clone());
        i
    }

    /// Add a node: `id` (made a slug), else one from the label or the type. Two nodes of one id are refused.
    pub fn add(&mut self, kind: &str, inputs: Vec<(String, Json)>, id: Option<&str>, label: Option<&str>, meta: Option<Json>) -> Result<String, String> {
        let nid = match id {
            Some(id) => {
                let nid = slug(id);
                if self.taken.contains(&nid) {
                    return Err(format!("{}: two nodes are called {nid}", self.id));
                }
                self.taken.insert(nid.clone());
                nid
            }
            None => self.unique_id(label.unwrap_or(kind)),
        };
        let mut node = Json::obj().with("id", nid.as_str()).with("type", kind).with("inputs", Json::Obj(inputs));
        if let Some(label) = label {
            node.set("label", label);
        }
        if let Some(meta) = meta {
            node.set("meta", meta);
        }
        self.nodes.push((nid.clone(), node));
        Ok(nid)
    }

    /// The graph: `{id, nodes, meta}`.
    pub fn build(self) -> Json {
        Json::obj().with("id", self.id.as_str()).with("nodes", Json::Obj(self.nodes)).with("meta", self.meta)
    }
}
