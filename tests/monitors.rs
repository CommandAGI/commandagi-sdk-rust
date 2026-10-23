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
    .same_as(json(&format!(r#"["ticker:{ticker}"]"#)))
}

#[test]
fn a_monitor_is_its_parts() {
    let (format, doc) = document(corporation("0001045810", "NVDA", "NVIDIA"));
    assert_eq!(format, "monitor");
    assert_eq!(
        doc,
        j(r#"{"title": "NVIDIA (NVDA)", "class": "corporation", "about": "cik:0001045810", "sameAs": ["ticker:NVDA"],
             "sources": [{"id": "filings", "adapter": "sec-filings", "params": {"cik": "0001045810"}, "every": "15m"}],
             "follows": [{"monitor": "../../central-banks/fed", "match": "fomc.*"}],
             "derives": [],
             "wakes": [{"match": "sec.filing.8-k*", "prompt": "Say what the report changes."}],
             "discovers": []}"#)
    );
    assert!(declare(monitor([]).same_as(json(r#"["NVDA"]"#))).unwrap_err().contains("sameAs is a list of subjects"));
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
        j(r#"{"title": null, "class": null, "about": null, "readme": null, "sameAs": [],
             "sources": [{"id": "s", "adapter": "x-profile", "params": {}, "every": "1h", "via": "phone", "exit": null,
               "host": null, "mode": "poll", "retain": null, "since": null, "session": null, "snapshot": null,
               "profile": null, "keep": null, "maxCostPerDay": null, "unit": "../../../worlds/fleet/world.tsx#android-01",
               "vantage": null}],
             "follows": [], "derives": [], "wakes": [], "discovers": []}"#)
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

// The TypeScript SDK's derive case and its mirror case: a derive names an operator and a folder per port; a source
// names its vantage, a follow filters, a discover writes one folder per conversation, and the exploits' discover reads
// three feeds and says when in data.
#[test]
fn a_derive_and_a_discover() {
    let (_, doc) = document(monitor([derive().id("vwap").op("vwap").inputs(json(r#"{"trades": "../trades"}"#)).params(json(r#"{"window": "1m"}"#))]));
    assert_eq!(doc.get("derives"), Some(&j(r#"[{"id": "vwap", "op": "vwap", "inputs": {"trades": "../trades"}, "params": {"window": "1m"}}]"#)));
    assert_eq!(monitor_with_defaults(&doc).get("derives"), Some(&j(r#"[{"id": "vwap", "op": "vwap", "inputs": {"trades": "../trades"}, "params": {"window": "1m"}, "since": null}]"#)));
    assert!(declare(monitor([derive().id("d").op("last").inputs(json("{}"))])).unwrap_err().contains("inputs maps each port"));
    // A port may name this monitor (an exploit's lifecycle from its own reads), as in the TypeScript SDK.
    declare(monitor([derive().id("d").op("consolidate").inputs(json(r#"{"own": "./"}"#))])).unwrap();

    let (_, mirror) = document(
        monitor([
            source().id("user-api").adapter("telegram-user").params(json(r#"{"account": "commandagi"}"#)).every("1m").via("host").vantage("../../../infra/vantages/laptop-linux"),
            discover()
                .when(json(r#"{"class": "message.*"}"#))
                .slug("conversation")
                .class("../../conversation.tsx#Conversation")
                .props(json(r#"{"conversation": "conversation", "title": "fromName"}"#))
                .name("fromName")
                .max(5),
        ])
        .title("Telegram: commandagi")
        .class("mirror"),
    );
    let full = monitor_with_defaults(&mirror);
    assert_eq!(
        full.get("discovers"),
        Some(&j(r#"[{"into": "./", "of": [], "when": {"class": "message.*"}, "slug": "conversation", "class": "../../conversation.tsx#Conversation",
                    "props": {"conversation": "conversation", "title": "fromName"}, "name": "fromName", "max": 5}]"#))
    );
    let Some(Json::Arr(sources)) = full.get("sources") else { panic!("sources") };
    assert_eq!(sources[0].get("vantage"), Some(&Json::Str("../../../infra/vantages/laptop-linux".into())));
    let (_, conversation) = document(monitor([follows().monitor("../").match_("message.*").where_("conversation=4412")]).title("Alice").class("conversation"));
    assert_eq!(conversation.get("follows"), Some(&j(r#"[{"monitor": "../", "match": "message.*", "where": "conversation=4412"}]"#)));

    let d = || discover().slug("conversation").class("../c.tsx#Conversation");
    declare(monitor([d()])).unwrap();
    declare(monitor([d()
        .of(json(r#"["./kev", "./cve-feed"]"#))
        .into_("./exploits")
        .when(json(r#"[{"class": "security.kev"}, {"class": "security.cve", "data.cvss3": {"gte": 9}}]"#))
        .slug(json(r#"{"paths": ["data.cve", "data.id", "data.aliases"], "pattern": "^CVE-"}"#))]))
    .unwrap();
    let refused = |part: El, words: &str| {
        let e = declare(monitor([part])).unwrap_err();
        assert!(e.contains(words), "{e} should say {words}");
    };
    refused(d().class("../c.tsx#conversation"), "class is the class file and its exported name");
    refused(d().class("../c.md#Conversation"), "class is the class file and its exported name");
    refused(d().slug("data[0]"), "slug is a data path");
    refused(d().slug(json(r#"{"paths": [], "pattern": "x"}"#)), "slug is a data path");
    refused(d().when(json(r#"{"cvss": 9}"#)), "when: \"cvss\" is class, about, key, title or a data path");
    refused(d().when(json(r#"{"data.cvss": {"above": 9}}"#)), "when: data.cvss: above is not an operator");
    refused(d().of(json("[]")), "of is a monitor's folder or a list of them");
    refused(d().props(json(r#"{"title": "a b"}"#)), "props.title is a data path");
    refused(d().max(500), "max is a whole number from 1 to 50");
    refused(follows().monitor("../x").where_("conversation"), "where is name=value terms");
}
