//! WHERE A DECLARATION WAS WRITTEN — the call site of each element, so an editor can write an edit back into the file.
//!
//! Each element constructor (`resistor("R1")`, `trace(…)`) is `#[track_caller]`: it knows the line and column of the
//! call that made it, and counts how many times that call ran (more than once: a loop, or a function called more than
//! once). A declaration puts the site exactly where the TypeScript SDK puts an element's `source` — a node's
//! `meta.source` ([`meta`]), a node's `meta.sources[<key>]` or a document's `sources[<key>]` ([`Sources`]) — as
//! `{"site": [line, column]}`; the run's answer lists every site with its count. The sandbox matches each site to the
//! call it is in the file's source map (computed by `syn` in `commandagi-map`) and replaces it with where that call
//! is and what its arguments are.

use super::element::El;
use super::json::Json;
use std::cell::RefCell;
use std::panic::Location;

/// A call site: line and column (1-based, the column in characters), as `Location` gives them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Site {
    pub line: u32,
    pub column: u32,
}

thread_local! {
    static COUNTS: RefCell<Vec<(Site, u32)>> = const { RefCell::new(Vec::new()) };
}

/// The site of the caller of the `#[track_caller]` function that calls this, counted once.
#[track_caller]
pub fn here() -> Site {
    let at = Location::caller();
    let site = Site { line: at.line(), column: at.column() };
    COUNTS.with(|c| {
        let mut c = c.borrow_mut();
        match c.iter_mut().find(|(s, _)| *s == site) {
            Some(entry) => entry.1 += 1,
            None => c.push((site, 1)),
        }
    });
    site
}

/// A node's `meta` for the element that declared it: `{"source": {"site": [l, c]}}`, or none (a fragment's).
pub fn meta(el: &El) -> Option<Json> {
    el.source_json().map(|s| Json::obj().with("source", s))
}

/// Sources by key (a node's `meta.sources`, a document's `sources`): each key the element it came from.
#[derive(Default)]
pub struct Sources(Vec<(String, Json)>);

impl Sources {
    pub fn new() -> Sources {
        Sources::default()
    }

    /// `key` came from `el` (nothing when `el` has no site).
    pub fn put(&mut self, key: impl Into<String>, el: &El) {
        if let Some(s) = el.source_json() {
            let key = key.into();
            self.0.retain(|(k, _)| *k != key);
            self.0.push((key, s));
        }
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn json(self) -> Json {
        Json::Obj(self.0)
    }
}

/// Every site that ran, with its count: `[[line, column, evaluations], …]`.
pub fn sites() -> Json {
    COUNTS.with(|c| Json::Arr(c.borrow().iter().map(|(s, n)| Json::Arr(vec![s.line.into(), s.column.into(), (*n).into()])).collect()))
}
