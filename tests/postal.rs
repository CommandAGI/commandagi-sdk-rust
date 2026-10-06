// Letters and postcards in Rust against the documents the TypeScript SDK's postal.test.ts pins for the same elements:
// the addresses, the paragraphs in order, a postcard's front image; what the vocabulary has no words for is refused by
// name.

use commandagi::design::postal::*;
use commandagi::design::{declare, Declared};

fn document(root: El) -> (String, Json, Json) {
    match declare(root).unwrap() {
        Declared::Document { format, document, sources } => (format, document, sources),
        Declared::Graph(_) => panic!("a letter is a document"),
    }
}

fn bad(root: El) -> String {
    declare(root).unwrap_err()
}

fn j(text: &str) -> Json {
    Json::parse(text).unwrap()
}

fn ada() -> El {
    to().name("Ada Lovelace").line1("12 St James's Square").city("London").postal_code("SW1Y 4JH").country("GB")
}

#[test]
fn a_letter_is_its_addresses_and_paragraphs_each_site_by_its_path() {
    let first = paragraph().text("Dear Ada,");
    let first_site = first.source_json().unwrap();
    let root = letter([
        ada(),
        from().name("Northwind").line1("1 Main St").city("Portland").region("OR").postal_code("97201").country("US"),
        first,
        paragraph().text("It ships on Monday."),
    ])
    .mail_class("first");
    let root_site = root.source_json().unwrap();
    let (format, doc, sources) = document(root);
    assert_eq!(format, "letter");
    assert_eq!(
        doc,
        j(r#"{"mailClass": "first",
              "to": {"name": "Ada Lovelace", "line1": "12 St James's Square", "city": "London", "postalCode": "SW1Y 4JH", "country": "GB"},
              "from": {"name": "Northwind", "line1": "1 Main St", "city": "Portland", "region": "OR", "postalCode": "97201", "country": "US"},
              "paragraphs": ["Dear Ada,", "It ships on Monday."]}"#)
    );
    assert_eq!(sources.get(""), Some(&root_site));
    assert_eq!(sources.get("paragraph@0"), Some(&first_site));
    assert!(sources.get("to").is_some());

    assert!(bad(letter([to().with_attr("street", "x")])).contains("street is not read"));
    assert!(bad(letter([]).mail_class("express")).contains(r#"mail_class is "first" or "standard""#));
    assert!(bad(letter([to(), to()])).contains("letter() has one to()"));
    assert!(bad(letter([]).color("yes")).contains("color is true or false"));
    assert!(bad(letter([front().image("x.jpg")])).contains("front() is not an element of a letter"));
}

#[test]
fn a_postcard_has_one_front_and_no_options() {
    let (format, doc, _) = document(postcard([ada(), front().image("Austin.jpg"), paragraph().text("Greetings from Austin!")]));
    assert_eq!(format, "postcard");
    assert_eq!(
        doc,
        j(r#"{"to": {"name": "Ada Lovelace", "line1": "12 St James's Square", "city": "London", "postalCode": "SW1Y 4JH", "country": "GB"},
              "front": "Austin.jpg", "paragraphs": ["Greetings from Austin!"]}"#)
    );
    assert!(bad(postcard([front()])).contains("needs image"));
    assert!(bad(postcard([]).color(true)).contains("color is not read"));
}
