// The office vocabulary against the documents the TypeScript SDK declares for the same elements (`office.test.ts`):
// a workbook's cells, a page's blocks, a deck's graph; the refusals by name; where each part was written.

use commandagi::design::office::*;
use commandagi::design::{declare, Declared};

fn document(root: El) -> (Json, Json) {
    match declare(root).unwrap() {
        Declared::Document { document, sources, .. } => (document, sources),
        Declared::Graph(g) => panic!("a graph: {}", g.text()),
    }
}

fn graph(root: El) -> Json {
    declare(root).unwrap().graph().unwrap()
}

fn bad(root: El) -> String {
    declare(root).unwrap_err()
}

fn golden(text: &str) -> String {
    Json::parse(text).unwrap().text()
}

#[test]
fn a_workbook_declares_cells_by_a1_reference_formats_and_sizes() {
    let (doc, sources) = document(
        workbook([sheet([column().at("B").width(120), cell().at("a1").value("Item").bold(true), cell().at("B2").formula("=1+1")])
            .name("Q1")
            .frozen_rows(1)])
        .title("Budget"),
    );
    assert_eq!(
        doc.text(),
        golden(
            r#"{"format": "workbook", "version": 1, "sheets": [{"id": "sheet-1", "name": "Q1", "kind": "grid", "rows": 200, "cols": 26,
                "cells": {"A1": {"v": "Item", "fmt": {"bold": true}}, "B2": {"f": "=1+1"}}, "colWidths": {"1": 120},
                "frozen": {"rows": 1, "cols": 0}}], "meta": {"title": "Budget"}}"#
        )
    );
    let keys: Vec<&str> = match &sources {
        Json::Obj(e) => e.iter().map(|(k, _)| k.as_str()).collect(),
        _ => vec![],
    };
    assert_eq!(keys, ["workbook", "sheet:sheet-1", "column:sheet-1!B", "cell:sheet-1!A1", "cell:sheet-1!B2"]);
}

#[test]
fn a_page_declares_blocks_whose_text_is_inline_html() {
    let (doc, _) = document(
        page([
            h1("Notes"),
            p(("One < two & ", b("bold"), " ", a("link").href("https://x.test"))),
            todo("done").checked(true),
            pre("a { }").lang("ts"),
            divider(),
        ])
        .title("Notes"),
    );
    assert_eq!(
        doc.get("blocks").unwrap().text(),
        golden(
            r#"[{"id": "block-1", "type": "h1", "html": "Notes"},
                {"id": "block-2", "type": "p", "html": "One &lt; two &amp; <b>bold</b> <a href=\"https://x.test\">link</a>"},
                {"id": "block-3", "type": "todo", "html": "done", "checked": true},
                {"id": "block-4", "type": "code", "html": "a { }", "lang": "ts"},
                {"id": "block-5", "type": "divider", "html": ""}]"#
        )
    );
}

