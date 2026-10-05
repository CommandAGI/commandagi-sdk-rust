// The map of a schematic in Rust: where each element, attribute and import is, and what the sandbox refuses.

use commandagi::design::Json;

const DIVIDER: &str = r#"use commandagi::design::sheet::{group, resistor, trace};

fn document() -> Group {
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
fn num(j: &Json, k: &str) -> f64 {
    j.get(k).and_then(Json::as_f64).unwrap_or_else(|| panic!("{k} of {}", j.text()))
}
fn prop<'a>(el: &'a Json, name: &str) -> &'a Json {
    match el.get("props") {
        Some(Json::Arr(p)) => p.iter().find(|p| p.get("name").and_then(Json::as_str) == Some(name)).unwrap(),
        _ => panic!(),
    }
}
fn slice(s: &str, a: f64, b: f64) -> String {
    s.encode_utf16().skip(a as usize).take((b - a) as usize).map(|u| char::from_u32(u as u32).unwrap()).collect()
}

#[test]
fn maps_each_element_its_attributes_and_its_place() {
    let m = commandagi_map::map(DIVIDER);
    let els = elements(&m);
    let tags: Vec<&str> = els.iter().map(|e| e.get("tag").and_then(Json::as_str).unwrap()).collect();
    assert_eq!(tags, ["group", "resistor", "resistor", "trace"]);
    let (group, r1, r2, trace) = (&els[0], &els[1], &els[2], &els[3]);
    assert_eq!(group.get("parent"), Some(&Json::Null));
    assert_eq!(num(r1, "parent"), 0.0);
    assert_eq!(r1.get("child"), Some(&Json::Bool(true)));
    assert_eq!(slice(DIVIDER, num(r1, "start"), num(r1, "end")), r#"resistor("R1").resistance("3k").sch_x(114.3).sch_y(-88.9)"#);
    assert_eq!(r1.get("site"), Some(&Json::Arr(vec![6u32.into(), 9u32.into()])));
    let x = prop(r1, "sch_x");
    assert_eq!(slice(DIVIDER, num(x, "start"), num(x, "end")), ".sch_x(114.3)");
    assert_eq!(x.get("value"), Some(&Json::Num(114.3)));
    assert_eq!(prop(r1, "sch_y").get("value"), Some(&Json::Num(-88.9)));
    let computed = prop(r2, "sch_x");
    assert_eq!(computed.get("literal"), Some(&Json::Bool(false)));
    assert_eq!(computed.get("expr").and_then(Json::as_str), Some("139.7 + 1.0"));
    assert_eq!(num(computed, "line"), 9.0);
    // The last item has no comma after it; the array's brackets are known for an insertion.
    assert_eq!(trace.get("item").and_then(|i| i.get("comma")), Some(&Json::Bool(false)));
    let items = group.get("items").unwrap();
    assert_eq!(num(items, "count"), 3.0);
    assert_eq!(items.get("indent").and_then(Json::as_str), Some("        "));
    let imports = m.get("imports").unwrap().text();
    assert!(imports.contains(r#""module":"commandagi::design::sheet","star":false"#), "{imports}");
}

#[test]
fn a_call_in_a_closure_is_not_a_child() {
    let m = commandagi_map::map("fn document() -> Group { group(\"L\", (1..4).map(|i| resistor(format!(\"R{i}\")).sch_x(i))) }");
    let r = &elements(&m)[1];
    assert_eq!(r.get("loop"), Some(&Json::Bool(true)));
    assert_eq!(r.get("child"), Some(&Json::Bool(false)));
}

#[test]
fn refuses_what_reads_the_computer_while_the_file_compiles() {
    let m = commandagi_map::map(
        "use std::include_str as f;\nmod other;\n#[path = \"x.rs\"] mod y {}\nfn document() -> Group { let k = include_str!(\"/etc/passwd\"); if !true {} group(k, vec![]) }\nmacro_rules! m { () => {} }\n",
    );
    let refused = m.get("refused").unwrap().text();
    for what in ["`use` of `include_str` as `f`", "`mod other;`", "#[path]", "`include_str!`", "`macro_rules!`"] {
        assert!(refused.contains(what), "{what} in {refused}");
    }
    assert!(!refused.contains("`if!`"), "{refused}");
}

#[test]
fn offsets_count_utf16_units() {
    let text = "// é 😀\nfn document() -> Group { group(\"D\", [resistor(\"R1\").sch_x(1)]) }\n";
    let m = commandagi_map::map(text);
    let r = &elements(&m)[1];
    assert_eq!(slice(text, num(r, "start"), num(r, "end")), "resistor(\"R1\").sch_x(1)");
    assert_eq!(num(&m, "length"), text.encode_utf16().count() as f64);
}

#[test]
fn a_file_that_does_not_parse_says_where() {
    let m = commandagi_map::map("fn document( {");
    assert!(m.get("error").and_then(Json::as_str).unwrap().starts_with("line 1"));
}
