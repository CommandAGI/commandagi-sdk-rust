// The map of a document in Rust: where each element, attribute, argument and import is, and what the sandbox refuses
// (the shape every SDK gives, `graph.meta.sourceMap`).

use commandagi::design::Json;

const DIVIDER: &str = r#"use commandagi::design::schematic::{group, resistor, trace};

fn document() -> El {
    group("Divider", [
        // R1
        resistor("R1").resistance("3k").sch_x(114.3).sch_y(-88.9),
        resistor("R2")
            .resistance("1.5k")
            .sch_x(139.7 + 1.0),
        trace(".R1 > .pin2", ".R2 > .pin1")
    ])
}
"#;

fn elements(m: &Json) -> &Vec<Json> {
    match m.get("elements") {
        Some(Json::Arr(e)) => e,
        _ => panic!("no elements: {}", m.text()),
    }
}
fn arr<'a>(j: &'a Json, k: &str) -> &'a Vec<Json> {
    match j.get(k) {
        Some(Json::Arr(a)) => a,
        _ => panic!("{k} of {}", j.text()),
    }
}
fn num(j: &Json, k: &str) -> f64 {
    j.get(k).and_then(Json::as_f64).unwrap_or_else(|| panic!("{k} of {}", j.text()))
}
fn prop<'a>(el: &'a Json, name: &str) -> &'a Json {
    arr(el, "props").iter().find(|p| p.get("name").and_then(Json::as_str) == Some(name)).unwrap()
}
fn slice(s: &str, a: f64, b: f64) -> String {
    s.encode_utf16().skip(a as usize).take((b - a) as usize).map(|u| char::from_u32(u as u32).unwrap()).collect()
}

