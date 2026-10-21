// A process flow in Rust against the document the TypeScript SDK's flows.test.ts pins for the same elements: its tools
// and its steps, each step with its params and measures; a wrong field refused by name.

use commandagi::design::flows::*;
use commandagi::design::{declare, Declared};

fn document(root: El) -> (String, Json) {
    match declare(root).unwrap() {
        Declared::Document { format, document, .. } => (format, document),
        Declared::Graph(_) => panic!("a flow is a document"),
    }
}

#[test]
fn a_flow_is_its_tools_and_steps() {
    let (format, doc) = document(
        flow([
            tool().id("coater").ref_("Kit.3d.tsx#coater").rate_per_hour(40).currency("USD"),
            step([]).id("why").kind("note").instruction("Marks first."),
            step([
                param().name("speed").value(3000).unit("rpm"),
                measure().id("resist").name("resist thickness").nominal(1.2).plus(0.1).minus(0.1).unit("um"),
            ])
            .id("coat")
            .order(1)
            .tool("coater")
            .by("in-house")
            .minutes(6),
        ])
        .name("Marks")
        .of("Kit.3d.tsx#wafer"),
    );
    assert_eq!(format, "flow");
    let want = Json::parse(
        r#"{"name":"Marks","of":"Kit.3d.tsx#wafer","tools":[{"id":"coater","ref":"Kit.3d.tsx#coater","ratePerHour":40,"currency":"USD"}],
        "steps":[{"id":"why","kind":"note","instruction":"Marks first."},
        {"id":"coat","order":1,"tool":"coater","by":"in-house","minutes":6,"params":[{"name":"speed","value":3000,"unit":"rpm"}],
        "measures":[{"id":"resist","name":"resist thickness","nominal":1.2,"plus":0.1,"minus":0.1,"unit":"um"}]}]}"#,
    )
    .unwrap();
    assert_eq!(doc, want);
}

#[test]
fn a_wrong_field_of_a_flow_is_refused_by_name() {
    let bad = |root: El| declare(root).unwrap_err();
    let cases: Vec<(El, &str)> = vec![
        (flow([step([]).id("a").kind("rest")]), r#"step().id("a"): kind must be one of work, note (got "rest")"#),
        (flow([step([]).id("a").minutes(-1)]), r#"step().id("a"): minutes is a number of at least 0"#),
        (flow([step([]).id("a").order(0)]), r#"step().id("a"): order is a whole number from 1"#),
        (flow([step([]).id("a").kind("note").minutes(5)]), r#"step().id("a"): a note step names no tool, takes no time"#),
        (flow([step([]).id("a").currency("usd")]), r#"step().id("a"): currency is an ISO 4217 code"#),
        (flow([tool().id("t").rate_per_hour(-2)]), r#"tool().id("t"): ratePerHour is a number of at least 0"#),
        (flow([step([measure().id("m").plus(0.1)]).id("a")]), r#"measure().id("m"): plus and minus are tolerances of the nominal: give nominal"#),
        (flow([step([measure().id("m").min(2).max(1)]).id("a")]), r#"measure().id("m"): min is at most max"#),
        (flow([step([param().name("t").value(1), param().name("t").value(2)]).id("a")]), r#"step().id("a"): two params are named "t""#),
        (flow([step([measure().id("m").max(1)]).id("a"), step([measure().id("m").max(2)]).id("b")]), r#"measure().id("m"): two measures have this id"#),
    ];
    for (root, why) in cases {
        let got = bad(root);
        assert!(got.contains(why), "{got} names {why}");
    }
}
