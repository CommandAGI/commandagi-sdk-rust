// A symbolic agent's skill in Rust against the document the TypeScript SDK's skills.ts gives for the same elements:
// its phrases, steps and reply; a literal script's result; a skill that does not read is refused with why.

use commandagi::design::skills::*;
use commandagi::design::{declare, Declared};

fn document(root: El) -> (String, Json) {
    match declare(root).unwrap() {
        Declared::Document { format, document, .. } => (format, document),
        Declared::Graph(_) => panic!("a skill is a document"),
    }
}

#[test]
fn the_example_is_its_document() {
    let (format, doc) = document(
        skill([
            phrase().match_("triple {x:number}"),
            step().expr("mul(x, 3)").as_("out"),
            step().tool("read_file").args(json(r#"{"path": "{file}"}"#)).as_("read").always(true),
            reply().text("{out}"),
            rule().if_(["unit(?u, ?d)"]).then("known(?u)"),
            fact().is_("ready(cam)"),
            example().say("triple 2").reply_(""),
        ])
        .about("Three times a number."),
    );
    assert_eq!(format, "skill");
    assert_eq!(
        doc,
        Json::parse(
            r#"{"about": "Three times a number.", "phrases": [{"match": "triple {x:number}"}],
                "steps": [{"expr": "mul(x, 3)", "as": "out"}, {"tool": "read_file", "args": {"path": "{file}"}, "as": "read", "always": true}],
                "reply": "{out}", "rules": [{"if": ["unit(?u, ?d)"], "then": "known(?u)"}], "facts": ["ready(cam)"],
                "examples": [{"say": "triple 2", "reply": ""}]}"#
        )
        .unwrap()
    );
    assert_eq!(document(result().value(2)), ("result".to_string(), Json::parse(r#"{"value": 2}"#).unwrap()));
}

#[test]
fn a_skill_that_does_not_read_is_refused_with_why() {
    for (s, why) in [
        (skill([step().expr("1").tool("read_file")]), "is exactly one of expr, tool, script, skill, see (it has expr and tool)"),
        (skill([step().as_("x")]), "is exactly one of"),
        (skill([step().expr("1").always("yes")]), "always is true or false"),
        (skill([step().tool("read_file").args(["a"])]), "args is an object"),
        (skill([rule().if_(json("[]")).then("x")]), "if is a fact or a list of facts"),
        (skill([]).needs("ready"), "needs is a list of facts"),
    ] {
        let said = declare(s).unwrap_err();
        assert!(said.contains(why), "{why}: {said}");
    }
}