#[test]
fn maps_each_element_its_attributes_its_arguments_and_its_place() {
    let m = commandagi_map::map(DIVIDER);
    assert_eq!(m.get("language").and_then(Json::as_str), Some("rs"));
    let els = elements(&m);
    let tags: Vec<&str> = els.iter().map(|e| e.get("tag").and_then(Json::as_str).unwrap()).collect();
    assert_eq!(tags, ["group", "resistor", "resistor", "trace"]);
    let (group, r1, r2, trace) = (&els[0], &els[1], &els[2], &els[3]);
    assert_eq!(els.iter().map(|e| num(e, "index")).collect::<Vec<_>>(), [0.0, 1.0, 2.0, 3.0]);
    assert_eq!(group.get("parent"), Some(&Json::Null));
    assert_eq!(group.get("callee").and_then(Json::as_str), Some("group"));
    assert!(group.get("spread").unwrap().get("expr").is_some(), "the root's value is passed on");
    assert_eq!(group.get("slot"), Some(&Json::Null));
    assert_eq!(slice(DIVIDER, num(group, "open"), num(group, "close")).chars().next(), Some('"'));
    assert_eq!(num(r1, "parent"), 0.0);
    assert_eq!(r1.get("placed"), Some(&Json::Bool(true)));
    assert_eq!(r1.get("looped"), Some(&Json::Bool(false)));
    assert_eq!(r1.get("spread"), Some(&Json::Null));
    assert_eq!(slice(DIVIDER, num(r1, "start"), num(r1, "end")), r#"resistor("R1").resistance("3k").sch_x(114.3).sch_y(-88.9)"#);
    assert_eq!(r1.get("site"), Some(&Json::Arr(vec![6u32.into(), 9u32.into()])));
    assert_eq!((num(r1, "line"), num(r1, "column")), (6.0, 9.0));
    let x = prop(r1, "sch_x");
    assert_eq!(slice(DIVIDER, num(x, "start"), num(x, "end")), ".sch_x(114.3)");
    assert_eq!(slice(DIVIDER, num(x, "valueStart"), num(x, "valueEnd")), "114.3");
    assert_eq!(x.get("value"), Some(&Json::Num(114.3)));
    assert_eq!(prop(r1, "sch_y").get("value"), Some(&Json::Num(-88.9)));
    let computed = prop(r2, "sch_x");
    assert_eq!(computed.get("literal"), Some(&Json::Bool(false)));
    assert_eq!(computed.get("expr").and_then(Json::as_str), Some("139.7 + 1.0"));
    assert_eq!(num(computed, "line"), 9.0);

    // The group's arguments: its name, then its children list, each entry naming its element.
    let args = arr(group, "args");
    assert_eq!(args[0].get("value"), Some(&Json::from("Divider")));
    let list = args[1].get("list").unwrap();
    assert_eq!(list.get("kind").and_then(Json::as_str), Some("array"));
    assert_eq!(list.get("trailing"), Some(&Json::Bool(false)));
    assert_eq!(slice(DIVIDER, num(list, "close"), num(list, "close") + 1.0), "]");
    let entries = arr(list, "entries");
    assert_eq!(entries.iter().map(|e| num(e, "element")).collect::<Vec<_>>(), [1.0, 2.0, 3.0]);
    assert_eq!(arr(trace, "args")[1].get("value"), Some(&Json::from(".R2 > .pin1")));
    // The last entry has no comma after it; its slot ends at the `]`.
    let slot = trace.get("slot").unwrap();
    assert_eq!(slot.get("comma"), Some(&Json::Bool(false)));
    assert_eq!(num(slot, "after"), num(list, "close"));
    assert_eq!(num(r2.get("slot").unwrap(), "before"), num(r1, "end"));

    let imports = m.get("imports").unwrap().text();
    assert!(imports.contains(r#""module":"commandagi::design::schematic","star":false"#), "{imports}");
    assert_eq!(arr(&m, "refused").len(), 0);
}

#[test]
fn a_call_in_a_closure_is_looped_and_not_placed() {
    let m = commandagi_map::map("fn document() -> El { group(\"L\", (1..4).map(|i| resistor(format!(\"R{i}\")).sch_x(i))) }");
    let els = elements(&m);
    let r = &els[1];
    assert_eq!(r.get("looped"), Some(&Json::Bool(true)));
    assert_eq!(r.get("placed"), Some(&Json::Bool(false)));
    assert_eq!(r.get("slot"), Some(&Json::Null));
    assert!(r.get("spread").unwrap().get("expr").is_some());
    assert_eq!(arr(&els[0], "args")[1].get("literal"), Some(&Json::Bool(false)));
}

#[test]
fn a_text_tuple_lists_its_texts_and_elements_and_json_is_a_literal() {
    let text = r##"fn document() -> El { page([p(("The run ", b("passed"), ".",)), p(b("x")), rect().points([[0.0, 0.0], [10.0, 0.5]]).tags(vec!["a", "b"]).meta(json(r#"{"k": [1]}"#)).dx(-1.5)]) }"##;
    let m = commandagi_map::map(text);
    let els = elements(&m);
    let tags: Vec<&str> = els.iter().map(|e| e.get("tag").and_then(Json::as_str).unwrap()).collect();
    assert_eq!(tags, ["page", "p", "b", "p", "b", "rect"], "json(…) is not an element");
    let tuple = arr(&els[1], "args")[0].get("list").unwrap();
    assert_eq!(tuple.get("kind").and_then(Json::as_str), Some("tuple"));
    assert_eq!(tuple.get("trailing"), Some(&Json::Bool(true)));
    let entries = arr(tuple, "entries");
    assert_eq!(entries[0].get("value"), Some(&Json::from("The run ")));
    assert_eq!(num(&entries[1], "element"), 2.0);
    assert!(entries[0].get("element").is_none());
    assert_eq!(num(&els[2], "parent"), 1.0);
    assert!(els[2].get("slot").unwrap().get("before").is_some(), "an entry of a tuple has a slot");
    // A lone element argument is the call's, not passed on, and has no slot.
    let lone = &els[4];
    assert_eq!(num(&arr(&els[3], "args")[0], "element"), 4.0);
    assert_eq!(lone.get("spread"), Some(&Json::Null));
    assert_eq!(lone.get("slot"), Some(&Json::Null));
    let rect = &els[5];
    assert_eq!(prop(rect, "points").get("value").unwrap().text(), "[[0,0],[10,0.5]]");
    assert_eq!(prop(rect, "tags").get("value").unwrap().text(), r#"["a","b"]"#);
    assert_eq!(prop(rect, "meta").get("value").unwrap().text(), r#"{"k":[1]}"#);
    assert_eq!(prop(rect, "dx").get("value"), Some(&Json::Num(-1.5)));
    let vec_list = commandagi_map::map("fn document() -> El { layer(vec![rect(),]) }");
    let list = arr(&elements(&vec_list)[0], "args")[0].get("list").unwrap().clone();
    assert_eq!(list.get("kind").and_then(Json::as_str), Some("vec"));
    assert_eq!(list.get("trailing"), Some(&Json::Bool(true)));
}

#[test]
fn a_type_path_or_a_variant_is_not_an_element() {
    let m = commandagi_map::map("fn document() -> El { let n = String::from(\"R1\"); group(&n, [Some(rect()).unwrap()]) }");
    let tags: Vec<&str> = elements(&m).iter().map(|e| e.get("tag").and_then(Json::as_str).unwrap()).collect();
    assert_eq!(tags, ["group", "rect"]);
}

#[test]
fn refuses_what_reads_the_computer_while_the_file_compiles() {
    let m = commandagi_map::map(
        "use std::include_str as f;\nmod other;\n#[path = \"x.rs\"] mod y {}\nfn document() -> El { let k = include_str!(\"/etc/passwd\"); if !true {} group(k, vec![]) }\nmacro_rules! m { () => {} }\n",
    );
    let refused = m.get("refused").unwrap().text();
    for what in ["`use` of `include_str` as `f`", "`mod other;`", "#[path]", "`include_str!`", "`macro_rules!`"] {
        assert!(refused.contains(what), "{what} in {refused}");
    }
    assert!(!refused.contains("`if!`"), "{refused}");
}

#[test]
fn offsets_count_utf16_units() {
    let text = "// é 😀\nfn document() -> El { group(\"D\", [resistor(\"R1\").sch_x(1)]) }\n";
    let m = commandagi_map::map(text);
    let r = &elements(&m)[1];
    assert_eq!(slice(text, num(r, "start"), num(r, "end")), "resistor(\"R1\").sch_x(1)");
    assert_eq!(num(&m, "length"), text.encode_utf16().count() as f64);
}

#[test]
fn a_file_that_does_not_parse_says_where() {
    let m = commandagi_map::map("fn document( {");
    assert!(m.get("error").and_then(Json::as_str).unwrap().starts_with("line 1"));
    assert_eq!(arr(&m, "elements").len(), 0);
}