#[test]
fn a_deck_declares_the_decks_graph_with_rich_text() {
    let g = graph(
        deck([
            slide([text(("Hello ", b("world"))).placeholder("title")]).layout("Title"),
            slide([shape().shape("ellipse").x(1).y(2).w(3).h(4)]).layout("Blank"),
        ])
        .name("Pitch"),
    );
    let nodes = g.get("nodes").unwrap();
    let ids: Vec<&str> = match nodes {
        Json::Obj(e) => e.iter().map(|(k, _)| k.as_str()).collect(),
        _ => vec![],
    };
    assert_eq!(ids, ["slide-1.1", "slide-1", "slide-2.1", "slide-2", "doc"]);
    let inputs = |id: &str| nodes.get(id).unwrap().get("inputs").unwrap().text();
    assert_eq!(
        inputs("doc"),
        golden(r#"{"name": "Pitch", "width": 1280, "height": 720, "slides.1": {"wire": {"node": "slide-1", "port": "out"}}, "slides.2": {"wire": {"node": "slide-2", "port": "out"}}}"#)
    );
    assert_eq!(inputs("slide-1"), golden(r#"{"layout": "Title", "elements.1": {"wire": {"node": "slide-1.1", "port": "out"}}}"#));
    assert_eq!(inputs("slide-1.1"), golden(r#"{"placeholder": "title", "text": {"paragraphs": [{"runs": [{"text": "Hello "}, {"text": "world", "bold": true}]}]}}"#));
    assert_eq!(inputs("slide-2.1"), golden(r#"{"box": {"x": 1, "y": 2, "w": 3, "h": 4}, "shape": "ellipse"}"#));
    assert_eq!(g.get("id").unwrap().as_str(), Some("deck:Pitch"));
}

#[test]
fn every_part_carries_the_call_it_came_from() {
    let line = line!() + 1;
    let g = graph(deck([slide([text("Hi").placeholder("title")])]));
    let site = |id: &str| g.get("nodes").unwrap().get(id).unwrap().get("meta").unwrap().get("source").unwrap().get("site").unwrap().text();
    assert_eq!(site("doc"), format!("[{line},19]"));
    assert_eq!(site("slide-1"), format!("[{line},25]"));
    assert_eq!(site("slide-1.1"), format!("[{line},32]"));
    let line = line!() + 1;
    let (_, sources) = document(page([h1("A"), p("B")]));
    assert_eq!(sources.get("block:block-2").unwrap().text(), format!(r#"{{"site":[{line},48]}}"#));
}

#[test]
fn refuses_what_the_documents_cannot_hold() {
    assert_eq!(bad(workbook([sheet([cell().at("A1").formula("SUM(B1)")])])), "cell().at(\"A1\"): a formula starts with = (\"=SUM(B2:B4)\")");
    assert_eq!(
        bad(workbook([sheet([cell().at("A1").width(3)])])),
        "cell().at(\"A1\"): width is not read on a cell() (it takes at, value, formula, num_fmt, bold, italic, align, bg, color, wrap)"
    );
    assert_eq!(bad(workbook([sheet([cell().at("A1").value(1).formula("=1")])])), "cell().at(\"A1\"): a cell has a value or a formula, not both");
    assert_eq!(bad(workbook([sheet([cell().at("1A")])])), "cell(): at is an A1 reference (\"B4\"), not \"1A\"");
    assert_eq!(bad(workbook([sheet([]).name("A"), sheet([]).name("a")])), "two sheets are called a");
    assert_eq!(bad(workbook([])), "a workbook() holds at least one sheet()");
    assert!(bad(page([shape()])).starts_with("shape() is not a block of a page()"));
    assert_eq!(bad(page([p(text("x"))])), "p(): text() is not a mark (the marks are b, i, u, s, code, a and br)");
    assert_eq!(bad(deck([slide([shape().x(1)])])), "the shape() 1 of slide-1 needs x, y, w and h (or a placeholder to take them from)");
    assert_eq!(bad(deck([slide([text([p("a"), b("b")])])])), "text() of slide-1: a text with paragraphs holds only p()s");
    assert_eq!(bad(deck([slide([shape().shape("star").x(0).y(0).w(1).h(1)])])), "shape(): shape is one of rect, ellipse, triangle, line");
}

#[test]
fn a_sheets_chart_pivot_and_filter_are_checked_against_the_workbook_schema() {
    let book = |kids: Vec<El>| workbook([sheet([vec![cell().at("A1").value("Region")], kids].concat()).name("Sales")]);
    let (doc, sources) = document(book(vec![
        chart().name("Sales").range("A1:C5").kind("bar").category("Region").series(["Q1", "Q2"]).at("E2"),
        pivot().name("By region").range("A1:C5").rows("Region").value("Q1").aggregate("sum"),
        filter([criterion().column("b").test("greater").value(100), criterion().column("A").test("notBlank")]).range("A1:C5"),
    ]));
    let Some(Json::Arr(sheets)) = doc.get("sheets") else { panic!("no sheets") };
    let s = &sheets[0];
    assert_eq!(s.get("charts").unwrap().text(), golden(r#"[{"id": "chart-1", "name": "Sales", "range": "A1:C5", "kind": "bar", "category": "Region", "series": ["Q1", "Q2"], "at": "E2"}]"#));
    assert_eq!(s.get("pivots").unwrap().text(), golden(r#"[{"id": "pivot-1", "name": "By region", "range": "A1:C5", "rows": "Region", "value": "Q1", "aggregate": "sum"}]"#));
    assert_eq!(s.get("filter").unwrap().text(), golden(r#"{"range": "A1:C5", "criteria": [{"column": "B", "test": "greater", "value": 100}, {"column": "A", "test": "notBlank"}]}"#));
    for key in ["chart:sheet-1!chart-1", "pivot:sheet-1!pivot-1", "filter:sheet-1", "criterion:sheet-1!B", "criterion:sheet-1!A"] {
        assert!(sources.get(key).is_some(), "{key} says where it was written");
    }
    // The same sentences as the TypeScript reader (office.test.ts).
    let refused = bad(book(vec![chart().name("C").range("A1:C5").kind("donut").series(["Q1"])]));
    assert!(refused.contains(r#"kind: must be one of bar, line, area, scatter, pie (got "donut")"#), "{refused}");
    assert!(bad(book(vec![pivot().name("P").range("nowhere").rows("Region").value("Q1").aggregate("sum")])).contains(r#"range is an A1 range ("A1:C20"), not "nowhere""#));
    assert!(bad(book(vec![pivot().name("P").range("A1:C5").rows("Region").aggregate("sum")])).contains("value: is required"));
    assert!(bad(book(vec![chart().name("C").range("A1:B2").kind("bar").series(["Q1"]), chart().name("c").range("A1:B2").kind("bar").series(["Q1"])])).contains("two charts are called c"));
    assert!(bad(book(vec![filter([]).range("A1:C5"), filter([]).range("A1:C5")])).contains("Sales has two filters"));
    assert!(bad(book(vec![filter([criterion().column("B").test("blank"), criterion().column("B").test("notBlank")]).range("A1:C5")])).contains("two criteria for column B"));
}
