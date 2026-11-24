//! A FILE'S OWN PARAMETERS — the inputs a `.rs` document declares, as `param(…)` is in a TypeScript or Python file.
//!
//! ```
//! use commandagi::design::params::param;
//! let width = param("width", 60.0).unit("mm").min(30.0).number();
//! assert_eq!(width, 60.0);
//! ```
//!
//! `param(name, default)` declares the input and returns a [`Param`]; its value is the sandbox's input of that name
//! when the run has one, else the default. The run's answer lists every declared param as
//! `{name: {default, unit, min, max, step, label, doc}}`, which is what the editor shows as the part's controls.

use super::json::Json;
use std::cell::RefCell;

thread_local! {
    static DECLARED: RefCell<Vec<(String, Json)>> = const { RefCell::new(Vec::new()) };
    static INPUTS: RefCell<Json> = const { RefCell::new(Json::Null) };
}

/// Set the run's inputs (the sandbox, through `abi::input_alloc`; a test): a JSON object of values by param name.
pub fn set_inputs(inputs: Json) {
    INPUTS.with(|i| *i.borrow_mut() = inputs);
}

/// Forget every declared param and input (a test between runs).
pub fn reset() {
    DECLARED.with(|d| d.borrow_mut().clear());
    set_inputs(Json::Null);
}

/// The declared params: `{name: {default, …}}`.
pub fn declared() -> Json {
    DECLARED.with(|d| Json::Obj(d.borrow().clone()))
}

/// One declared input.
pub struct Param {
    name: String,
}

/// Declare the input `name` with its default.
pub fn param(name: impl Into<String>, default: impl Into<Json>) -> Param {
    let name = name.into();
    let entry = Json::obj().with("default", default);
    DECLARED.with(|d| {
        let mut d = d.borrow_mut();
        match d.iter_mut().find(|(k, _)| *k == name) {
            Some(e) => e.1 = entry,
            None => d.push((name.clone(), entry)),
        }
    });
    Param { name }
}

impl Param {
    fn field(self, key: &str, value: impl Into<Json>) -> Param {
        DECLARED.with(|d| {
            if let Some((_, e)) = d.borrow_mut().iter_mut().find(|(k, _)| *k == self.name) {
                e.set(key, value);
            }
        });
        self
    }

    /// The unit the number is in (`"mm"`).
    pub fn unit(self, unit: &str) -> Param {
        self.field("unit", unit)
    }

    /// The name a control shows.
    pub fn label(self, label: &str) -> Param {
        self.field("label", label)
    }

    /// What the input is for.
    pub fn doc(self, doc: &str) -> Param {
        self.field("doc", doc)
    }

    pub fn min(self, min: f64) -> Param {
        self.field("min", min)
    }

    pub fn max(self, max: f64) -> Param {
        self.field("max", max)
    }

    pub fn step(self, step: f64) -> Param {
        self.field("step", step)
    }

    /// The value of this run: the input of this name, else the default.
    pub fn value(&self) -> Json {
        let given = INPUTS.with(|i| i.borrow().get(&self.name).cloned());
        given
            .or_else(|| DECLARED.with(|d| d.borrow().iter().find(|(k, _)| *k == self.name).and_then(|(_, e)| e.get("default").cloned())))
            .unwrap_or(Json::Null)
    }

    /// The value as a number (0 when it is not one).
    pub fn number(&self) -> f64 {
        self.value().as_f64().unwrap_or(0.0)
    }

    /// The value as text (empty when it is not text).
    pub fn text(&self) -> String {
        self.value().as_str().unwrap_or("").to_string()
    }

    /// The value as a switch (false when it is not one).
    pub fn flag(&self) -> bool {
        matches!(self.value(), Json::Bool(true))
    }
}
