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
        "Deal" => deal(children),
        "DealMove" => deal_move(),
        "Pipeline" => pipeline(children),
        "DealStage" => deal_stage(),
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
            pipeline([deal_stage().id("lead").label("first call").probability(0.1), deal_stage().id("proposal").probability("0.5")]).stale_days(30),
            deal([deal_move().from("lead").to("proposal").at("2026-11-01T17:02:00Z").by("person:ada").note("the call went well")])
                .id("acme-pilot")
                .title("Pilot of ten units")
                .counterparty("relationships/cards/acme.vcf")
                .stage("proposal")
                .value("120000")
                .currency("USD")
                .expected("2027-03-31")
                .owner("ceo")
                .probability(0.3)
                .probability_reason("their budget is not approved"),
            deal([]).id("county").title("County survey").counterparty("county").stage("lead"),
        ])
        .name("X"),
    )
    .unwrap() else {
        panic!("a document")
    };
    let Some(Json::Arr(deals)) = document.get("deals") else { panic!("the deals") };
    assert_eq!(deals.len(), 2);
    assert_eq!(deals[0].get("counterparty"), Some(&Json::Str("relationships/cards/acme.vcf".into())));
    assert_eq!(deals[0].get("probabilityReason"), Some(&Json::Str("their budget is not approved".into())));
    let Some(Json::Arr(moves)) = deals[0].get("moves") else { panic!("the moves") };
    assert_eq!(moves[0].get("from"), Some(&Json::Str("lead".into())));
    assert_eq!(moves[0].get("by"), Some(&Json::Str("person:ada".into())));
    assert_eq!(deals[1].get("value"), None);
    assert_eq!(deals[1].get("moves"), None);
    let Some(pipe) = document.get("pipeline") else { panic!("the pipeline") };
    assert_eq!(pipe.get("staleDays"), Some(&Json::Num(30.0)));
    let Some(Json::Arr(stages)) = pipe.get("stages") else { panic!("the stages") };
    assert_eq!(stages[0].get("label"), Some(&Json::Str("first call".into())));
    assert_eq!(stages[1].get("probability"), Some(&Json::Str("0.5".into())));
}
