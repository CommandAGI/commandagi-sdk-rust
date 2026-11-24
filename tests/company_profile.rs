// A company's legal profile (website, email, phone, address(), listing(), stock(), owner()), read as the same document as the
// TypeScript SDK's JSX. company_profile.json is the golden of the three SDKs
// (sdk/typescript/src/design/company-profile.golden.json, sdk/python/tests/company_profile.json).

use commandagi::design::business::*;
use commandagi::design::{declare, Declared};

fn with_attrs(mut el: El, attrs: &Json) -> El {
    let Json::Obj(fields) = attrs else { panic!("attrs is an object") };
    for (k, v) in fields.iter() {
        el = el.attr(k.clone(), v.clone());
    }
    el
}

fn element(tag: &str, attrs: &Json) -> El {
    let el = match tag {
        "Entity" => entity(),
        "Address" => address(),
        "Listing" => listing(),
        "Stock" => stock(),
        "Owner" => owner(),
        _ => panic!("no tag {tag}"),
    };
    with_attrs(el, attrs)
}

fn company_of(props: &Json, children: &[Json]) -> Result<Json, String> {
    let parts: Vec<El> = children
        .iter()
        .map(|c| {
            let Some(Json::Str(tag)) = c.get("tag") else { panic!("a tag") };
            element(tag, c.get("attrs").unwrap())
        })
        .collect();
    match declare(with_attrs(company(parts), props))? {
        Declared::Document { document, .. } => Ok(document),
        Declared::Graph(_) => panic!("a company is a document"),
    }
}

#[test]
fn the_golden_of_the_three_sdks() {
    let golden = Json::parse(include_str!("company_profile.json")).unwrap();
    let rows = |k: &str| match golden.get(k) {
        Some(Json::Arr(a)) => a.clone(),
        _ => panic!("the golden has {k}"),
    };
    for row in rows("valid") {
        let Some(Json::Arr(children)) = row.get("children") else { panic!("children") };
        let d = company_of(row.get("company").unwrap(), children).unwrap();
        let Some(Json::Obj(want)) = row.get("document") else { panic!("document") };
        for (field, value) in want.iter() {
            assert_eq!(d.get(field).cloned().unwrap_or(Json::Null), value.clone(), "{field}");
        }
    }
    let plain = Json::parse(r#"[{"tag": "Entity", "attrs": {"jurisdiction": "US-DE"}}]"#).unwrap();
    let Json::Arr(entity_only) = plain else { panic!() };
    for row in rows("invalid") {
        let Some(Json::Str(error)) = row.get("error") else { panic!("an error") };
        let mut children = entity_only.clone();
        children.push(row.clone());
        let got = company_of(&Json::parse(r#"{"name": "Acme"}"#).unwrap(), &children).unwrap_err();
        assert!(got.contains(error.as_str()), "{got} should say {error}");
    }
}
