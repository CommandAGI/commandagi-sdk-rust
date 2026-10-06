// Tasks and projects in Rust against the documents the TypeScript SDK's tasks.test.ts pins for the same elements:
// fields are attributes, subtasks and views children in order, times read from a date or a UTC time as JavaScript's
// Date.parse reads them; what the vocabulary has no words for is refused by name.

use commandagi::design::tasks::*;
use commandagi::design::{declare, Declared};

fn document(root: El) -> (String, Json, Json) {
    match declare(root).unwrap() {
        Declared::Document { format, document, sources } => (format, document, sources),
        Declared::Graph(_) => panic!("a task is a document"),
    }
}

fn bad(root: El) -> String {
    declare(root).unwrap_err()
}

fn j(text: &str) -> Json {
    Json::parse(text).unwrap()
}

#[test]
fn a_task_is_its_body_subtasks_in_order_each_site_by_its_path() {
    let first = subtask().ref_("Ship it/Draft.task.tsx");
    let first_site = first.source_json().unwrap();
    let root = task([first, subtask().ref_("Ship it/Send.task.tsx")])
        .id("ship")
        .title("Ship it")
        .status("doing")
        .priority("high")
        .assignees(["agent:writer"])
        .due_at("2026-10-20")
        .readme("Ship it.md");
    let root_site = root.source_json().unwrap();
    let (format, doc, sources) = document(root);
    assert_eq!(format, "task");
    assert_eq!(
        doc,
        j(r#"{"id": "ship", "title": "Ship it", "status": "doing", "priority": "high", "assignees": ["agent:writer"], "dueAt": 1792454400000,
              "readme": "Ship it.md", "subtasks": ["Ship it/Draft.task.tsx", "Ship it/Send.task.tsx"]}"#)
    );
    assert_eq!(sources.get(""), Some(&root_site));
    assert_eq!(sources.get("subtask#Ship it/Draft.task.tsx"), Some(&first_site));

    assert!(bad(task([]).title("no id")).contains("task() needs id"));
    assert!(bad(task([]).id("a").with_attr("subtasks", ["x.task.tsx"])).contains("subtasks is not read"));
    assert!(bad(task([task([]).id("b")]).id("a")).contains("task() stands in"));
    assert!(bad(task([subtask().ref_("x.task.tsx"), subtask().ref_("x.task.tsx")]).id("a")).contains(r#"two subtask() in task() have ref_ "x.task.tsx""#));
    assert!(bad(task([]).id("a").due_at("next week")).contains(r#"task().id("a") due_at is a date"#));
    assert!(bad(task([subtask().ref_("")]).id("a")).contains("ref_ is a path"));
}

#[test]
fn times_read_as_javascripts_date_parse_reads_them() {
    let t = |v: Json| time_of(&v, "t");
    // Each value beside what `Date.parse` (V8) gives for it.
    for (text, ms) in [
        ("2026-10-20", 1792454400000_i64),
        ("2026-10-20T14:30:00Z", 1792506600000),
        ("2026-10-20T14:30Z", 1792506600000),
        ("2026-10-20T14:30:00.123456Z", 1792506600123),
        ("2026-10-20T14:30:00.5+02:00", 1792499400500),
        ("2026-10-20T00:00:00+23:59", 1792368060000),
        ("2026-10-20T24:00:00Z", 1792540800000),
        ("2026-02-30T00:00:00Z", 1772409600000),
        ("0000-01-01T00:00:00Z", -62167219200000),
    ] {
        assert_eq!(t(Json::from(text)), Ok(Json::Num(ms as f64)), "{text}");
    }
    for text in ["2026-13-01", "2026-10-00", "2026-10-32", "2026-10-20T24:30:00Z", "2026-10-20T10:00:60Z", "2026-10-20T00:00:00+24:00", "2026-10-20T14:30:00", "20/10/2026"] {
        assert!(t(Json::from(text)).unwrap_err().contains("t is a date"), "{text}");
    }
    assert_eq!(t(Json::Num(1.0)), Ok(Json::Num(1.0)));
    assert!(t(Json::Num(1.5)).is_err());
}

#[test]
fn a_projects_views_are_view_children() {
    let root = project([
        view().id("board").name("Board").mode("board").group_by("status"),
        view().id("due").name("By date").mode("timeline").group_by("due").filter(json(r#"{"includeClosed": true}"#)),
    ])
    .id("launch")
    .name("Launch")
    .glyph("R")
    .archived(false);
    let (format, doc, _) = document(root);
    assert_eq!(format, "project");
    assert_eq!(
        doc,
        j(r#"{"id": "launch", "name": "Launch", "glyph": "R", "archived": false, "views": [
              {"id": "board", "name": "Board", "mode": "board", "groupBy": "status"},
              {"id": "due", "name": "By date", "mode": "timeline", "groupBy": "due", "filter": {"includeClosed": true}}]}"#)
    );
    assert!(bad(project([view().id("v").with_attr("colour", "red")]).id("p")).contains(r#"view().id("v"): colour is not read"#));
}
