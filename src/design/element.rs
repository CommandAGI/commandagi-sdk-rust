//! THE ONE ELEMENT TYPE — every document family is a tree of [`El`]: a tag (the TypeScript SDK's JSX tag), attributes
//! by their TypeScript names in the order they were written, children (elements, and texts for a tag that holds text),
//! and the call site that made it. The family modules declare their constructors and their attribute traits with the
//! two macros of this file (`elements!`, `attributes!`); `super` (design/mod.rs) states the conventions.

use super::json::Json;
use super::source::{here, Site};

/// One element: what one constructor call wrote, with the methods chained on it.
///
/// Its inherent methods are few and never an attribute's name, so a family's attribute trait is never shadowed.
#[derive(Clone, Debug)]
pub struct El {
    /// The TypeScript SDK's tag (`resistor`, `brush-stroke`, `Company`).
    pub tag: &'static str,
    /// The attributes by TypeScript name (`schX`), in the order they were written.
    pub attrs: Vec<(String, Json)>,
    /// How many of the first attributes the constructor took by position (the schematic's `name`, `from`, `to`).
    pub positional: usize,
    /// Elements and texts, in order. A fragment is flattened into the element that holds it.
    pub children: Vec<Child>,
    /// The constructor's call site; none for a fragment.
    pub site: Option<Site>,
    /// Values refused when they were written (invalid JSON text): the declaration refuses the first one by name.
    pub refused: Vec<String>,
}

/// A child of an element: an element, or a text of a tag that holds text.
#[derive(Clone, Debug)]
pub enum Child {
    El(El),
    Text(String),
}

impl El {
    /// An element of `tag`, at its caller's site (every constructor is `#[track_caller]`, so this is the file's call).
    #[track_caller]
    pub fn new(tag: &'static str) -> El {
        El { tag, attrs: Vec::new(), positional: 0, children: Vec::new(), site: Some(here()), refused: Vec::new() }
    }

    /// Set attribute `name` (a TypeScript name). Written twice, the last value wins in the first one's place.
    pub fn with_attr(mut self, name: impl Into<String>, value: impl Into<Value>) -> El {
        let name = name.into();
        let value = match value.into().0 {
            Ok(v) => v,
            Err(why) => {
                self.refused.push(format!("{} is JSON text ({why})", rust_name(&name)));
                Json::Null
            }
        };
        match self.attrs.iter_mut().find(|(k, _)| *k == name) {
            Some(entry) => entry.1 = value,
            None => self.attrs.push((name, value)),
        }
        self
    }

    /// Append children; a fragment's children take its place.
    pub fn with_children(mut self, children: impl IntoIterator<Item = Child>) -> El {
        fn push(out: &mut Vec<Child>, c: Child) {
            match c {
                Child::El(el) if el.tag == FRAGMENT => el.children.into_iter().for_each(|c| push(out, c)),
                Child::Text(t) if t.is_empty() => {}
                c => out.push(c),
            }
        }
        for c in children {
            push(&mut self.children, c);
        }
        self
    }

    /// The value of attribute `name` (a TypeScript name).
    pub fn attr(&self, name: &str) -> Option<&Json> {
        self.attrs.iter().find(|(k, _)| k == name).map(|(_, v)| v)
    }

    /// The child elements, in order (texts left out).
    pub fn child_elements(&self) -> impl Iterator<Item = &El> {
        self.children.iter().filter_map(|c| match c {
            Child::El(el) => Some(el),
            Child::Text(_) => None,
        })
    }

    /// The call as a person reads it in a message: `resistor("R1")`, `unit("U1", 2)`, `layer()`.
    pub fn call_text(&self) -> String {
        let args: Vec<String> = self.attrs.iter().take(self.positional).map(|(_, v)| v.text()).collect();
        format!("{}({})", fn_name(self.tag), args.join(", "))
    }

    /// What TypeScript puts in a `source` for this element: `{"site": [line, column]}`, or none.
    pub fn source_json(&self) -> Option<Json> {
        self.site.map(|s| Json::obj().with("site", vec![Json::from(s.line), Json::from(s.column)]))
    }

    /// The first value refused anywhere in the tree, by the call that wrote it.
    pub fn refusal(&self) -> Result<(), String> {
        if let Some(why) = self.refused.first() {
            return Err(format!("{}: {why}", self.call_text()));
        }
        self.child_elements().try_for_each(El::refusal)
    }
}

/// The tag of a fragment: several elements as one item of a children list, React's fragment.
pub const FRAGMENT: &str = "fragment";

