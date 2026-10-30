// A scout's comparison and a replacement-cost analysis in Rust against the documents the TypeScript SDK's
// scouts.test.ts pins for the same examples: each is its record, sites included; what would make a run dishonest is
// refused with why (a customer's case, a case with two scorers, two incumbents, a check with no kind).

use commandagi::design::scouts::*;
use commandagi::design::{declare, Declared};

fn document(root: El) -> (String, Json, Json) {
    match declare(root).unwrap() {
        Declared::Document { format, document, sources } => (format, document, sources),
        Declared::Graph(_) => panic!("a comparison and a replacement-cost analysis are documents"),
    }
}

/// The cases of a comparison's first workload.
fn first_workload_cases(doc: &Json) -> Vec<Json> {
    match doc.get("workloads") {
        Some(Json::Arr(workloads)) => match workloads[0].get("cases") {
            Some(Json::Arr(cases)) => cases.clone(),
            _ => panic!("the workload's cases are a list"),
        },
        _ => panic!("a comparison's workloads are a list"),
    }
}

fn comparison(first_case: El) -> El {
    scout([
        configuration().id("sonnet").model("anthropic/claude-sonnet-5-5").effort("low").incumbent(true),
        configuration().id("glm").model("dashscope/glm-5.3"),
        workload([
            first_case,
            case().id("page").from_("own-run").source("runs/thread-1/streams/chat@a/records.jsonl#12").prompt("Write the provider page.").rubric("Names the price and the terms."),
        ])
        .id("knowledge").title("Knowledge work").needs(["structured"]).sample(20),
        judge().model("openai/gpt-5.6-terra"),
    ])
    .budget(10).repeats(3).title("GLM against Sonnet")
}

