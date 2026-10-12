// Monitor blueprints in Rust against the document the TypeScript SDK's monitors.test.ts pins for the same elements: a
// class is a function, its instances data, and the declaration is the MonitorsDoc; values out of range are refused.

use commandagi::design::monitors::*;
use commandagi::design::{declare, Declared};

fn document(root: El) -> (String, Json) {
    match declare(root).unwrap() {
        Declared::Document { format, document, .. } => (format, document),
        Declared::Graph(_) => panic!("a blueprint is a document"),
    }
}

fn j(text: &str) -> Json {
    Json::parse(text).unwrap()
}

fn series(id: &str, title: &str, code: &str) -> El {
    monitor([source().id("fred").adapter("fred").params(json(&format!(r#"{{"series": "{code}"}}"#))).every("6h")])
        .id(format!("economy/{id}"))
        .title(title)
        .class("economic-series")
        .about(format!("fred:{code}"))
}

#[test]
fn a_blueprint_is_its_monitors() {
    let root = monitors([
        fragment([("cpi", "CPI, all items", "CPIAUCSL"), ("unrate", "Unemployment rate", "UNRATE")].map(|(i, t, c)| series(i, t, c))),
        monitor([
            source().id("dgs10").adapter("fred").params(json(r#"{"series": "DGS10"}"#)).every("6h"),
            follows().monitor("economy/cpi").match_("fred.*"),
            wake().match_("fred.observation").prompt("Say what moved."),
        ])
        .id("economy/treasury-yields")
        .title("Treasury yields")
        .class("interest-rate")
        .visibility("unlisted"),
    ])
    .id("economy")
    .title("Economy");
    let (format, doc) = document(root);
    assert_eq!(format, "monitors");
    assert_eq!(
        doc,
        j(r#"{"id": "economy", "title": "Economy", "monitors": [
            {"id": "economy/cpi", "title": "CPI, all items", "class": "economic-series", "about": "fred:CPIAUCSL",
             "sources": [{"id": "fred", "adapter": "fred", "params": {"series": "CPIAUCSL"}, "every": "6h"}], "follows": [], "wakes": []},
            {"id": "economy/unrate", "title": "Unemployment rate", "class": "economic-series", "about": "fred:UNRATE",
             "sources": [{"id": "fred", "adapter": "fred", "params": {"series": "UNRATE"}, "every": "6h"}], "follows": [], "wakes": []},
            {"id": "economy/treasury-yields", "title": "Treasury yields", "class": "interest-rate", "visibility": "unlisted",
             "sources": [{"id": "dgs10", "adapter": "fred", "params": {"series": "DGS10"}, "every": "6h"}],
             "follows": [{"monitor": "economy/cpi", "match": "fred.*"}],
             "wakes": [{"match": "fred.observation", "prompt": "Say what moved."}]}]}"#)
    );
}

#[test]
fn values_out_of_range_are_refused_by_name() {
    let one = |part: El| declare(monitors([monitor([part]).id("m")]).id("b")).unwrap_err();
    assert!(one(source().id("s").adapter("fred").every("10s")).contains("every is from 30s to 7d"));
    assert!(one(source().id("s").adapter("fred").every("hourly")).contains("every is a duration"));
    assert!(one(follows().monitor("m")).contains("follows itself"));
    assert!(declare(monitors([monitor([]).id("m").visibility("secret")]).id("b")).unwrap_err().contains(r#"visibility is "public", "unlisted", "private""#));
    assert_eq!(duration_seconds("6h"), Some(21_600));
    assert_eq!(duration_seconds("0m"), None);
}
