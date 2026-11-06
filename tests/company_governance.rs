// A company's seats, register, evidence, IP and policies, and the votes on its papers, in Rust against the documents
// the TypeScript SDK pins. company_governance.json is the golden of the three SDKs
// (sdk/typescript/src/design/company-governance.golden.json, sdk/python/tests/company_governance.json).

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
        "Paper" => paper(children),
        "Seat" => seat(),
        "Compliance" => compliance(),
        "Obligation" => obligation(),
        "Evidence" => evidence(),
        "IpAsset" => ip_asset(),
        "Entity" => entity(),
        "Product" => product(),
        "Relationships" => relationships(),
        "Assignment" => assignment(),
        "Policy" => policy(),
        "Vote" => vote(),
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
    match Json::parse(include_str!("company_governance.json")).unwrap().get(key) {
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
            seat().body("board").title("director").holder("a.vcf").start("2024-01-31"),
            policy().kind("cyber").carrier("A").start("2026-01-01").end("2027-01-01").limit(500000),
            evidence().obligation("own:x").due("2026-01-01").done("2026-01-01").file("x.pdf"),
            paper([vote().holder("a.vcf").choice("for").signed("2026-01-02")]).kind("board-consent").file("c.pdf").body("board").threshold("unanimous"),
        ])
        .name("X"),
    )
    .unwrap() else {
        panic!("a document")
    };
    assert!(document.get("insurance").is_some() && document.get("ip").is_none());
    assert_eq!(document.get("compliance").and_then(|c| c.get("industries")), Some(&Json::Arr(Vec::new())));
}
