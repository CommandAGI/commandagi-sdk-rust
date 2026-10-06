// A company, an RFC and a case in Rust against the documents the TypeScript SDK's business.test.ts pins for the same
// elements: the native documents, each element's site by path; an unknown attribute or child is refused by name.

use commandagi::design::business::*;
use commandagi::design::{declare, Declared};

fn document(root: El) -> (String, Json, Json) {
    match declare(root).unwrap() {
        Declared::Document { format, document, sources } => (format, document, sources),
        Declared::Graph(_) => panic!("a business document is a document"),
    }
}

fn bad(root: El) -> String {
    declare(root).unwrap_err()
}

fn j(text: &str) -> Json {
    Json::parse(text).unwrap()
}

#[test]
fn a_company_is_the_company_document_with_each_site_by_path() {
    let parts = [
        entity().jurisdiction("US-DE").form("llc").formed("2024-01-31").ids(json(r#"{"ein": "12-3"}"#)),
        books().journal("Northwind/books/main.journal"),
        cap_table().ocf("Northwind/captable/"),
        registration().kind("tax-id").jurisdiction("US").id("12-3"),
    ];
    let sites: Vec<Json> = parts.iter().map(|p| p.source_json().unwrap()).collect();
    let root = company(parts).name("Northwind").files(["Northwind/"]);
    let root_site = root.source_json().unwrap();
    let (format, doc, sources) = document(root);
    assert_eq!(format, "company");
    assert_eq!(
        doc,
        j(r#"{"format": "commandagi-company", "name": "Northwind", "about": "", "files": ["Northwind/"], "dashboard": null,
              "entity": {"jurisdiction": "US-DE", "form": "llc", "name": null, "formed": "2024-01-31", "fiscalYearEnd": null, "ids": {"ein": "12-3"}, "operatesIn": []},
              "standard": {"enabled": true, "books": "Northwind/books/main.journal", "captable": "Northwind/captable/", "people": null, "calendar": null,
                           "registrations": [{"kind": "tax-id", "jurisdiction": "US", "id": "12-3"}], "matters": null}}"#)
    );
    let want = Json::obj()
        .with("", root_site)
        .with("entity", sites[0].clone())
        .with("books", sites[1].clone())
        .with("captable", sites[2].clone())
        .with("registrations/0", sites[3].clone());
    assert_eq!(sources, want);
    assert_eq!(document(company([]).name("Bare")).1.get("standard"), Some(&j(r#"{"enabled": false}"#)), "no part named: the standard is off");
}

#[test]
fn an_rfc_is_its_draft_and_id_a_case_its_harms_and_relief() {
    let ch = change().op("set_parameter").parameter("rfcDepositCents").value("500");
    let ch_site = ch.source_json().unwrap();
    let (format, doc, sources) = document(rfc([option([ch]).title("Ten hours")]).id("rfc_1").title("Rest"));
    assert_eq!(format, "rfc");
    assert_eq!(doc, j(r#"{"id": "rfc_1", "draft": {"title": "Rest", "options": [{"title": "Ten hours", "changes": [{"op": "set_parameter", "value": "500", "key": "rfcDepositCents"}]}]}}"#));
    assert_eq!(sources.get("options/0/changes/0"), Some(&ch_site));

    let (format, kase, _) = document(case([harm().id("h1").interest("property").amount("4200"), relief().kind("restitution").harm_ids(["h1"])]).respondent("Acme"));
    assert_eq!(format, "case");
    assert_eq!(kase, j(r#"{"draft": {"respondent": "Acme", "harms": [{"id": "h1", "interest": "property", "amount": "4200"}], "relief": [{"kind": "restitution", "harmIds": ["h1"]}]}}"#));
    // An amount a file stored as a number stays a number (the form types text; both are kept as they are).
    let (_, stored, _) = document(case([harm().id("h1").amount(4200), relief().kind("exclusion").amount(10.5).days(30)]));
    assert_eq!(stored, j(r#"{"draft": {"harms": [{"id": "h1", "amount": 4200}], "relief": [{"kind": "exclusion", "amount": 10.5, "days": 30}]}}"#));
    assert!(bad(case([harm().id("h1").amount(true)])).contains("harm(): amount is text or a number, not true"));
}

#[test]
fn what_the_vocabulary_does_not_say_is_refused_by_name() {
    assert!(bad(company([]).name("X").with_attr("mailbox", "a@b")).contains("company() has no attribute mailbox"));
    assert!(bad(company([harm().id("h")]).name("X")).contains("company() does not take harm()"));
    assert!(bad(company([books().journal("a"), books().journal("b")]).name("X")).contains("two books()"));
    assert!(bad(company([])).contains("company() needs name"));
    assert!(bad(company([]).name("")).contains("company() needs name"));
    assert!(bad(company([]).name("X").files("X/")).contains("files is a list of text"));
    assert!(bad(entity().jurisdiction("US")).contains("not the root of a document"));
}
