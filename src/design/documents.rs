//! DOCUMENTS AS ELEMENTS — a JSON document of CommandAGI (a world, a dashboard, a task, a machining setup …) written
//! as a tree of elements: the Rust form of the TypeScript SDK's `documents.ts`.
//!
//! Each element is one record of the document, and its attributes are that record's fields, verbatim
//! (`unit().uid("arm").position([0, 0, 0])` is the unit `{"uid": "arm", "position": [0, 0, 0]}`). A [`Vocabulary`]
//! says which tags a document has, where each may stand, which attribute names a record among its siblings, and how
//! the tree maps to the document's JSON.
//!
//! What a declaration gives is the one shape of a document that is not a graph ([`Declared::Document`]): the format,
//! the document's JSON, and each element's site keyed by its path ([`tree_paths`]: `""` the root, `unit#arm`,
//! `scene/body#cube`), the keys TypeScript gives. Structure only: the application that reads the document checks
//! what it means.

use super::element::{fn_name, rust_name, Child, El};
use super::json::Json;
use super::Declared;

/// One element of a document: its tag, its attributes (the record's fields), its children, and its site.
#[derive(Clone, Debug)]
pub struct DocTree {
    pub tag: &'static str,
    pub attrs: Vec<(String, Json)>,
    pub children: Vec<DocTree>,
    /// `{"site": [line, column]}`; none for a fragment's.
    pub source: Option<Json>,
}

impl DocTree {
    pub fn attr(&self, name: &str) -> Option<&Json> {
        self.attrs.iter().find(|(k, _)| k == name).map(|(_, v)| v)
    }

    /// The attributes as one JSON object (a record).
    pub fn record(&self) -> Json {
        Json::Obj(self.attrs.clone())
    }

    /// The first child of `tag`.
    pub fn single(&self, tag: &str) -> Option<&DocTree> {
        self.children.iter().find(|c| c.tag == tag)
    }
}

/// Where a tag may stand, and how its records are told apart.
pub struct TagRule {
    pub tag: &'static str,
    /// The tags it may be a child of (none: it is the root).
    pub parents: &'static [&'static str],
    /// The attribute that names one record among its siblings of this tag (`uid`, `id`).
    pub key: Option<&'static str>,
    /// At most one per parent.
    pub single: bool,
    /// Attributes it must have.
    pub required: &'static [&'static str],
    /// The attributes it may have; none: any.
    pub attrs: Option<&'static [&'static str]>,
}

impl TagRule {
    /// A tag that stands in `parents`, of any attributes, none required.
    pub const fn new(tag: &'static str, parents: &'static [&'static str]) -> TagRule {
        TagRule { tag, parents, key: None, single: false, required: &[], attrs: None }
    }
    pub const fn key(mut self, key: &'static str) -> TagRule {
        self.key = Some(key);
        self
    }
    pub const fn single(mut self) -> TagRule {
        self.single = true;
        self
    }
    pub const fn required(mut self, required: &'static [&'static str]) -> TagRule {
        self.required = required;
        self
    }
    pub const fn attrs(mut self, attrs: &'static [&'static str]) -> TagRule {
        self.attrs = Some(attrs);
        self
    }
}

/// A document format written as elements: its tags, and the JSON a tree declares.
pub struct Vocabulary {
    /// The format's name (a declared document's `format`).
    pub format: &'static str,
    /// What a person calls it ("a world").
    pub noun: &'static str,
    pub root: &'static str,
    pub tags: &'static [TagRule],
    /// The document a tree declares. Adds nothing the tree does not say: no defaults.
    pub from_tree: fn(&DocTree) -> Result<Json, String>,
}

impl Vocabulary {
    pub fn rule(&self, tag: &str) -> Option<&TagRule> {
        self.tags.iter().find(|r| r.tag == tag)
    }
}

/// How a message names an element: its call, with the attribute that names it (`operation().id("a")`).
fn call(el: &El, key: Option<&str>) -> String {
    match key.and_then(|k| el.attr(k).map(|v| (k, v))) {
        Some((k, v @ Json::Str(_))) => format!("{}().{}({})", fn_name(el.tag), rust_name(k), v.text()),
        _ => format!("{}()", fn_name(el.tag)),
    }
}

/// A value is plain data with finite numbers (`f64::NAN` is refused, as TypeScript refuses `NaN`).
fn plain(v: &Json, what: &str) -> Result<(), String> {
    match v {
        Json::Num(n) if !n.is_finite() => Err(format!("{what} is {n}, not a finite number")),
        Json::Arr(items) => items.iter().enumerate().try_for_each(|(i, x)| plain(x, &format!("{what}[{i}]"))),
        Json::Obj(entries) => entries.iter().try_for_each(|(k, x)| plain(x, &format!("{what}.{k}"))),
        _ => Ok(()),
    }
}

/// JavaScript's `String(v)` for a key's value.
pub fn js_string(v: Option<&Json>) -> String {
    match v {
        None => "undefined".into(),
        Some(Json::Str(s)) => s.clone(),
        Some(Json::Arr(items)) => items.iter().map(|x| if *x == Json::Null { String::new() } else { js_string(Some(x)) }).collect::<Vec<_>>().join(","),
        Some(Json::Obj(_)) => "[object Object]".into(),
        Some(other) => other.text(),
    }
}