#[test]
fn a_comparison_declares_its_configurations_workloads_and_judges() {
    let (format, doc, sources) = document(comparison(
        case().id("rate").from_("synthetic").prompt("What is 4.2% as a decimal?").check(json(r#"{"number": 0.042, "tolerance": 0.0005}"#)),
    ));
    assert_eq!(format, "scout");
    let expected = Json::parse(
        r#"{"title": "GLM against Sonnet", "budget": 10, "repeats": 3,
            "configurations": [
              {"id": "sonnet", "model": "anthropic/claude-sonnet-5-5", "effort": "low", "incumbent": true},
              {"id": "glm", "model": "dashscope/glm-5.3"}],
            "workloads": [
              {"id": "knowledge", "title": "Knowledge work", "needs": ["structured"], "sample": 20, "cases": [
                {"id": "rate", "from": "synthetic", "prompt": "What is 4.2% as a decimal?", "check": {"number": 0.042, "tolerance": 0.0005}},
                {"id": "page", "from": "own-run", "prompt": "Write the provider page.", "rubric": "Names the price and the terms.",
                 "source": "runs/thread-1/streams/chat@a/records.jsonl#12"}]}],
            "judges": [{"model": "openai/gpt-5.6-terra"}]}"#,
    )
    .unwrap();
    assert_eq!(doc, expected);
    for path in ["", "configuration#sonnet", "configuration#glm", "workload#knowledge", "workload#knowledge/case#rate", "workload#knowledge/case#page", "judge#openai/gpt-5.6-terra"] {
        assert!(sources.get(path).is_some(), "a site for {path}");
    }
}

#[test]
fn a_comparison_refuses_what_would_make_its_verdict_dishonest() {
    let refuse = |root: El| declare(root).unwrap_err();
    let two_incumbents = scout([configuration().id("a").model("x/a").incumbent(true), configuration().id("b").model("x/b").incumbent(true)]).budget(5);
    let one = |c: El| refuse(scout([workload([c.id("c").from_("synthetic").prompt("p")]).id("w")]).budget(1));
    for (said, why) in [
        (refuse(comparison(case().id("rate").from_("customer").prompt("p").check(json(r#"{"exact": "1"}"#)))), "from is one of public, synthetic, own-run"),
        (one(case().rubric("and a judge").check(json(r#"{"exact": "1"}"#))), "is scored by exactly one of check (a program) or rubric"),
        (one(case().check(json(r#"{"tolerance": 1}"#))), "check is exactly one of"),
        (one(case().check(json(r#"{"exact": "a", "contains": ["b"]}"#))), "it has exact and contains"),
        (one(case().check(json(r#"{"json": [1]}"#))), "json is the object the answer must hold"),
        (one(case().check(json(r#"{"letter": "ab"}"#))), "letter is one capital letter"),
        (one(case().check(json(r#"{"tool": "send_email"}"#))), "tool needs the case's tools"),
        (one(case().turns(3).check(json(r#"{"exact": "x"}"#))), "turns goes with tools"),
        (one(case().tools(json(r#"[{"name": "a b", "parameters": {}}]"#)).check(json(r#"{"exact": "x"}"#))), "a tool's name is a word"),
        (one(case().tools(json(r#"[{"name": "a", "parameters": {}, "results": [{"when": {"x": 1}, "result": "r"}]}]"#)).check(json(r#"{"exact": "x"}"#))), "a result is"),
        (one(case().check(json(r#"{"program": {"data": []}}"#))), "program is { data: [[x, y], …] }"),
        (one(case().check(json(r#"{"program": {"data": [[1, 4]], "tolerance": -1}}"#))), "program tolerance is a number of 0 or more"),
        (refuse(two_incumbents), "one incumbent"),
        (refuse(scout([]).budget(0)), "budget is a number above 0"),
        (refuse(scout([]).title("no budget")), "needs budget"),
        (refuse(scout([case().id("x").from_("public").prompt("p").check(json(r#"{"exact": "1"}"#))]).budget(5)), "stands in workload(), not in scout()"),
    ] {
        assert!(said.contains(why), "{why}: {said}");
    }
}

#[test]
fn a_tool_case_offers_its_tools_and_their_answers() {
    let tools = Json::parse(
        r#"[{"name": "get_capital", "parameters": {"type": "object", "properties": {"country": {"type": "string"}}},
             "results": [{"when": {"country": "france"}, "result": "Paris"}]},
            {"name": "send_email", "description": "send an email", "parameters": {"type": "object"}}]"#,
    )
    .unwrap();
    let (_, doc, _) = document(scout([workload([
        case().id("chain").from_("synthetic").prompt("p").tools(tools.clone()).turns(4).check(json(r#"{"number": 4280000}"#)),
        case().id("first").from_("synthetic").prompt("p").tools(tools.clone()).check(json(r#"{"tool": "send_email"}"#)),
        case().id("mc").from_("public").source("TIGER-Lab/MMLU-Pro test 12").prompt("p").check(json(r#"{"letter": "I"}"#)),
        case().id("crux").from_("public").prompt("p").check(json(r#"{"literal": "[1, 'a']"}"#)),
    ]).id("agentic")]).budget(1));
    let cases = first_workload_cases(&doc);
    assert_eq!(cases[0].get("tools"), Some(&tools));
    assert_eq!(cases[0].get("turns"), Some(&Json::Num(4.0)));
    assert_eq!(cases[1].get("check"), Some(&Json::parse(r#"{"tool": "send_email"}"#).unwrap()));
    let (_, prog, _) = document(scout([workload([case().id("c").from_("synthetic").prompt("p").check(json(r#"{"program": {"data": [[1, 4], [2, 7], [5, 16]]}}"#))]).id("w")]).budget(1));
    assert_eq!(first_workload_cases(&prog)[0].get("check"), Some(&Json::parse(r#"{"program": {"data": [[1, 4], [2, 7], [5, 16]]}}"#).unwrap()));
}

#[test]
fn a_replacement_analysis_declares_its_layers_capabilities_and_decision() {
    let (format, doc, _) = document(
        replacement([
            layer([
                capability([
                    evidence().source("beehiiv checklist").claim("reported"),
                ])
                .id("bulk-send").title("3M+ emails a minute").need("must").have("none").effort("XL").run(2000).labour(160).note("ESP or own MTAs"),
                capability([]).id("abuse").title("signup validation").need("must").have("partial").effort("L").ref_("docs/moderation.md"),
            ])
            .kind("invisible").title("Deliverability"),
            layer([capability([]).id("collab").title("real-time collaboration").need("should").have("partial").effort("M")])
                .kind("visible").title("Editor"),
            layer([]).kind("visible").title("Website"),
            decision().choice("skip").because("The invisible layer is the product."),
            differentiation().text("Agents author the newsletter as code."),
            economics().price("$49 a month").margin("unknown"),
        ])
        .product("beehiiv").vendor("beehiiv, Inc.").url("https://www.beehiiv.com").as_of("2026-10-14").by_("scout"),
    );
    assert_eq!(format, "replacement");
    let expected = Json::parse(
        r#"{"product": "beehiiv", "vendor": "beehiiv, Inc.", "url": "https://www.beehiiv.com", "asOf": "2026-10-14", "by": "scout",
            "layers": [
              {"kind": "invisible", "title": "Deliverability", "capabilities": [
                {"id": "bulk-send", "title": "3M+ emails a minute", "need": "must", "have": "none", "effort": "XL", "run": 2000, "labour": 160,
                 "note": "ESP or own MTAs", "evidence": [{"source": "beehiiv checklist", "claim": "reported"}]},
                {"id": "abuse", "title": "signup validation", "need": "must", "have": "partial", "effort": "L", "ref": "docs/moderation.md",
                 "evidence": []}]},
              {"kind": "visible", "title": "Editor", "capabilities": [
                {"id": "collab", "title": "real-time collaboration", "need": "should", "have": "partial", "effort": "M", "evidence": []}]},
              {"kind": "visible", "title": "Website", "capabilities": []}],
            "decision": {"choice": "skip", "because": "The invisible layer is the product."},
            "differentiation": "Agents author the newsletter as code.",
            "economics": {"price": "$49 a month", "margin": "unknown"}}"#,
    )
    .unwrap();
    assert_eq!(doc, expected);
}

#[test]
fn a_replacement_analysis_refuses_a_value_its_sums_cannot_read() {
    let one = |c: El| {
        declare(replacement([layer([c.id("c").title("t")]).kind("visible").title("V")]).product("p").as_of("2026-10-14")).unwrap_err()
    };
    for (said, why) in [
        (one(capability([]).need("nice").have("none")), "need is one of must, should, later"),
        (one(capability([]).need("must").have("some")), "have is one of built, partial, none"),
        (one(capability([]).need("must").have("none").effort("XXL")), "effort is one of S, M, L, XL"),
        (one(capability([]).need("must").have("none").run("ESP or own MTAs")), "run is a number of 0 or more"),
        (declare(replacement([]).product("p").as_of("October")).unwrap_err(), "asOf is a day"),
        (declare(replacement([layer([]).kind("hidden").title("H")]).product("p").as_of("2026-10-14")).unwrap_err(), "kind is one of visible, invisible"),
        (declare(replacement([decision().choice("maybe").because("x")]).product("p").as_of("2026-10-14")).unwrap_err(), "choice is one of build, buy, integrate, skip"),
    ] {
        assert!(said.contains(why), "{why}: {said}");
    }
}
