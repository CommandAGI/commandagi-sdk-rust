// A script in Rust against the document the TypeScript SDK's scripts.test.ts pins for the same elements: a class is a
// function, an instance one call, and the declaration is the ScriptDoc; a tail names a file, never a folder; values
// out of range are refused by name.

use commandagi::design::scripts::*;
use commandagi::design::{declare, Declared};

fn document(root: El) -> (String, Json) {
    match declare(root).unwrap() {
        Declared::Document { format, document, .. } => (format, document),
        Declared::Graph(_) => panic!("a script is a document"),
    }
}

fn j(text: &str) -> Json {
    Json::parse(text).unwrap()
}

fn with_attrs(mut el: El, attrs: &Json) -> El {
    let Json::Obj(fields) = attrs else { panic!("attrs is an object") };
    for (k, v) in fields.iter() {
        el = el.attr(k.clone(), v.clone());
    }
    el
}

/// The document back to code: each list's entries as their elements, in the vocabulary's order.
fn code_of(doc: &Json) -> El {
    let mut parts = Vec::new();
    for (list, part) in [("sources", source as fn() -> El), ("tails", tail), ("derives", derive), ("wakes", wake), ("discovers", discover), ("judges", judge), ("params", param)] {
        let Some(Json::Arr(entries)) = doc.get(list) else { panic!("{list} is a list") };
        parts.extend(entries.iter().map(|e| with_attrs(part(), e)));
    }
    let mut fields = Json::obj();
    for k in SCRIPT_FIELDS {
        if let Some(v) = doc.get(k) {
            fields.set(k, v.clone());
        }
    }
    with_attrs(script(parts), &fields)
}

fn refused(part: El, words: &str) {
    let e = declare(script([part])).unwrap_err();
    assert!(e.contains(words), "{e} should say {words}");
}

