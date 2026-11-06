// A company's deals in Rust against the documents the TypeScript SDK pins. company_deals.json is the golden of the
// three SDKs (sdk/typescript/src/design/company-deals.golden.json, sdk/python/tests/company_deals.json).

use commandagi::design::business::*;
use commandagi::design::{declare, Declared};

fn element(tree: &Json) -> El {
    let Some(Json::Str(tag)) = tree.get("tag") else { panic!("a tree names its tag") };
    let children: Vec<El> = match tree.get("children") {
        Some(Json::Arr(list)) => list.iter().map(element).collect(),
        _ => Vec::new(),
    };
    let mut el = match tag.as_str() {
        "Company" => company(children),
        "Deal" => deal(),
        other => panic!("{other} is not in the golden's vocabulary"),
    };
    if let Some(Json::Obj(fields)) = tree.get("attrs") {
        for (k, v) in fields.iter() {
            el = el.attr(k.clone(), v.clone());
        }
    }
    el
}

fn golden(key: &str) -> Vec<Json> {
    match Json::parse(include_str!("company_deals.json")).unwrap().get(key) {
        Some(Json::Arr(rows)) => rows.clone(),
        _ => panic!("the golden has {key}"),
    }
}

#[test]
fn each_part_is_the_company_document_and_a_part_not_written_is_absent() {
    for row in golden("valid") {
        let Declared::Document { format, document, .. } = declare(element(row.get("tree").unwrap())).unwrap() else { panic!("a document") };
        assert_eq!(format, "company");
        assert_eq!(Some(&document), row.get("document"));
    }
}

#[test]
fn what_the_vocabulary_does_not_say_is_refused_by_name() {
    for row in golden("invalid") {
        let Some(Json::Str(error)) = row.get("error") else { panic!("an error") };
        let got = declare(element(row.get("tree").unwrap())).unwrap_err();
        assert!(got.contains(error.as_str()), "{got} should say {error}");
    }
}

#[test]
fn the_calls_chain_their_attributes() {
    let Declared::Document { document, .. } = declare(
        company([
            deal().id("acme-pilot").title("Pilot of ten units").counterparty("relationships/cards/acme.vcf").stage("proposal").value("120000").currency("USD").expected("2027-03-31").owner("ceo"),
            deal().id("county").title("County survey").counterparty("county").stage("lead"),
        ])
        .name("X"),
    )
    .unwrap() else {
        panic!("a document")
    };
    let Some(Json::Arr(deals)) = document.get("deals") else { panic!("the deals") };
    assert_eq!(deals.len(), 2);
    assert_eq!(deals[0].get("counterparty"), Some(&Json::Str("relationships/cards/acme.vcf".into())));
    assert_eq!(deals[1].get("value"), None);
}
