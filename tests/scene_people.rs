// A person in a scene against scene_people.json, the golden of the three SDKs (scene-people.golden.json in the
// TypeScript SDK): each case the engine accepts passes, and each refusal is the engine's own words.

use commandagi::design::scene_people::*;
use commandagi::design::Json;

#[test]
fn the_engines_refusals() {
    let g = Json::parse(include_str!("scene_people.json")).unwrap();
    let Some(Json::Arr(cases)) = g.get("cases") else { panic!() };
    for c in cases {
        let value = c.get("value").unwrap();
        let said = match c.get("check").and_then(Json::as_str).unwrap() {
            "look" => check_look(value, "robot"),
            "goal" => check_goal(value),
            "humanoid" => check_humanoid_policy(value),
            "motors" => check_motors(value),
            "assist" => check_assist(value),
            "face" => check_face(value),
            other => panic!("no check {other}"),
        };
        match c.get("says").and_then(Json::as_str) {
            None => assert!(said.is_ok(), "{}: {said:?}", value.text()),
            Some(want) => {
                let got = said.expect_err(want);
                assert!(got.contains(want), "{want}: {got}");
            }
        }
    }
}

#[test]
fn the_joints() {
    let mut joints = HUMANOID_JOINTS.to_vec();
    joints.sort();
    joints.dedup();
    assert_eq!(joints.len(), 33);
    assert_eq!(humanoid_constraint("ana", "shoulder_l_abduct"), "ana-shoulder-l-abduct");
}

#[test]
fn the_face_units_and_ethnicities() {
    let mut units = FACE_UNITS.to_vec();
    units.sort();
    units.dedup();
    assert_eq!(units.len(), 52);
    assert_eq!(ETHNICITIES, ["african", "asian", "caucasian"]);
}