/// Several elements as one item of a children list: `fragment((1..=4).map(|i| resistor(format!("R{i}"))))`.
/// The elements a closure or a loop makes are the code's to edit; the list around them stays the editor's.
pub fn fragment(children: impl IntoIterator<Item = El>) -> El {
    El { tag: FRAGMENT, attrs: Vec::new(), positional: 0, children: Vec::new(), site: None, refused: Vec::new() }
        .with_children(children.into_iter().map(Child::El))
}

// ── Values ─────────────────────────────────────────────────────────────────────────────────────────────────────────

/// An attribute's value: a number, a string, a bool, a homogeneous array of these (nested), or JSON text ([`json`]).
#[derive(Clone, Debug)]
pub struct Value(pub(crate) Result<Json, String>);

/// JSON text as a value: `.excitation(json(r#"{"acMagnitude": 1}"#))`. Text that is not JSON is refused, by the
/// attribute's name, when the document is declared.
pub fn json(text: &str) -> Value {
    Value(Json::parse(text))
}

macro_rules! value_from_number {
    ($($t:ty),*) => {$(
        impl From<$t> for Value {
            fn from(n: $t) -> Value {
                Value(Ok(Json::Num(n as f64)))
            }
        }
    )*};
}
value_from_number!(i8, i16, i32, i64, u8, u16, u32, u64, usize, isize, f64);

impl From<f32> for Value {
    /// The shortest decimal that reads back as the same `f32` (`0.1f32` is 0.1, not 0.10000000149011612).
    fn from(n: f32) -> Value {
        Value(Ok(Json::Num(n.to_string().parse::<f64>().unwrap_or(n as f64))))
    }
}
impl From<&str> for Value {
    fn from(s: &str) -> Value {
        Value(Ok(Json::from(s)))
    }
}
impl From<String> for Value {
    fn from(s: String) -> Value {
        Value(Ok(Json::from(s)))
    }
}
impl From<&String> for Value {
    fn from(s: &String) -> Value {
        Value(Ok(Json::from(s.as_str())))
    }
}
impl From<bool> for Value {
    fn from(b: bool) -> Value {
        Value(Ok(Json::Bool(b)))
    }
}
impl From<Json> for Value {
    fn from(j: Json) -> Value {
        Value(Ok(j))
    }
}
impl<T: Into<Value>, const N: usize> From<[T; N]> for Value {
    fn from(items: [T; N]) -> Value {
        array(items)
    }
}
impl<T: Into<Value>> From<Vec<T>> for Value {
    fn from(items: Vec<T>) -> Value {
        array(items)
    }
}
impl<T: Into<Value> + Clone> From<&[T]> for Value {
    fn from(items: &[T]) -> Value {
        array(items.iter().cloned())
    }
}

fn array<T: Into<Value>>(items: impl IntoIterator<Item = T>) -> Value {
    Value(items.into_iter().map(|v| v.into().0).collect::<Result<Vec<_>, _>>().map(Json::Arr))
}

// ── Children and text parameters ───────────────────────────────────────────────────────────────────────────────────

/// What a text-holding tag takes: `p("The run passed.")`, `p(("The run ", b("passed"), "."))`, `p(b("x"))`,
/// `p([b("x"), i("y")])`, `p(vec![…])`; empty: `p("")` or `p([])`.
pub trait Text {
    fn into_children(self) -> Vec<Child>;
}

/// One item of a text tuple: a text or an element.
pub trait TextPart {
    fn into_child(self) -> Child;
}

impl TextPart for &str {
    fn into_child(self) -> Child {
        Child::Text(self.to_string())
    }
}
impl TextPart for String {
    fn into_child(self) -> Child {
        Child::Text(self)
    }
}
impl TextPart for &String {
    fn into_child(self) -> Child {
        Child::Text(self.clone())
    }
}
impl TextPart for El {
    fn into_child(self) -> Child {
        Child::El(self)
    }
}

macro_rules! text_from_part {
    ($($t:ty),*) => {$(
        impl Text for $t {
            fn into_children(self) -> Vec<Child> {
                vec![self.into_child()]
            }
        }
    )*};
}
text_from_part!(&str, String, &String, El);
impl<const N: usize> Text for [El; N] {
    fn into_children(self) -> Vec<Child> {
        self.into_iter().map(Child::El).collect()
    }
}
impl Text for Vec<El> {
    fn into_children(self) -> Vec<Child> {
        self.into_iter().map(Child::El).collect()
    }
}