fn corporation(cik: &str, ticker: &str, name: &str) -> El {
    script([
        source().id("filings").adapter("sec-filings").params(json(&format!(r#"{{"cik": "{cik}"}}"#))).every("30m"),
        tail().of("../../central-banks/fed/fed.script.tsx").when(json(r#"{"class": "fomc.*"}"#)),
        wake().when(json(r#"{"class": "sec.filing.8-k*"}"#)).prompt("Say what the report changes."),
    ])
    .title(format!("{name} ({ticker})"))
    .class("corporation")
    .about(format!("cik:{cik}"))
    .same_as(json(&format!(r#"["ticker:{ticker}"]"#)))
}

#[test]
fn a_script_is_its_parts_both_ways() {
    let (format, doc) = document(corporation("0001045810", "NVDA", "NVIDIA"));
    assert_eq!(format, "script");
    assert_eq!(
        doc,
        j(r#"{"title": "NVIDIA (NVDA)", "class": "corporation", "about": "cik:0001045810", "sameAs": ["ticker:NVDA"],
             "sources": [{"id": "filings", "adapter": "sec-filings", "params": {"cik": "0001045810"}, "every": "30m"}],
             "tails": [{"of": "../../central-banks/fed/fed.script.tsx", "when": {"class": "fomc.*"}}],
             "derives": [],
             "wakes": [{"when": {"class": "sec.filing.8-k*"}, "prompt": "Say what the report changes."}],
             "discovers": [], "judges": [], "params": []}"#)
    );
    assert_eq!(document(code_of(&doc)).1, doc, "both ways");
    let full = script_with_defaults(&doc);
    assert_eq!(
        full.get("sources"),
        Some(&j(r#"[{"id": "filings", "adapter": "sec-filings", "params": {"cik": "0001045810"}, "every": "30m", "via": "worker", "exit": null,
                    "host": null, "mode": "poll", "retain": null, "since": null, "session": null, "snapshot": null, "profile": null, "keep": null,
                    "maxCostPerDay": null, "unit": null, "vantage": null, "connection": null, "stream": "records"}]"#))
    );
    assert_eq!(full.get("tails"), Some(&j(r#"[{"of": "../../central-banks/fed/fed.script.tsx", "when": {"class": "fomc.*"}, "stream": "records"}]"#)));
    assert_eq!(full.get("wakes"), Some(&j(r#"[{"when": {"class": "sec.filing.8-k*"}, "every": null, "prompt": "Say what the report changes.", "agent": null, "maxSpend": null}]"#)));
}

#[test]
fn an_empty_script_its_file_name_and_a_part_that_names_another_stream() {
    let (_, doc) = document(script([]));
    assert_eq!(doc, j(r#"{"sources": [], "tails": [], "derives": [], "wakes": [], "discovers": [], "judges": [], "params": []}"#));
    for name in ["nvidia.script.tsx", "a.script.jsx", "b.script.py", "c.script.rs"] {
        assert!(script_file_ok(name), "{name}");
    }
    for name in ["monitor.tsx", "script.tsx", "nvidia.tsx"] {
        assert!(!script_file_ok(name), "{name}");
    }
    let (_, ticks) = document(script([source().id("q").adapter("fred").every("1h").stream("ticks")]));
    let Some(Json::Arr(sources)) = script_with_defaults(&ticks).get("sources").cloned() else { panic!("sources") };
    assert_eq!(sources[0].get("stream"), Some(&Json::Str("ticks".into())));
    refused(source().id("q").adapter("fred").every("1h").stream("Ticks!"), "stream is a word");
}

#[test]
fn a_tail_names_a_file_a_script_a_runs_records_file_or_a_series_never_a_folder() {
    let one = |of: &str| declare(script([tail().of(of)]));
    for of in [
        "../nvidia/nvidia.script.tsx",
        "cloud://home/user-commandagi/entities/fed/fed.script.tsx",
        "../x/runs/thread-1/streams/records@cloud/records.jsonl",
        "fred:DGS10",
        "ecb:FM.D.U2.EUR.4F.KR.DFR.LEV",
    ] {
        one(of).unwrap_or_else(|e| panic!("{of}: {e}"));
    }
    let e = one("../nvidia").unwrap_err();
    assert!(e.contains(r#"tail().of("../nvidia") names a file: a script ("../nvidia/nvidia.script.tsx")"#), "{e}");
    assert!(one("./").unwrap_err().contains("names this script's folder"));
    assert!(one("../nvidia/monitor.tsx").unwrap_err().contains("names a file"));
    assert_eq!(tail_ref_problem("fred:DGS10"), None);
    let e = declare(script([tail().of("../x/x.script.tsx").attr("match", "a.*")])).unwrap_err();
    assert!(e.contains("is not read (of, when, stream)"), "{e}");
    assert!(declare(script([El::new("follows").attr("monitor", "../x")])).unwrap_err().contains("follows"));
    refused(tail().of("../x/x.script.tsx").when(json(r#"{"data.n": {"above": 1}}"#)), "when: data.n: above is not an operator");
}

#[test]
fn a_source_refuses_by_name_what_it_has_no_words_for_and_values_out_of_range() {
    let src = || source().id("s").adapter("fred").every("1h");
    let e = declare(script([src(), src()])).unwrap_err();
    assert!(e.contains(r#"two source() in script() have id "s""#), "{e}");
    assert!(declare(script([]).same_as(json(r#"["NVDA"]"#))).unwrap_err().contains("sameAs is a list of subjects"));
    refused(source().id("s").adapter("fred"), "needs every");
    refused(src().every("10s"), "every is from 30s to 7d");
    refused(src().via("ftp"), r#"via is "worker", "host", "vpn", "browser", "vm", "android-vm", "phone""#);
    refused(src().mode("stream"), r#"mode="stream" holds a connection open, which runs on a host"#);
    refused(src().via("vm").snapshot("genesis"), r#"via="vm" costs money: it says maxCostPerDay"#);
    refused(src().via("phone"), r#"via="phone" names its phone: unit="#);
    refused(src().via("host").connection("gmail@jacobfv123@gmail.com"), "connection names a connected account, which the api worker reads");
    // The id is the file's path, and who reads the run is the file's Drive visibility: neither is a word of the file.
    assert!(declare(script([]).id("economy/cpi")).unwrap_err().contains("id is not read"));
    assert!(declare(script([]).visibility("public")).unwrap_err().contains("visibility is not read"));
}

#[test]
fn a_derive_names_an_operator_and_each_ports_file_a_discover_the_files_it_reads() {
    let (_, doc) = document(
        script([
            derive().id("spread").op("spread").inputs(json(r#"{"a": "fred:SOFR", "b": "fred:IORB"}"#)),
            derive().id("own").op("last").inputs(json(r#"{"records": "./"}"#)).since("2026-10-01").stream("derived"),
            discover()
                .of("./kev/kev.script.tsx")
                .into_("./exploits")
                .when(json(r#"{"class": "security.kev"}"#))
                .slug("data.cve")
                .class("./exploit.tsx#Exploit")
                .props(json(r#"{"cve": "data.cve"}"#))
                .max(20)
                .visibility("public"),
        ])
        .title("SOFR minus IORB"),
    );
    assert_eq!(document(code_of(&doc)).1, doc, "both ways");
    let full = script_with_defaults(&doc);
    assert_eq!(
        full.get("derives"),
        Some(&j(r#"[{"id": "spread", "op": "spread", "inputs": {"a": "fred:SOFR", "b": "fred:IORB"}, "params": {}, "since": null, "stream": "records"},
                    {"id": "own", "op": "last", "inputs": {"records": "./"}, "params": {}, "since": "2026-10-01", "stream": "derived"}]"#))
    );
    assert_eq!(
        full.get("discovers"),
        Some(&j(r#"[{"into": "./exploits", "of": ["./kev/kev.script.tsx"], "when": {"class": "security.kev"}, "slug": "data.cve",
                    "class": "./exploit.tsx#Exploit", "props": {"cve": "data.cve"}, "name": null, "max": 20, "visibility": "public"}]"#))
    );
    let d = |inputs: &str| derive().id("d").op("last").inputs(json(inputs));
    refused(d("{}"), "inputs maps each port to a file it tails");
    refused(d(r#"{"a": "../quotes"}"#), "inputs.a names a file");
    let disc = || discover().slug("cve").class("../c.tsx#Exploit");
    declare(script([disc()])).unwrap();
    refused(disc().of("./kev"), "of names a file");
    refused(disc().of(json("[]")), "of is a script file or a list of them");
    refused(disc().max(500), "max is a whole number from 1 to 50");
}

#[test]
fn a_judge_names_what_it_judges_its_rubric_and_its_model() {
    let (_, doc) = document(script([judge().id("claims").of("task").rubric("./rubric.md").model("fireworks/glm-5p3")]));
    assert_eq!(script_with_defaults(&doc).get("judges"), Some(&j(r#"[{"id": "claims", "of": "task", "rubric": "./rubric.md", "model": "fireworks/glm-5p3"}]"#)));
    refused(judge().id("j").of("thread").rubric("r.md").model("m"), r#"of is "turn", "task""#);
}

#[test]
fn a_template_declares_its_params_each_a_typed_param_its_placeholders_name_both_ways() {
    let (_, doc) = document(
        script([
            param().id("cik").type_("string").label("the company's CIK"),
            param().id("ticker").type_("string").required(false).default("NVDA"),
            source().id("filings").adapter("sec-filings").params(json(r#"{"cik": "${cik}"}"#)).every("30m"),
        ])
        .title("${name} (${ticker})"),
    );
    assert_eq!(document(code_of(&doc)).1, doc, "both ways");
    assert_eq!(
        script_with_defaults(&doc).get("params"),
        Some(&j(r#"[{"id": "cik", "type": "string", "label": "the company's CIK", "default": null, "required": true, "choices": null, "accept": null},
                    {"id": "ticker", "type": "string", "label": null, "default": "NVDA", "required": false, "choices": null, "accept": null}]"#))
    );
    let p = || param().id("p").type_("string");
    refused(param().id("p").type_("date"), r#"type is "file""#);
    refused(param().id("a-b").type_("string"), "id is letters, digits and _");
    refused(param().id("p").type_("choice"), "a choice names its choices");
    declare(script([param().id("p").type_("choice").choices(json(r#"["a", "b"]"#))])).unwrap();
    let e = declare(script([p(), param().id("p").type_("number")])).unwrap_err();
    assert!(e.contains(r#"two param() in script() have id "p""#) || e.contains("declared twice"), "{e}");
}

#[test]
fn a_placeholder_as_a_whole_value_passes_the_file_checks() {
    // A template's ${name} is a file once its copy fills it.
    assert_eq!(tail_ref_problem("${of}"), None);
    let (_, doc) = document(script([
        param().id("of").type_("file"),
        param().id("cls").type_("string"),
        tail().of("${of}"),
        derive().id("d").op("last").inputs(json(r#"{"a": "${of}"}"#)),
        discover().of("${of}").slug("data.cve").class("${cls}"),
    ]));
    assert_eq!(doc.get("discovers").and_then(|d| match d {
        Json::Arr(a) => a[0].get("class").cloned(),
        _ => None,
    }), Some(Json::Str("${cls}".into())));
    refused(tail().of("../${of}"), "names a file");
    refused(discover().slug("cve").class("${a-b}"), "class is the class file");
}

// script_wake.json is the golden of the three SDKs (sdk/typescript/src/design/script-wake.golden.json,
// sdk/python/tests/script_wake.json). Rust writes a keyword attribute with a `_` (`match_`), so an error's
// `match_ is not read` is the golden's `match is not read`.
#[test]
fn a_wake_on_the_scripts_own_records_or_on_a_clock() {
    let golden = Json::parse(include_str!("script_wake.json")).unwrap();
    let rows = |k: &str| match golden.get(k) {
        Some(Json::Arr(a)) => a.clone(),
        _ => panic!("the golden has {k}"),
    };
    for row in rows("valid") {
        let (_, doc) = document(script([with_attrs(wake(), row.get("attrs").unwrap())]));
        assert_eq!(script_with_defaults(&doc).get("wakes"), Some(&Json::Arr(vec![row.get("document").unwrap().clone()])));
    }
    for row in rows("invalid") {
        let Some(Json::Str(error)) = row.get("error") else { panic!("an error") };
        let got = declare(script([with_attrs(wake(), row.get("attrs").unwrap())])).unwrap_err().replace("match_", "match");
        assert!(got.contains(error.as_str()), "{got} should say {error}");
    }
}
