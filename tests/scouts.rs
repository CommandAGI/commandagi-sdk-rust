// A scout's comparison and a replacement-cost analysis in Rust against the documents the TypeScript SDK's
// scouts.test.ts pins for the same examples: each is its record; what would make a run dishonest is refused by name (a
// customer's case, a case with two scorers, two incumbents, a check with no kind).

use commandagi::design::scouts::*;
use commandagi::design::{declare, Declared};

fn document(root: El) -> (String, Json, Json) {
    match declare(root).unwrap() {
        Declared::Document { format, document, sources } => (format, document, sources),
        Declared::Graph(_) => panic!("a comparison and a replacement analysis are documents"),
    }
}

fn comparison(first: El, more: Vec<El>) -> El {
    let mut children = vec![
        configuration().id("sonnet").model("anthropic/claude-sonnet-5-5").effort("low").incumbent(true),
        configuration().id("glm").model("dashscope/glm-5.3"),
        workload([
            first,
            case()
                .id("page")
                .from_("own-run")
                .source("runs/thread-1/streams/chat@a/records.jsonl#12")
                .prompt("Write the provider page.")
                .rubric("Names the price and the terms."),
        ])
        .id("knowledge")
        .title("Knowledge work")
        .sample(20)
        .needs(["structured"]),
        judge().model("openai/gpt-5.6-terra"),
    ];
    children.extend(more);
    scout(children).title("GLM against Sonnet").budget(10).repeats(3)
}

