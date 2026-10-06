// The one element model every family is written in: values, children and text parameters, the family registry and
// the run's answer. A family of this test's own (`page`, `p`, `b`) shows how a module is written.

use commandagi::design::{declare, Child, Declared, El, Json};

mod page {
    pub use commandagi::design::{fragment, json, El};

    // A family lives in the crate (its macros are crate-internal; src/design/element.rs tests them), so this one is
    // written by hand, the way the macros expand.
    #[track_caller]
    pub fn p(text: impl commandagi::design::Text) -> El {
        El::new("p").with_children(text.into_children())
    }
    #[track_caller]
    pub fn b(text: impl commandagi::design::Text) -> El {
        El::new("b").with_children(text.into_children())
    }
    #[track_caller]
    pub fn layer(children: impl IntoIterator<Item = El>) -> El {
        El::new("layer").with_children(children.into_iter().map(commandagi::design::Child::El))
    }
    #[track_caller]
    pub fn rect() -> El {
        El::new("rect")
    }
    pub trait PageAttrs: Sized {
        fn frozen_rows(self, v: impl Into<commandagi::design::Value>) -> Self;
        fn points(self, v: impl Into<commandagi::design::Value>) -> Self;
    }
    impl PageAttrs for El {
        fn frozen_rows(self, v: impl Into<commandagi::design::Value>) -> El {
            self.with_attr(commandagi::design::element::ts_name("frozen_rows"), v)
        }
        fn points(self, v: impl Into<commandagi::design::Value>) -> El {
            self.with_attr("points", v)
        }
    }
}
use page::*;

fn texts(el: &El) -> Vec<String> {
    el.children
        .iter()
        .map(|c| match c {
            Child::Text(t) => format!("{t:?}"),
            Child::El(e) => format!("<{}>", e.tag),
        })
        .collect()
}

