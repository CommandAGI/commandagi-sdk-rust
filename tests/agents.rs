// A harness and an agent in Rust against the documents the TypeScript SDK's agents.test.ts pins for the same elements:
// a base names its harness, a fork names its parent, never both; a program harness needs no model; a realtime harness
// says what its model takes and gives live; what the vocabulary has no words for is refused by name.

use commandagi::design::agents::*;
use commandagi::design::{declare, Declared};

fn document(root: El) -> (String, Json, Json) {
    match declare(root).unwrap() {
        Declared::Document { format, document, sources } => (format, document, sources),
        Declared::Graph(_) => panic!("a harness and an agent are documents"),
    }
}

fn bad(root: El) -> String {
    declare(root).unwrap_err()
}

fn j(text: &str) -> Json {
    Json::parse(text).unwrap()
}

const REALTIME: &str = r#"{"in": ["audio", "video", "text"], "out": ["audio", "text", "tools"],
    "audio": {"in": ["g711-ulaw@8000", "pcm16@24000"], "out": ["g711-ulaw@8000"]}, "fps": 1, "hz": 2, "minutes": 60}"#;

fn api(realtime: &str) -> El {
    harness().kind("api").label("X").provider("p").realtime(json(realtime))
}

#[test]
fn a_harness_of_each_kind() {
    let root = harness().kind("cli").label("Claude Code").provider("anthropic").cli("claude").runs(["host", "cloud"]);
    let site = root.source_json().unwrap();
    let (format, doc, sources) = document(root);
    assert_eq!(format, "harness");
    assert_eq!(doc, j(r#"{"kind": "cli", "label": "Claude Code", "provider": "anthropic", "cli": "claude", "runs": ["host", "cloud"]}"#));
    assert_eq!(sources.get(""), Some(&site));
    let (_, program, _) = document(harness().kind("program").label("Symbolic Planner 001").family("symbolic").experimental(true).runs(["host"]));
    assert_eq!(program, j(r#"{"kind": "program", "label": "Symbolic Planner 001", "runs": ["host"], "family": "symbolic", "experimental": true}"#));
    let (_, single, _) = document(harness().kind("api").label("ScaleDown Classify-1").provider("scaledown").tasks(["classify", "decide"]).runs(["host"]));
    assert_eq!(single.get("tasks"), Some(&j(r#"["classify", "decide"]"#)));
}

#[test]
fn a_base_names_its_harness_and_a_fork_its_parent() {
    let (format, base, _) = document(agent().harness("anthropic").model("anthropic/claude-opus-5-5").effort("medium").name("Claude Opus 5.5"));
    assert_eq!(format, "agent");
    assert_eq!(base, j(r#"{"harness": "anthropic", "name": "Claude Opus 5.5", "model": "anthropic/claude-opus-5-5", "effort": "medium"}"#));
    let (_, led, _) = document(agent().parent("cloud://global/agents/claude-code").supervisor("agent:lead"));
    assert_eq!(led, j(r#"{"parent": "cloud://global/agents/claude-code", "supervisor": "agent:lead"}"#));
    assert!(bad(agent().parent("cloud://global/agents/claude-code").supervisor("lead")).contains("supervisor is an agent or a person"));
    let (_, fork, _) = document(
        agent()
            .parent("cloud://global/agents/claude-code")
            .name("Joe's secretary")
            .instructions("Keep Joe's calendar.")
            .voice_mode("push-to-talk")
            .tools(["calendar_save_event"]),
    );
    assert_eq!(
        fork,
        j(r#"{"parent": "cloud://global/agents/claude-code", "name": "Joe's secretary", "instructions": "Keep Joe's calendar.",
              "voiceMode": "push-to-talk", "tools": ["calendar_save_event"]}"#)
    );
}

#[test]
fn a_realtime_harness_says_what_its_model_takes_and_gives_live() {
    let (_, h, _) = document(harness().kind("api").label("OpenAI Realtime API").provider("openai").runs(["worker", "host"]).realtime(json(REALTIME)));
    assert_eq!(h.get("realtime"), Some(&j(REALTIME)));
    let (_, live, _) = document(api(r#"{"in": ["audio"], "out": ["audio", "tools"], "audio": {"in": ["pcm16@16000"], "out": ["pcm16@24000"]}, "hz": 1, "minutes": 15, "videoMinutes": 2}"#));
    assert_eq!(live.get("realtime").and_then(|r| r.get("videoMinutes")), Some(&Json::Num(2.0)));
    assert!(bad(api(&REALTIME.replace(r#""fps": 1, "#, ""))).contains("fps is the most video frames"));
    assert!(bad(api(&REALTIME.replace(r#""hz": 2"#, r#""hz": 0"#))).contains("hz is the most tool calls"));
    assert!(bad(api(r#"{"in": ["audio"], "out": ["text"]}"#)).contains("audio names the formats"));
    assert!(bad(api(r#"{"in": ["smell"], "out": ["text"]}"#)).contains("in is a list of"));
    assert!(bad(api(&REALTIME.replace(r#""minutes": 60"#, r#""minutes": 60, "latency": 1"#))).contains("has no field latency"));
    assert!(bad(api(r#"{"in": ["text"], "out": ["text"], "audio": {"in": ["pcm16@24000"]}}"#)).contains("audio is for a harness that takes or gives audio"));
    assert!(bad(api(REALTIME).tasks(["act"])).contains("realtime is for kind(\"api\") with no tasks"));
}

#[test]
fn refusals_by_name() {
    assert!(bad(agent().name("x")).contains("names its harness"));
    assert!(bad(agent().harness("codex").parent("../a")).contains("not both: a fork keeps its parent's harness"));
    assert!(bad(agent().harness("Codex CLI")).contains("harness is a harness's id"));
    assert!(bad(agent().parent("../a").voice_mode("loud")).contains("voice_mode is"));
    assert!(bad(agent().parent("../a").tools("all")).contains("tools is a list"));
    assert!(bad(agent().parent("../a").persona("warm")).contains("persona is an object"));
    assert!(bad(harness().kind("cli").label("X")).contains("names the program it starts"));
    assert!(bad(harness().kind("api").label("X")).contains("names its provider"));
    assert!(bad(harness().kind("program").label("X").cli("x")).contains("cli is for kind(\"cli\") only"));
    assert!(bad(harness().kind("api").label("X").provider("p").runs(["laptop"])).contains("runs is a list of"));
    assert!(bad(harness().kind("robot").label("X")).contains("kind is"));
    assert!(bad(harness().kind("program").label("X").experimental("yes")).contains("experimental is true or false"));
    assert!(bad(harness().kind("api").label("X").provider("p").tasks(["chat"])).contains("tasks is a list of"));
    assert!(bad(harness().kind("program").label("X").tasks(["classify"])).contains("tasks is for kind(\"api\") only"));
    assert!(bad(harness().label("X")).contains("harness() needs kind"));
    assert!(bad(agent().parent("../a").with_attr("colour", "red")).contains("colour is not read"));
}
