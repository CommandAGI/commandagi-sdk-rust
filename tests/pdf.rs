// A PDF in Rust against the document the TypeScript SDK's pdf.test.ts pins for the same elements: pages by ref and
// blank, marks with replies, fills, nested bookmarks, labels and attachments; what the vocabulary has no words for
// is refused by name.

use commandagi::design::pdf::*;
use commandagi::design::{declare, Declared};

fn document(root: El) -> (String, Json) {
    match declare(root).unwrap() {
        Declared::Document { format, document, .. } => (format, document),
        Declared::Graph(_) => panic!("a PDF is a document"),
    }
}

fn bad(root: El) -> String {
    declare(root).unwrap_err()
}

#[test]
fn a_pdf_is_its_pages_marks_and_structure() {
    let (format, doc) = document(
        pdf([
            page([]).src("Contract.pdf").n(1),
            page([
                highlight().rects([[72, 700, 300, 712]]).author("Ada").text("Check"),
                note([reply().text("Because.").author("Bob")]).at([500, 700]).text("Why?"),
                redact().rect([72, 500, 300, 520]),
                field().kind("text").name("Name").rect([72, 100, 300, 120]),
            ])
            .src("Contract.pdf")
            .n(3)
            .rotate(90),
            page([]).size("a4"),
            fill().name("Name").value("Ada Lovelace"),
            bookmark([bookmark([]).title("Payment").page(2).top(500)]).title("Terms").page(2),
            label().from(1).style("r"),
            attach().src("data.csv"),
        ])
        .title("Signed")
        .author("Ada"),
    );
    assert_eq!(format, "pdf");
    assert_eq!(
        doc,
        Json::parse(
            r#"{"title": "Signed", "author": "Ada", "pages": [
                {"src": "Contract.pdf", "n": 1},
                {"src": "Contract.pdf", "n": 3, "rotate": 90, "marks": [
                    {"type": "highlight", "rects": [[72, 700, 300, 712]], "author": "Ada", "text": "Check"},
                    {"type": "note", "at": [500, 700], "text": "Why?", "replies": [{"text": "Because.", "author": "Bob"}]},
                    {"type": "redact", "rect": [72, 500, 300, 520]},
                    {"type": "field", "kind": "text", "name": "Name", "rect": [72, 100, 300, 120]}]},
                {"size": "a4"}],
              "fill": [{"name": "Name", "value": "Ada Lovelace"}],
              "bookmarks": [{"title": "Terms", "page": 2, "children": [{"title": "Payment", "page": 2, "top": 500}]}],
              "labels": [{"from": 1, "style": "r"}],
              "attachments": [{"src": "data.csv"}]}"#
        )
        .unwrap()
    );
}

#[test]
fn what_it_cannot_say_is_refused() {
    assert!(bad(pdf([page([]).src("a.pdf")])).contains("n is the page's number"));
    assert!(bad(pdf([page([])])).contains("blank page"));
    assert!(bad(pdf([page([stamp().rect([0, 0, 10, 10])]).size("a4")])).contains("has a name"));
    assert!(bad(pdf([page([signature().rect([0, 0, 10, 10]).typed("A").image("s.png")]).size("a4")])).contains("one of typed, image or strokes"));
}