macro_rules! text_tuples {
    ($(($($p:ident),+)),*) => {$(
        impl<$($p: TextPart),+> Text for ($($p,)+) {
            #[allow(non_snake_case)]
            fn into_children(self) -> Vec<Child> {
                let ($($p,)+) = self;
                vec![$($p.into_child()),+]
            }
        }
    )*};
}
text_tuples!(
    (A),
    (A, B),
    (A, B, C),
    (A, B, C, D),
    (A, B, C, D, E),
    (A, B, C, D, E, F),
    (A, B, C, D, E, F, G),
    (A, B, C, D, E, F, G, H),
    (A, B, C, D, E, F, G, H, I),
    (A, B, C, D, E, F, G, H, I, J),
    (A, B, C, D, E, F, G, H, I, J, K),
    (A, B, C, D, E, F, G, H, I, J, K, L),
    (A, B, C, D, E, F, G, H, I, J, K, L, M),
    (A, B, C, D, E, F, G, H, I, J, K, L, M, N),
    (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O),
    (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P)
);

// ── Names ──────────────────────────────────────────────────────────────────────────────────────────────────────────

const KEYWORDS: &[&str] = &[
    "as", "async", "await", "box", "break", "const", "continue", "crate", "do", "dyn", "else", "enum", "extern", "false", "final", "fn", "for",
    "gen", "if", "impl", "in", "let", "loop", "macro", "match", "mod", "move", "mut", "override", "priv", "pub", "ref", "return", "self",
    "static", "struct", "super", "trait", "true", "try", "type", "typeof", "unsafe", "unsized", "use", "virtual", "where", "while", "yield",
];

/// The TypeScript name of a Rust method or parameter: `sch_x` → `schX`, `type_` → `type` (each `_x` is an uppercase X).
pub fn ts_name(rust: &str) -> String {
    let mut out = String::new();
    let mut up = false;
    for c in rust.trim_end_matches('_').chars() {
        if c == '_' {
            up = true;
        } else if up {
            out.extend(c.to_uppercase());
            up = false;
        } else {
            out.push(c);
        }
    }
    out
}

fn snake(ts: &str, first_upper_joins: bool) -> String {
    let mut out = String::new();
    for (i, c) in ts.chars().enumerate() {
        if c == '-' {
            out.push('_');
        } else if c.is_uppercase() {
            if i > 0 || !first_upper_joins {
                out.push('_');
            }
            out.extend(c.to_lowercase());
        } else {
            out.push(c);
        }
    }
    if KEYWORDS.contains(&out.as_str()) {
        out.push('_');
    }
    out
}

/// The Rust method of a TypeScript attribute name: `schX` → `sch_x`, `type` → `type_` (what a message names).
pub fn rust_name(ts: &str) -> String {
    snake(ts, false)
}

/// The constructor of a TypeScript tag: `brush-stroke` → `brush_stroke`, `CapTable` → `cap_table`, `move` → `move_`.
pub fn fn_name(tag: &str) -> String {
    snake(tag, true)
}

/// A constructor's tag: the one written in `elements!`, else the function's name without a keyword's `_`.
pub fn tag_of(function: &'static str, written: Option<&'static str>) -> &'static str {
    written.unwrap_or_else(|| function.strip_suffix('_').unwrap_or(function))
}

// ── The two macros a family module declares itself with ───────────────────────────────────────────────────────────