#[test]
fn a_comparison_declares_its_configurations_workloads_with_their_cases_and_judges() {
    let rate = case().id("rate").from_("synthetic").prompt("What is 4.2% as a decimal?").check(json(r#"{"number": 0.042, "tolerance": 0.0005}"#));
    let site = rate.source_json().unwrap();
    let (format, doc, sources) = document(comparison(rate, Vec::new()));
    assert_eq!(format, "scout");
    let expected = r#"{"title": "GLM against Sonnet", "budget": 10, "repeats": 3,
        "configurations": [{"id": "sonnet", "model": "anthropic/claude-sonnet-5-5", "effort": "low", "incumbent": true},
                           {"id": "glm", "model": "dashscope/glm-5.3"}],
        "workloads": [{"id": "knowledge", "title": "Knowledge work", "needs": ["structured"], "sample": 20, "cases": [
            {"id": "rate", "from": "synthetic", "prompt": "What is 4.2% as a decimal?", "check": {"number": 0.042, "tolerance": 0.0005}},
            {"id": "page", "from": "own-run", "prompt": "Write the provider page.", "rubric": "Names the price and the terms.",
             "source": "runs/thread-1/streams/chat@a/records.jsonl#12"}]}],
        "judges": [{"model": "openai/gpt-5.6-terra"}]}"#;
    assert_eq!(doc, Json::parse(expected).unwrap());
    assert_eq!(sources.get("workload#knowledge/case#rate"), Some(&site));
}

#[test]
fn a_comparison_refuses_what_would_make_its_verdict_dishonest() {
    let exact = || json(r#"{"exact": "1"}"#);
    let rate = || case().id("rate").from_("synthetic").prompt("p").check(exact());
    for (said, why) in [
        (declare(comparison(case().id("rate").from_("customer").prompt("p").check(exact()), Vec::new())).unwrap_err(), "from is one of public, synthetic, own-run"),
        (declare(comparison(rate().rubric("and a judge"), Vec::new())).unwrap_err(), "exactly one of check"),
        (declare(comparison(rate().check(json(r#"{"tolerance": 1}"#)), Vec::new())).unwrap_err(), "check is exactly one of"),
        (declare(comparison(rate().check(json(r#"{"exact": "a", "contains": ["b"]}"#)), Vec::new())).unwrap_err(), "it has exact and contains"),
        (declare(comparison(rate(), vec![configuration().id("third").model("openai/gpt-6-luna").incumbent(true)])).unwrap_err(), "one incumbent"),
        (declare(scout([]).budget(0)).unwrap_err(), "budget is a number above 0"),
        (declare(scout([]).title("no budget")).unwrap_err(), "scout() needs budget"),
        (declare(scout([case().id("x").from_("public").prompt("p").check(exact())]).budget(5)).unwrap_err(), "case() stands in"),
    ] {
        assert!(said.contains(why), "{why}: {said}");
    }
}

#[test]
fn a_tool_case_offers_its_tools_and_their_answers() {
    let tools_text = r#"[{"name": "get_capital", "parameters": {"type": "object", "properties": {"country": {"type": "string"}}},
             "results": [{"when": {"country": "france"}, "result": "Paris"}]},
            {"name": "send_email", "description": "send an email", "parameters": {"type": "object"}}]"#;
    let tools = json(tools_text);
    let (_, doc, _) = document(
        scout([workload([
            case().id("chain").from_("synthetic").prompt("p").tools(tools.clone()).turns(4).check(json(r#"{"number": 4280000}"#)),
            case().id("first").from_("synthetic").prompt("p").tools(tools.clone()).check(json(r#"{"tool": "send_email"}"#)),
            case().id("mc").from_("public").prompt("p").check(json(r#"{"letter": "I"}"#)),
            case().id("prog").from_("public").prompt("p").check(json(r#"{"program": {"data": [[1, 4], [2, 7], [5, 16]]}}"#)),
        ])
        .id("agentic")])
        .budget(1),
    );
    let Json::Arr(workloads) = doc.get("workloads").expect("workloads") else { panic!("workloads is a list") };
    let Json::Arr(cases) = workloads[0].get("cases").expect("cases") else { panic!("cases is a list") };
    assert_eq!(cases[0].get("tools"), Some(&Json::parse(tools_text).unwrap()));
    assert_eq!(cases[0].get("turns"), Some(&Json::Num(4.0)));

    let one = |given: El| declare(scout([workload([given.id("c").from_("synthetic").prompt("p")]).id("w")]).budget(1)).unwrap_err();
    for (said, why) in [
        (one(case().check(json(r#"{"tool": "send_email"}"#))), "tool needs the case's tools"),
        (one(case().check(json(r#"{"letter": "ab"}"#))), "letter is one capital letter"),
        (one(case().check(json(r#"{"exact": "x"}"#)).turns(3)), "turns goes with tools"),
        (one(case().check(json(r#"{"exact": "x"}"#)).tools(json(r#"[{"name": "a b", "parameters": {}}]"#))), "a tool's name is a word"),
        (one(case().check(json(r#"{"program": {"data": []}}"#))), "program is { data: [[x, y], …] }"),
        (one(case().check(json(r#"{"program": {"data": [[1, 4]], "tolerance": -1}}"#))), "program tolerance is a number of 0 or more"),
    ] {
        assert!(said.contains(why), "{why}: {said}");
    }
}

#[test]
fn a_replacement_analysis_declares_its_layers_capabilities_with_evidence_and_its_decision() {
    let (format, doc, _) = document(
        replacement([
            layer([
                capability([evidence().source("beehiiv checklist").claim("reported")])
                    .id("bulk-send")
                    .title("3M+ emails a minute")
                    .need("must")
                    .have("none")
                    .effort("XL")
                    .run(2000)
                    .labour(160)
                    .note("ESP or own MTAs"),
                capability([]).id("abuse").title("signup validation").need("must").have("partial").effort("L").ref_("docs/moderation.md"),
            ])
            .kind("invisible")
            .title("Deliverability"),
            layer([capability([]).id("collab").title("real-time collaboration").need("should").have("partial").effort("M")])
                .kind("visible")
                .title("Editor"),
            layer([]).kind("visible").title("Website"),
            decision().choice("skip").because("The invisible layer is the product."),
            differentiation().text("Agents author the newsletter as code."),
            economics().price("$49 a month").margin("unknown"),
        ])
        .product("beehiiv")
        .vendor("beehiiv, Inc.")
        .url("https://www.beehiiv.com")
        .as_of("2026-10-14")
        .by_("scout"),
    );
    assert_eq!(format, "replacement");
    let expected = r#"{"product": "beehiiv", "vendor": "beehiiv, Inc.", "url": "https://www.beehiiv.com", "asOf": "2026-10-14", "by": "scout",
        "layers": [
            {"kind": "invisible", "title": "Deliverability", "capabilities": [
                {"id": "bulk-send", "title": "3M+ emails a minute", "need": "must", "have": "none", "effort": "XL", "run": 2000,
                 "labour": 160, "note": "ESP or own MTAs", "evidence": [{"source": "beehiiv checklist", "claim": "reported"}]},
                {"id": "abuse", "title": "signup validation", "need": "must", "have": "partial", "effort": "L", "ref": "docs/moderation.md",
                 "evidence": []}]},
            {"kind": "visible", "title": "Editor", "capabilities": [
                {"id": "collab", "title": "real-time collaboration", "need": "should", "have": "partial", "effort": "M", "evidence": []}]},
            {"kind": "visible", "title": "Website", "capabilities": []}],
        "decision": {"choice": "skip", "because": "The invisible layer is the product."},
        "differentiation": "Agents author the newsletter as code.",
        "economics": {"price": "$49 a month", "margin": "unknown"}}"#;
    assert_eq!(doc, Json::parse(expected).unwrap());
}

#[test]
fn a_replacement_analysis_refuses_a_value_its_sums_cannot_read() {
    let one = |c: El| declare(replacement([layer([c]).kind("visible").title("V")]).product("p").as_of("2026-10-14")).unwrap_err();
    let cap = || capability([]).id("c").title("t").need("must").have("none");
    for (said, why) in [
        (one(cap().need("nice")), "need is one of must, should, later"),
        (one(cap().have("some")), "have is one of built, partial, none"),
        (one(cap().effort("XXL")), "effort is one of S, M, L, XL"),
        (one(cap().run("ESP or own MTAs")), "run is a number of 0 or more"),
        (declare(replacement([]).product("p").as_of("October")).unwrap_err(), "asOf is a day"),
        (declare(replacement([layer([]).kind("hidden").title("H")]).product("p").as_of("2026-10-14")).unwrap_err(), "kind is one of visible, invisible"),
        (
            declare(replacement([decision().choice("maybe").because("x")]).product("p").as_of("2026-10-14")).unwrap_err(),
            "choice is one of build, buy, integrate, skip",
        ),
    ] {
        assert!(said.contains(why), "{why}: {said}");
    }
}
