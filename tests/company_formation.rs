// A company's registered_agent() children, a filing paper()'s attributes and entity().dissolved(…), read as the same document as the
// TypeScript SDK's JSX. company_formation.json is the golden of the three SDKs
// (sdk/typescript/src/design/company-formation.golden.json, sdk/python/tests/company_formation.json).

use commandagi::design::business::*;
use commandagi::design::{declare, Declared};

fn element(tag: &str, attrs: &Json) -> El {
    let Json::Obj(fields) = attrs else { panic!("attrs is an object") };
    let mut el = match tag {
        "Paper" => paper([]),
        "RegisteredAgent" => registered_agent(),
        "Entity" => entity(),
        _ => panic!("no tag {tag}"),
    };
    for (k, v) in fields.iter() {
        el = el.attr(k.clone(), v.clone());
    }
    el
}

fn company_of(entity: &Json, children: &[Json]) -> Result<Json, String> {
    let mut parts = vec![element("Entity", entity)];
    for c in children {
        let Some(Json::Str(tag)) = c.get("tag") else { panic!("a tag") };
        parts.push(element(tag, c.get("attrs").unwrap()));
    }
    match declare(company(parts).name("Acme"))? {
        Declared::Document { document, .. } => Ok(document),
        Declared::Graph(_) => panic!("a company is a document"),
    }
}

#[test]
fn the_golden_of_the_three_sdks() {
    let golden = Json::parse(include_str!("company_formation.json")).unwrap();
    let rows = |k: &str| match golden.get(k) {
        Some(Json::Arr(a)) => a.clone(),
        _ => panic!("the golden has {k}"),
    };
    for row in rows("valid") {
        let Some(Json::Arr(children)) = row.get("children") else { panic!("children") };
        let d = company_of(row.get("entity").unwrap(), children).unwrap();
        let want = row.get("document").unwrap();
        for field in ["entity", "papers", "registeredAgents"] {
            assert_eq!(d.get(field).cloned().unwrap_or(Json::Null), want.get(field).cloned().unwrap_or(Json::Null), "{field}");
        }
    }
    for row in rows("invalid") {
        let Some(Json::Str(error)) = row.get("error") else { panic!("an error") };
        let got = company_of(&Json::parse(r#"{"jurisdiction": "US-DE"}"#).unwrap(), &[row.clone()]).unwrap_err();
        assert!(got.contains(error.as_str()), "{got} should say {error}");
    }
}
