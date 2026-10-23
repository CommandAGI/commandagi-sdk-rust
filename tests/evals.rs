// An eval and a metric in Rust against the documents the TypeScript SDK's evals.test.ts pins for the same example:
// each is its record; a case names its task file or says its instruction (one of the two); the objective's metrics and
// the scoring's weights are refs; `frozen` is a digest; what does not read is refused with why.

use commandagi::design::evals::*;
use commandagi::design::{declare, Declared};

fn document(root: El) -> (String, Json, Json) {
    match declare(root).unwrap() {
        Declared::Document { format, document, sources } => (format, document, sources),
        Declared::Graph(_) => panic!("an eval and a metric are documents"),
    }
}

fn digest() -> String {
    format!("sha256:{}", "ab".repeat(32))
}

#[test]
fn an_eval_holds_one_case_per_case_each_site_by_its_slug() {
    let first = case()
        .slug("undock-and-lap")
        .name("Undock and lap")
        .task("../Tasks/Undock and lap.task.tsx")
        .environment(json(r#"{"kind": "synthetic", "seed": 41}"#));
    let first_site = first.source_json().unwrap();
    let (format, doc, sources) = document(
        eval([
            first,
            case()
                .slug("open-a-terminal")
                .name("Open a terminal")
                .instruction("Open a terminal window.")
                .acceptance_metric("Terminal open.metric.tsx")
                .hints(["The dock has a terminal icon."])
                .time_limit_ms(120000)
                .environment(json(r#"{"kind": "none"}"#))
                .tags(["smoke"]),
        ])
        .slug("yard-shakedown")
        .domain("robot")
        .version("3")
        .metrics(["Fleet availability.metric.tsx", "Lap time.metric.tsx"])
        .scoring(json(r#"{"kind": "weighted", "weights": {"Fleet availability.metric.tsx": 2, "Lap time.metric.tsx": 1}}"#))
        .objective(json(
            r#"{"kind": "linear", "terms": [{"metric": "Fleet availability.metric.tsx", "weight": 2}, {"metric": "Lap time.metric.tsx", "weight": -0.01}], "bias": 0}"#,
        ))
        .repeats(3)
        .frozen(digest()),
    );
    assert_eq!(format, "eval");
    let expected = format!(
        r#"{{"domain": "robot", "slug": "yard-shakedown", "version": "3", "metrics": ["Fleet availability.metric.tsx", "Lap time.metric.tsx"],
            "scoring": {{"kind": "weighted", "weights": {{"Fleet availability.metric.tsx": 2, "Lap time.metric.tsx": 1}}}},
            "objective": {{"kind": "linear", "terms": [{{"metric": "Fleet availability.metric.tsx", "weight": 2}}, {{"metric": "Lap time.metric.tsx", "weight": -0.01}}], "bias": 0}},
            "repeats": 3,
            "frozen": "{}", "cases": [
            {{"slug": "undock-and-lap", "name": "Undock and lap", "task": "../Tasks/Undock and lap.task.tsx", "environment": {{"kind": "synthetic", "seed": 41}}}},
            {{"slug": "open-a-terminal", "name": "Open a terminal", "instruction": "Open a terminal window.", "acceptanceMetric": "Terminal open.metric.tsx",
              "hints": ["The dock has a terminal icon."], "timeLimitMs": 120000, "environment": {{"kind": "none"}}, "tags": ["smoke"]}}]}}"#,
        digest()
    );
    assert_eq!(doc, Json::parse(&expected).unwrap());
    assert_eq!(sources.get("case#undock-and-lap"), Some(&first_site));
    assert!(sources.get("").is_some());
}

#[test]
fn a_metric_is_one_element() {
    let (format, doc, _) = document(metric().slug("fleet-availability").probe(json(r#"{"kind": "event", "eventKind": "telemetry"}"#)).aggregation("rate").target(0.96));
    assert_eq!(format, "metric");
    assert_eq!(doc, Json::parse(r#"{"slug": "fleet-availability", "probe": {"kind": "event", "eventKind": "telemetry"}, "aggregation": "rate", "target": 0.96}"#).unwrap());
}

#[test]
fn what_does_not_read_is_refused_with_why() {
    let kase = |c: El| declare(eval([c.slug("c").name("C")]).slug("e")).unwrap_err();
    let world = || json(r#"{"kind": "none"}"#);
    for (said, why) in [
        (kase(case().environment(world())), "says what the agent is handed once: a task file (task) or an instruction, not neither"),
        (kase(case().environment(world()).task("a.task.tsx").instruction("Do it.")), "not both"),
        (kase(case().environment(world()).task(json(r#"{"id": "t", "instruction": "Do it."}"#))), "task is text"),
        (kase(case().instruction("Do it.").environment(json(r#"{"seed": 1}"#))), "environment is a world"),
        (kase(case().environment(world()).instruction("Do it.").time_limit_ms(1.5)), "time_limit_ms is a whole number"),
        (declare(eval([]).slug("e").frozen(true)).unwrap_err(), "frozen is the digest the eval is frozen at"),
        (declare(eval([]).slug("e").with_attr("metricIds", ["m1"])).unwrap_err(), "metric_ids is not read"),
        (declare(eval([]).slug("e").metrics("m.metric.tsx")).unwrap_err(), "metrics is a list of texts"),
        (
            declare(eval([]).slug("e").objective(json(r#"{"kind": "metric", "metricId": "m1"}"#))).unwrap_err(),
            "objective names its metric file by ref (metric), not by node id (metricId)",
        ),
        (
            declare(eval([]).slug("e").objective(json(r#"{"kind": "linear", "terms": [{"metric": "a.metric.tsx"}, {"metricId": "m1", "weight": 1}]}"#))).unwrap_err(),
            "objective.terms[1] names its metric file by ref",
        ),
        (declare(eval([]).slug("e").objective(json(r#"{"kind": "linear", "terms": []}"#))).unwrap_err(), "objective.terms is a list"),
        (declare(eval([]).slug("e").objective(json(r#"{"kind": "metric"}"#))).unwrap_err(), "objective.metric is a metric file's ref"),
        (
            declare(eval([]).slug("e").scoring(json(r#"{"kind": "weighted", "weights": {"a.metric.tsx": "2"}}"#))).unwrap_err(),
            "scoring.weights is",
        ),
        (declare(metric().slug("m").probe(json("{}")).aggregation("rate")).unwrap_err(), "probe is what to measure"),
        (declare(metric().slug("m").aggregation("rate")).unwrap_err(), "metric() needs probe"),
    ] {
        assert!(said.contains(why), "{why}: {said}");
    }
}