#[test]
fn values_are_numbers_strings_bools_homogeneous_arrays_and_json_text() {
    let el = rect()
        .points(1)
        .points(2.5f32)
        .frozen_rows(3usize);
    assert_eq!(el.attrs, vec![("points".to_string(), Json::Num(2.5)), ("frozenRows".to_string(), Json::Num(3.0))], "the last value wins in the first one's place");
    let v = |e: El| e.attr("points").unwrap().text();
    assert_eq!(v(rect().points(0.1f32)), "0.1");
    assert_eq!(v(rect().points(-4i64)), "-4");
    assert_eq!(v(rect().points("a")), r#""a""#);
    assert_eq!(v(rect().points(String::from("b"))), r#""b""#);
    assert_eq!(v(rect().points(true)), "true");
    assert_eq!(v(rect().points([1.0, 2.5])), "[1,2.5]");
    assert_eq!(v(rect().points([[0, 0], [10, 0]])), "[[0,0],[10,0]]");
    assert_eq!(v(rect().points([[0.0, 0.0], [10.0, 0.5]])), "[[0,0],[10,0.5]]");
    assert_eq!(v(rect().points(["a", "b"])), r#"["a","b"]"#);
    assert_eq!(v(rect().points([true, false])), "[true,false]");
    assert_eq!(v(rect().points(vec![1u32, 2])), "[1,2]");
    assert_eq!(v(rect().points(&[3i32, 4][..])), "[3,4]");
    assert_eq!(v(rect().points(json(r#"{"acMagnitude": 1, "list": [null]}"#))), r#"{"acMagnitude":1,"list":[null]}"#);
}

#[test]
fn json_text_that_is_not_json_is_refused_by_name_when_declared() {
    let root = layer([rect().points(json("{nope"))]);
    assert_eq!(root.refusal().unwrap_err(), "rect(): points is JSON text (expected a string at 1)");
    assert_eq!(declare(root).unwrap_err(), "rect(): points is JSON text (expected a string at 1)", "refused before any family reads it");
}

#[test]
fn a_container_takes_arrays_vecs_iterators_and_fragments() {
    assert_eq!(layer([]).children.len(), 0);
    assert_eq!(layer([rect(), rect()]).children.len(), 2);
    assert_eq!(layer(vec![rect()]).children.len(), 1);
    assert_eq!(layer((0..3).map(|_| rect())).children.len(), 3);
    let l = layer([rect(), fragment((0..2).map(|_| layer([]))), rect()]);
    assert_eq!(l.child_elements().map(|e| e.tag).collect::<Vec<_>>(), ["rect", "layer", "layer", "rect"]);
}

#[test]
fn a_text_tag_takes_a_text_an_element_a_tuple_or_a_list_of_elements() {
    assert_eq!(texts(&p("The run passed.")), [r#""The run passed.""#]);
    assert_eq!(texts(&p(String::from("s"))), [r#""s""#]);
    assert_eq!(texts(&p(&String::from("s"))), [r#""s""#]);
    assert_eq!(texts(&p(b("x"))), ["<b>"]);
    assert_eq!(texts(&p(("The run ", b("passed"), "."))), [r#""The run ""#, "<b>", r#"".""#]);
    assert_eq!(texts(&p((b("a"),))), ["<b>"]);
    assert_eq!(texts(&p([b("a"), b("c")])), ["<b>", "<b>"]);
    assert_eq!(texts(&p(vec![b("a")])), ["<b>"]);
    assert!(p("").children.is_empty(), "an empty text is no child");
    assert!(p([]).children.is_empty());
    assert_eq!(texts(&p(("a", fragment([b("x"), b("y")]), "z"))), [r#""a""#, "<b>", "<b>", r#""z""#]);
}

#[test]
fn each_constructor_knows_the_call_that_made_it() {
    let el = rect();
    let line = line!() - 1;
    assert_eq!(el.site.map(|s| (s.line, s.column)), Some((line, 14)));
    assert_eq!(el.source_json().unwrap().text(), format!("{{\"site\":[{line},14]}}"));
    assert!(fragment([]).site.is_none());
}

#[test]
fn names_follow_the_typescript_names() {
    use commandagi::design::element::{fn_name, rust_name, ts_name};
    assert_eq!(ts_name("sch_x"), "schX");
    assert_eq!(ts_name("frozen_rows"), "frozenRows");
    assert_eq!(ts_name("type_"), "type");
    assert_eq!(rust_name("schX"), "sch_x");
    assert_eq!(rust_name("type"), "type_");
    assert_eq!(fn_name("brush-stroke"), "brush_stroke");
    assert_eq!(fn_name("CapTable"), "cap_table");
    assert_eq!(fn_name("Company"), "company");
    assert_eq!(fn_name("move"), "move_");
}

#[test]
fn the_registry_declares_by_the_roots_tag_and_refuses_a_root_no_family_has() {
    assert!(matches!(declare(commandagi::design::schematic::group("D", [])), Ok(Declared::Graph(_))));
    let err = declare(layer([])).unwrap_err();
    assert!(err.starts_with("layer() is not the root of a document (schematic::group"), "{err}");
}

#[test]
fn the_answer_carries_the_graph_or_the_document_beside_an_empty_graph() {
    use commandagi::abi::answer;
    let graph = answer(declare(commandagi::design::schematic::group("D", []))).unwrap();
    assert_eq!(graph.get("graph").and_then(|g| g.get("id")), Some(&Json::from("eda:D")));
    assert!(graph.get("document").is_none());
    let doc = Declared::Document { format: "page".into(), document: Json::obj().with("title", "T"), sources: Json::obj().with("", Json::obj().with("site", vec![Json::from(1u32), Json::from(1u32)])) };
    let a = answer(Ok(doc)).unwrap();
    assert_eq!(a.get("graph").unwrap().text(), r#"{"id":"Document","nodes":{}}"#);
    assert_eq!(a.get("document").unwrap().text(), r#"{"format":"page","document":{"title":"T"},"sources":{"":{"site":[1,1]}}}"#);
    assert_eq!(a.get("ok"), Some(&Json::Bool(true)));
    assert!(matches!(a.get("sites"), Some(Json::Arr(_))));
}

#[test]
fn run_leaves_the_answer_or_the_error() {
    assert_eq!(commandagi::abi::run(|| commandagi::design::schematic::group("D", [])), 1);
    assert!(commandagi::abi::out_text().starts_with(r#"{"ok":true,"graph":{"id":"eda:D""#));
    assert_eq!(commandagi::abi::run(|| layer([])), 0);
    assert!(commandagi::abi::out_text().starts_with(r#"{"ok":false,"error":"layer() is not the root"#));
}