/// The tree an element declares, checked against the vocabulary's tags (the root's rule has no parents).
pub fn tree_of_element(root: &El, v: &Vocabulary) -> Result<DocTree, String> {
    fn visit(el: &El, parent: Option<&'static str>, v: &Vocabulary) -> Result<DocTree, String> {
        let Some(rule) = v.rule(el.tag) else {
            let tags: Vec<String> = v.tags.iter().map(|r| format!("{}()", fn_name(r.tag))).collect();
            return Err(format!("{}() is not an element of {} ({})", fn_name(el.tag), v.noun, tags.join(", ")));
        };
        match parent {
            None if !rule.parents.is_empty() => return Err(format!("{} in code is one {}() element, not {}()", v.noun, fn_name(v.root), fn_name(el.tag))),
            Some(p) if !rule.parents.contains(&p) => {
                let ps: Vec<String> = rule.parents.iter().map(|p| format!("{}()", fn_name(p))).collect();
                return Err(format!("{}() stands in {}, not in {}()", fn_name(el.tag), ps.join(" or "), fn_name(p)));
            }
            _ => {}
        }
        let at = call(el, rule.key);
        for (k, value) in &el.attrs {
            if let Some(allowed) = rule.attrs {
                if !allowed.contains(&k.as_str()) {
                    let names: Vec<String> = allowed.iter().map(|a| rust_name(a)).collect();
                    return Err(format!("{at}: {} is not read ({})", rust_name(k), names.join(", ")));
                }
            }
            plain(value, &format!("{at} {}", rust_name(k)))?;
        }
        for k in rule.required {
            if el.attr(k).is_none() {
                return Err(format!("{at} needs {}", rust_name(k)));
            }
        }
        for c in &el.children {
            if let Child::Text(t) = c {
                if !t.trim().is_empty() {
                    return Err(format!("{at}: text is not read ({:?})", t.trim().chars().take(40).collect::<String>()));
                }
            }
        }
        let children = el.child_elements().map(|c| visit(c, Some(el.tag), v)).collect::<Result<Vec<_>, _>>()?;
        let mut seen: Vec<String> = Vec::new();
        for c in &children {
            let r = v.rule(c.tag).expect("a child's tag was checked");
            // A record whose key is not written is told apart by its place (as `identities` does).
            let id = match (r.key.filter(|key| c.attr(key).is_some()), r.single) {
                (Some(key), _) => format!("{}#{}", c.tag, js_string(c.attr(key))),
                (None, true) => c.tag.to_string(),
                _ => continue,
            };
            if seen.contains(&id) {
                return Err(match r.key.filter(|key| c.attr(key).is_some()) {
                    Some(key) => format!("two {}() in {}() have {} {:?}", fn_name(c.tag), fn_name(el.tag), rust_name(key), js_string(c.attr(key))),
                    None => format!("{}() has one {}()", fn_name(el.tag), fn_name(c.tag)),
                });
            }
            seen.push(id);
        }
        Ok(DocTree { tag: el.tag, attrs: el.attrs.clone(), children, source: el.source_json() })
    }
    visit(root, None, v)
}

/// A record's identity among its siblings: `unit#arm` by its key, `space` when single, else `split@1` by position.
pub fn identities(children: &[DocTree], v: &Vocabulary) -> Vec<String> {
    let mut nth: Vec<(&str, usize)> = Vec::new();
    children
        .iter()
        .map(|c| {
            let r = v.rule(c.tag);
            if let Some(k) = r.and_then(|r| r.key).and_then(|key| c.attr(key)) {
                return format!("{}#{}", c.tag, k.as_str().map(str::to_string).unwrap_or_else(|| k.text()));
            }
            let i = match nth.iter_mut().find(|(t, _)| *t == c.tag) {
                Some(entry) => {
                    entry.1 += 1;
                    entry.1 - 1
                }
                None => {
                    nth.push((c.tag, 1));
                    0
                }
            };
            if r.is_some_and(|r| r.single) {
                c.tag.to_string()
            } else {
                format!("{}@{i}", c.tag)
            }
        })
        .collect()
}

/// Visit each element of a tree with its path: the identities from the root down, joined by `/` (`""` is the root).
pub fn tree_paths(tree: &DocTree, v: &Vocabulary, visit: &mut dyn FnMut(&DocTree, &str)) {
    fn walk(t: &DocTree, path: &str, v: &Vocabulary, visit: &mut dyn FnMut(&DocTree, &str)) {
        visit(t, path);
        let ids = identities(&t.children, v);
        for (c, id) in t.children.iter().zip(ids) {
            let p = if path.is_empty() { id } else { format!("{path}/{id}") };
            walk(c, &p, v, visit);
        }
    }
    walk(tree, "", v, visit);
}

/// Declare a document from its root element: the format, the document, and each element's site by its path.
pub fn declare_document(root: &El, v: &Vocabulary) -> Result<Declared, String> {
    let tree = tree_of_element(root, v)?;
    let document = (v.from_tree)(&tree)?;
    let mut sources = Json::obj();
    tree_paths(&tree, v, &mut |t, path| {
        if let Some(s) = &t.source {
            sources.set(path, s.clone());
        }
    });
    Ok(Declared::Document { format: v.format.to_string(), document, sources })
}
