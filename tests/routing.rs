use commandagi::design::routing::*;
use commandagi::design::{declare, Declared};

#[test]
fn routing_knowledge_has_four_record_groups() {
    let root = routing([
        prior().id("abstract-codex").region("abstract").harness("codex").quality(0.75).evidence("reported"),
        rule().id("cli-only").deny(true).reason("CLI login only"),
        price().id("api-model").model("provider/model").input_per_million(1.2),
        observation().id("sample").count(4).cost(0.2).evidence("checked"),
    ]);
    let Declared::Document { format, document, .. } = declare(root).unwrap() else { panic!("routing is a document") };
    assert_eq!(format, "routing");
    for group in ["priors", "rules", "prices", "observations"] {
        assert!(matches!(document.get(group), Some(Json::Arr(rows)) if rows.len() == 1));
    }
}

#[test]
fn duplicate_routing_record_ids_are_refused() {
    let root = routing([rule().id("same"), rule().id("same")]);
    assert!(declare(root).unwrap_err().contains("have id \"same\""));
}