/// A family's constructors, one per tag:
///
/// ```text
/// elements! {
///     /// An ideal resistor.
///     resistor(name): leaf;                  // fn resistor(name: impl Into<Value>) -> El
///     group(name): children;                 // fn group(name, children: impl IntoIterator<Item = El>) -> El
///     p: text;                               // fn p(text: impl Text) -> El
///     brush_stroke = "brush-stroke": leaf;   // a tag that is not the function's name
/// }
/// ```
///
/// The parenthesised names are the tag's positional attributes (TypeScript names by [`ts_name`]); only the schematic
/// has them. Every constructor is `#[track_caller]`, so the element knows the file's call.
macro_rules! elements {
    ($( $(#[$doc:meta])* $f:ident $(= $tag:literal)? $(( $($arg:ident),* ))? : $holds:ident; )*) => {
        $( elements!(@one [$(#[$doc])*] $f [$($tag)?] [$($($arg)*)?] $holds); )*
    };
    (@one [$($doc:tt)*] $f:ident [$($tag:literal)?] [$($arg:ident)*] leaf) => {
        $($doc)*
        #[track_caller]
        pub fn $f($($arg: impl Into<$crate::design::Value>),*) -> $crate::design::El {
            elements!(@new $f [$($tag)?] [$($arg)*])
        }
    };
    (@one [$($doc:tt)*] $f:ident [$($tag:literal)?] [$($arg:ident)*] children) => {
        $($doc)*
        #[track_caller]
        pub fn $f($($arg: impl Into<$crate::design::Value>,)* children: impl IntoIterator<Item = $crate::design::El>) -> $crate::design::El {
            elements!(@new $f [$($tag)?] [$($arg)*]).with_children(children.into_iter().map($crate::design::Child::El))
        }
    };
    (@one [$($doc:tt)*] $f:ident [$($tag:literal)?] [$($arg:ident)*] text) => {
        $($doc)*
        #[track_caller]
        pub fn $f($($arg: impl Into<$crate::design::Value>,)* text: impl $crate::design::Text) -> $crate::design::El {
            elements!(@new $f [$($tag)?] [$($arg)*]).with_children(text.into_children())
        }
    };
    (@new $f:ident [$($tag:literal)?] [$($arg:ident)*]) => {{
        #[allow(unused_mut)]
        let mut el = $crate::design::El::new($crate::design::element::tag_of(stringify!($f), None $(.or(Some($tag)))?));
        $( el = el.with_attr($crate::design::element::ts_name(stringify!($arg)), $arg); el.positional += 1; )*
        el
    }};
}
pub(crate) use elements;

/// A family's attribute methods, as one trait implemented for [`El`]:
///
/// ```text
/// attributes! {
///     /// The schematic's attributes.
///     pub trait SchematicAttrs {
///         /// Where the symbol sits, across: millimetres.
///         sch_x;                          // .sch_x(v) sets "schX"
///         type_;                          // .type_(v) sets "type"
///         stroke_dash = "stroke-dash";    // a name that is not the method's camelCase
///     }
/// }
/// ```
///
/// Each method takes any [`Value`] (numbers, strings, bools, arrays of them, `json(…)`); the family checks the value
/// when it declares, and refuses by name what it cannot read.
macro_rules! attributes {
    ($(#[$tdoc:meta])* pub trait $trait:ident { $( $(#[$doc:meta])* $m:ident $(= $ts:literal)?; )* }) => {
        $(#[$tdoc])*
        pub trait $trait: Sized {
            $( $(#[$doc])* fn $m(self, value: impl Into<$crate::design::Value>) -> Self; )*
        }
        impl $trait for $crate::design::El {
            $(
                fn $m(self, value: impl Into<$crate::design::Value>) -> $crate::design::El {
                    let name: Option<&str> = None $(.or(Some($ts)))?;
                    self.with_attr(name.map(str::to_string).unwrap_or_else(|| $crate::design::element::ts_name(stringify!($m))), value)
                }
            )*
        }
    };
}
pub(crate) use attributes;

#[cfg(test)]
mod tests {
    use super::*;

    mod family {
        elements! {
            /// A leaf with two positional attributes.
            wire_(from, to): leaf;
            /// A container.
            layer: children;
            /// A text tag.
            p: text;
            /// A tag that is not the function's name.
            brush_stroke = "brush-stroke": leaf;
            /// A tag with a capital.
            cap_table = "CapTable": children;
        }
        attributes! {
            /// Attributes.
            pub trait FamilyAttrs {
                /// A camelCase name.
                frozen_rows;
                /// A keyword.
                type_;
                /// A name written beside it.
                stroke_dash = "stroke-dash";
            }
        }
    }
    use family::*;

    #[test]
    fn the_macros_declare_constructors_and_attribute_traits() {
        let w = wire_("a", 2).type_("x").frozen_rows(1).stroke_dash([2, 1]);
        assert_eq!(w.tag, "wire");
        assert_eq!(w.positional, 2);
        assert_eq!(w.call_text(), r#"wire("a", 2)"#);
        let names: Vec<&str> = w.attrs.iter().map(|(k, _)| k.as_str()).collect();
        assert_eq!(names, ["from", "to", "type", "frozenRows", "stroke-dash"]);
        assert_eq!(brush_stroke().tag, "brush-stroke");
        assert_eq!(cap_table([layer([])]).tag, "CapTable");
        assert_eq!(p(("a", brush_stroke())).children.len(), 2);
        assert!(p("").children.is_empty() && p([]).children.is_empty() && layer([]).children.is_empty());
        let line = line!() + 1;
        let el = layer([]);
        assert_eq!(el.site, Some(Site { line, column: 18 }), "the site is the call in this file, through the macro");
    }
}
