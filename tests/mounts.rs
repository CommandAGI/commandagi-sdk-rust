// A thread's mounts in Rust against the document the TypeScript SDK's mounts.test.ts pins for the same elements: each
// mount is its record, one task or a list; a mount that does not read is refused with why.

use commandagi::design::mounts::*;
use commandagi::design::{declare, Declared};

fn document(root: El) -> (String, Json, Json) {
    match declare(root).unwrap() {
        Declared::Document { format, document, sources } => (format, document, sources),
        Declared::Graph(_) => panic!("mounts are a document"),
    }
}

fn bad(m: El) -> String {
    declare(mounts([m])).unwrap_err()
}

#[test]
fn the_example_is_its_document_each_site_by_its_label() {
    let first = mount().label("launch").folder("../../Tasks/launch");
    let first_site = first.source_json().unwrap();
    let (format, doc, sources) = document(mounts([
        first,
        mount().label("ship it").task("../../Tasks/Ship it.task.tsx"),
        mount().label("owner inbox").assignee("user:u_1").status(["todo", "doing", "blocked"]),
    ]));
    assert_eq!(format, "mounts");
    assert_eq!(
        doc,
        Json::parse(
            r#"{"mounts": [
                {"label": "launch", "folder": "../../Tasks/launch"},
                {"label": "ship it", "task": "../../Tasks/Ship it.task.tsx"},
                {"label": "owner inbox", "assignee": "user:u_1", "status": ["todo", "doing", "blocked"]}]}"#
        )
        .unwrap()
    );
    assert_eq!(sources.get("mount#launch"), Some(&first_site));
    assert_eq!(document(mounts([])).1, Json::parse(r#"{"mounts": []}"#).unwrap());
}

#[test]
fn a_mount_that_does_not_read_is_refused_with_why() {
    for (m, why) in [
        (mount().label("a").task("x.task.tsx").folder("Tasks"), "mounts one task, or a list"),
        (mount().label("a"), "mounts nothing"),
        (mount().label("a").labels(json("null")), "labels is a list of texts"),
        (mount().label("a").assignee("ada"), r#"mount().label("a") assignee is user:<id> or agent:<id>, not "ada""#),
        (mount().label("a").assignee("user:a b"), "assignee is user:<id> or agent:<id>"),
        (mount().label("a").status(["later"]), "status is a list of proposed, todo, doing"),
        (mount().label("a").labels([""]), "labels is a list of texts"),
        (mount().label("a").folder(""), "folder is a text"),
        (mount().label("x".repeat(201)).folder("Tasks"), "label is a short text"),
        (mount().folder("Tasks"), "needs label"),
        (mount().label("a").folder("Tasks").with_attr("colour", "red"), "colour is not read"),
    ] {
        let said = bad(m);
        assert!(said.contains(why), "{why}: {said}");
    }
    let twice = declare(mounts([mount().label("a").folder("x"), mount().label("a").folder("y")])).unwrap_err();
    assert!(twice.contains(r#"two mount() in mounts() have label "a""#), "{twice}");
}
