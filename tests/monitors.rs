// A monitor in Rust against the document the TypeScript SDK's monitors.test.ts pins for the same elements: a class is a
// function, an instance one call, and the declaration is the MonitorDoc; values out of range are refused.

use commandagi::design::monitors::*;
use commandagi::design::{declare, Declared};

fn document(root: El) -> (String, Json) {
    match declare(root).unwrap() {
        Declared::Document { format, document, .. } => (format, document),
        Declared::Graph(_) => panic!("a monitor is a document"),
    }
}

fn j(text: &str) -> Json {
    Json::parse(text).unwrap()
}

fn corporation(cik: &str, ticker: &str, name: &str) -> El {
    monitor([
        source().id("filings").adapter("sec-filings").params(json(&format!(r#"{{"cik": "{cik}"}}"#))).every("15m"),
        follows().monitor("../../central-banks/fed").match_("fomc.*"),
        wake().match_("sec.filing.8-k*").prompt("Say what the report changes."),
    ])
    .title(format!("{name} ({ticker})"))
    .class("corporation")
    .about(format!("cik:{cik}"))
}

#[test]
fn a_monitor_is_its_parts() {
    let (format, doc) = document(corporation("0001045810", "NVDA", "NVIDIA"));
    assert_eq!(format, "monitor");
    assert_eq!(
        doc,
        j(r#"{"title": "NVIDIA (NVDA)", "class": "corporation", "about": "cik:0001045810",
             "sources": [{"id": "filings", "adapter": "sec-filings", "params": {"cik": "0001045810"}, "every": "15m"}],
             "follows": [{"monitor": "../../central-banks/fed", "match": "fomc.*"}],
             "wakes": [{"match": "sec.filing.8-k*", "prompt": "Say what the report changes."}]}"#)
    );
}

#[test]
fn values_out_of_range_are_refused_by_name() {
    let one = |part: El| declare(monitor([part])).unwrap_err();
    assert!(one(source().id("s").adapter("fred").every("10s")).contains("every is from 30s to 7d"));
    assert!(one(source().id("s").adapter("fred").every("hourly")).contains("every is a duration"));
    assert!(one(follows().monitor(".")).contains("names this monitor"));
    assert!(declare(monitor([]).id("economy/cpi")).unwrap_err().contains("id"));
    assert_eq!(duration_seconds("6h"), Some(21_600));
    assert_eq!(duration_seconds("0m"), None);
}
