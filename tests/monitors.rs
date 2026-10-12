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

#[test]
fn paths_and_their_fields() {
    let one = |part: El| match document(monitor([part])).1.get("sources") {
        Some(Json::Arr(a)) => a[0].clone(),
        _ => panic!("sources"),
    };
    let s = || source().id("s").adapter("x-profile").every("1h");
    assert_eq!(
        one(s().via("browser").host("studio").session("../../../infra/sessions/x-web/session.tsx")),
        j(r#"{"id": "s", "adapter": "x-profile", "every": "1h", "via": "browser", "host": "studio",
             "session": "../../../infra/sessions/x-web/session.tsx"}"#)
    );
    let vm = one(s().via("vm").snapshot("x-logged-in").keep("10m").max_cost_per_day(2.5));
    assert_eq!(vm.get("maxCostPerDay"), Some(&Json::Num(2.5)));
    let phone = one(s().via("phone").unit("../../../worlds/fleet/world.tsx#android-01"));
    let doc = Json::obj().with("sources", Json::Arr(vec![phone])).with("follows", Json::Arr(vec![])).with("wakes", Json::Arr(vec![]));
    assert_eq!(
        monitor_with_defaults(&doc),
        j(r#"{"title": null, "class": null, "about": null, "readme": null,
             "sources": [{"id": "s", "adapter": "x-profile", "params": {}, "every": "1h", "via": "phone", "exit": null,
               "host": null, "mode": "poll", "retain": null, "since": null, "session": null, "snapshot": null,
               "profile": null, "keep": null, "maxCostPerDay": null, "unit": "../../../worlds/fleet/world.tsx#android-01"}],
             "follows": [], "wakes": []}"#)
    );
    let refused = |part: El, words: &str| {
        let e = declare(monitor([part])).unwrap_err();
        assert!(e.contains(words), "{e} should say {words}");
    };
    refused(s().via("vm").snapshot("x"), "costs money: it says maxCostPerDay");
    refused(s().via("vm").max_cost_per_day(1), "launches from a snapshot: it names a session or a snapshot");
    refused(s().via("android-vm").snapshot("x").max_cost_per_day(0), "maxCostPerDay is a price in USD");
    refused(s().via("worker").max_cost_per_day(1), "maxCostPerDay bounds what");
    refused(s().via("browser").profile("p").session("a/session.tsx"), "a session or a profile");
    refused(s().via("host").profile("p"), "for via=\"browser\" only");
    refused(s().via("vpn").exit("DE").host("h"), "host names the host that reads it");
    refused(s().via("browser").snapshot("x"), "snapshot is what");
    refused(s().via("vm").snapshot("x").max_cost_per_day(1).keep("5s"), "keep is a duration");
    refused(s().via("browser").keep("10m"), "keep is how long");
    refused(s().via("phone"), "names its phone: unit=\"<world.tsx>#<uid>\"");
    refused(s().via("phone").unit("android-01"), "unit is \"<world.tsx ref>#<uid>\"");
    refused(s().via("worker").unit("../w/world.tsx#a"), "unit names the phone");
    refused(s().via("browser").mode("stream"), "mode=\"stream\" holds a connection open");
    refused(s().via("ssh"), "via is \"worker\", \"host\", \"vpn\", \"browser\", \"vm\", \"android-vm\", \"phone\"");
}
