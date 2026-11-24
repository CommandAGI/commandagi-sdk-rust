// A company's team() in Rust against the `team` the TypeScript SDK's people-ops.test.ts pins for the same elements:
// each list in the file's order, each element's site by path; an unknown attribute or a misplaced element is refused.

use commandagi::design::business::*;
use commandagi::design::{declare, Declared};

fn document(root: El) -> (Json, Json) {
    match declare(root).unwrap() {
        Declared::Document { document, sources, .. } => (document, sources),
        Declared::Graph(_) => panic!("a company is a document"),
    }
}

#[test]
fn a_team_is_the_company_team() {
    let pilot = role().id("pilot").title("Drone pilot").reports_to("ceo").band("ic-2").headcount(2).kind("employee");
    let pilot_site = pilot.source_json().unwrap();
    let root = company([
        people().folder("Northwind/people/"),
        team([
            role().id("ceo").title("Managing member"),
            pilot,
            band().id("ic-2").currency("USD").per("year").min("80000").max("110000"),
            benefit().id("health").kind("health").eligible(["employee"]).waiting_days(30),
            checklist().id("onboarding").when("start").steps(["Sign the offer"]),
        ])
        .hiring("Northwind/hiring/"),
    ])
    .name("Northwind");
    let (doc, sources) = document(root);
    assert_eq!(
        doc.get("team"),
        Some(
            &Json::parse(
                r#"{"hiring": "Northwind/hiring/",
                    "roles": [{"id": "ceo", "title": "Managing member"}, {"id": "pilot", "title": "Drone pilot", "reportsTo": "ceo", "band": "ic-2", "headcount": 2, "kind": "employee"}],
                    "bands": [{"id": "ic-2", "currency": "USD", "per": "year", "min": "80000", "max": "110000"}],
                    "benefits": [{"id": "health", "kind": "health", "eligible": ["employee"], "waitingDays": 30}],
                    "checklists": [{"id": "onboarding", "when": "start", "steps": ["Sign the offer"]}]}"#
            )
            .unwrap()
        )
    );
    assert_eq!(sources.get("team/roles/1"), Some(&pilot_site));
    assert_eq!(document(company([]).name("Bare")).0.get("team"), None, "no team() declared: no team");
}

#[test]
fn a_team_refuses_what_it_does_not_say() {
    let err = declare(company([team([]), team([])]).name("N")).unwrap_err();
    assert!(err.contains("company() has two team()"), "{err}");
    let err = declare(company([role().id("x").title("X")]).name("N")).unwrap_err();
    assert!(err.contains("company() does not take role()"), "{err}");
    let err = declare(company([team([role().title("X")])]).name("N")).unwrap_err();
    assert!(err.contains("role() needs id"), "{err}");
}
