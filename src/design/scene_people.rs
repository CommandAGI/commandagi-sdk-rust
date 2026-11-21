//! A PERSON IN A SCENE — the human avatar (`cloud://global/devices/human-avatar/definition.tsx`) placed in a design's
//! `slot().name("scene")` as a `robots` entry with a `look`, the `humanoid` policy that drives it, and the `goal`,
//! `motors`, `assist` and `face` commands an agent sends it (docs/formats.md § a person in the scene, in the CommandAGI
//! repository). The same as the TypeScript SDK's (`scene-people.ts`) and the Python SDK's
//! (`commandagi.design.scene_people`):
//!
//! ```
//! use commandagi::design::scene_people::*;
//! use commandagi::design::Json;
//!
//! check_look(&Json::parse(r#"{ "body": "feminine", "height_m": 1.68, "outfit": "blouse_skirt" }"#).unwrap(), "robot ana").unwrap();
//! check_humanoid_policy(&Json::parse(r#"{ "id": "ana-day", "kind": "humanoid", "unit": "ana",
//!   "routine": [{ "do": "walk", "route": [[0, 0], [1, 2]] }, { "do": "sit", "seat": [1, 2.4, 0.45], "facing": 1.57 }] }"#).unwrap()).unwrap();
//! assert_eq!(humanoid_constraint("ana", "knee_l"), "ana-knee-l");
//! ```
//!
//! `check_look`, `check_goal`, `check_humanoid_policy`, `check_motors`, `check_assist` and `check_face` refuse what
//! the engine refuses, in its words. They check the shape only: whether a unit, a body or a constraint is in the scene is the
//! engine's to say.

use super::Json;

pub const LOOK_BODIES: &[&str] = &["masculine", "feminine"];
pub const HAIR_STYLES: &[&str] = &["none", "afro", "bob", "bob_sleek", "braid", "long", "ponytail", "short", "short_swept", "pixie", "crop"];
pub const OUTFITS: &[&str] = &[
    "tee_jeans", "crew_tee_jeans", "longsleeve_jeans", "shirt_jeans", "denim_shirt_jeans", "jacket_jeans", "suit_tie", "overalls",
    "fitted_tee_jeans", "tee_shorts", "blouse_skirt", "sport_top_leggings",
];
pub const FOOTWEAR: &[&str] = &["leather_shoes", "hiking_shoes", "boots", "black_shoes", "white_trainers", "blue_trainers"];
/// MakeHuman's ethnic macros. A look's `ethnicity` weighs them (each 0 or more, not all 0); absent, an even blend.
pub const ETHNICITIES: &[&str] = &["african", "asian", "caucasian"];
/// A look's numbers and their bounds (`weight`, `muscle`, `proportions`: MakeHuman's macros, 0.5 average or neutral;
/// `age_years`: MakeHuman's adult ages, absent 25),
/// and its colours (sRGB 0 to 1; `top`, `bottom`, `shoes` tint the outfit's and the footwear's own). One table, the
/// same in the three SDKs.
pub const LOOK_BOUNDS: &[(&str, f64, f64)] = &[("height_m", 1.2, 2.2), ("weight", 0.0, 1.0), ("muscle", 0.0, 1.0), ("proportions", 0.0, 1.0), ("age_years", 25.0, 90.0)];
pub const LOOK_COLOURS: &[&str] = &["skin", "hair", "top", "bottom", "shoes"];
pub const LOOK_FIELDS: &[&str] = &[
    "body", "height_m", "weight", "muscle", "proportions", "age_years", "ethnicity", "skin", "hair", "top", "bottom", "shoes", "hair_style", "outfit", "footwear",
];
/// Each goal's own fields, beside `do`, `plan` and `task`. One table, the same in the three SDKs. `take` … `ride` are a
/// worker's actions on a factory floor, `drive` … `stop` a vehicle's (CommandAGI's docs/formats.md § a factory floor).
pub const GOAL_FIELDS: &[(&str, &[&str])] = &[
    ("walk", &["route", "speed", "straight"]),
    ("step", &["to", "heading"]),
    ("turn", &["heading"]),
    ("sit", &["seat", "facing"]),
    ("stand", &[]),
    ("idle", &["activity", "seconds", "at", "with"]),
    ("reach", &["at", "hand", "seconds"]),
    ("take", &["body", "hands"]),
    ("put", &["at"]),
    ("push", &["body", "handle"]),
    ("jack", &["body", "handle"]),
    ("press", &["at"]),
    ("inspect", &["body", "seconds"]),
    ("hand", &["body", "to"]),
    ("ride", &["body"]),
    ("drive", &["route", "speed", "reverse"]),
    ("dock", &["at", "heading", "speed", "reverse"]),
    ("lift", &["height", "load"]),
    ("stop", &[]),
];
pub const BODY_GOALS: &[&str] = &["take", "push", "jack", "inspect", "hand", "ride"];
pub const VEHICLE_GOALS: &[&str] = &["drive", "dock", "lift", "stop"];
pub const WORK_HANDS: &[&str] = &["both", "left", "right"];
pub const IDLE_ACTIVITIES: &[&str] = &["type", "look", "still", "talk"];
pub const REACH_HANDS: &[&str] = &["left", "right", "both"];
pub const HUMANOID_POLICY_FIELDS: &[&str] = &["id", "kind", "unit", "hz", "lookahead", "routine", "repeat"];
/// The person's joints. Each is the constraint `humanoid_constraint(unit, joint)` that a `motors` command names.
pub const HUMANOID_JOINTS: &[&str] = &[
    "spine_lower", "spine_lower_lean", "spine_lower_twist", "spine_upper", "spine_upper_lean", "spine_upper_twist", "neck_turn", "neck_nod",
    "neck_lean", "shoulder_l", "shoulder_l_abduct", "shoulder_l_twist", "elbow_l", "wrist_l", "wrist_l_deviate", "hip_l", "hip_l_abduct",
    "hip_l_twist", "knee_l", "ankle_l", "ankle_l_roll", "shoulder_r", "shoulder_r_abduct", "shoulder_r_twist", "elbow_r", "wrist_r",
    "wrist_r_deviate", "hip_r", "hip_r_abduct", "hip_r_twist", "knee_r", "ankle_r", "ankle_r_roll",
];

/// `ana`, `knee_l` → `ana-knee-l`.
pub fn humanoid_constraint(unit: &str, joint: &str) -> String {
    format!("{unit}-{}", joint.replace('_', "-"))
}

fn num(x: Option<&Json>) -> Option<f64> {
    x.and_then(Json::as_f64).filter(|n| n.is_finite())
}
fn point(x: Option<&Json>, n: usize) -> bool {
    matches!(x, Some(Json::Arr(items)) if items.len() == n && items.iter().all(|c| num(Some(c)).is_some()))
}
fn text(x: Option<&Json>) -> bool {
    x.and_then(Json::as_str).is_some_and(|s| !s.is_empty())
}
fn one_of(list: &[&str], x: Option<&Json>) -> bool {
    x.and_then(Json::as_str).is_some_and(|s| list.contains(&s))
}
fn entries(x: &Json) -> Option<&Vec<(String, Json)>> {
    match x {
        Json::Obj(e) => Some(e),
        _ => None,
    }
}
fn unknown(g: &Json, fields: &[&str], what: &str) -> Option<String> {
    entries(g)?.iter().find(|(k, _)| !fields.contains(&k.as_str())).map(|(k, _)| format!("{what} has no field {k} (it has {})", fields.join(", ")))
}

/// Refuses what the engine would refuse of a look. `where_`: the entry, for the words.
pub fn check_look(look: &Json, where_: &str) -> Result<(), String> {
    let no = |why: String| Err(format!("{where_}: {why}"));
    if entries(look).is_none() {
        return no(format!("look is {{ {} }}", LOOK_FIELDS.iter().map(|k| format!("{k}?")).collect::<Vec<_>>().join(", ")));
    }
    if let Some(bad) = unknown(look, LOOK_FIELDS, "look") {
        return no(bad);
    }
    if look.get("body").is_some() && !one_of(LOOK_BODIES, look.get("body")) {
        return no(format!("look.body is {}", LOOK_BODIES.join(" or ")));
    }
    for (k, lo, hi) in LOOK_BOUNDS {
        if look.get(k).is_some() && !num(look.get(k)).is_some_and(|v| v >= *lo && v <= *hi) {
            return no(format!("look.{k} is a number from {lo} to {hi}"));
        }
    }
    for k in LOOK_COLOURS {
        let ok = point(look.get(k), 3) && matches!(look.get(k), Some(Json::Arr(c)) if c.iter().all(|x| x.as_f64().is_some_and(|v| (0.0..=1.0).contains(&v))));
        if look.get(k).is_some() && !ok {
            return no(format!("look.{k} is [r, g, b], sRGB from 0 to 1"));
        }
    }
    if let Some(e) = look.get("ethnicity") {
        let ok = entries(e).is_some_and(|w| {
            unknown(e, ETHNICITIES, "").is_none()
                && w.iter().all(|(_, v)| num(Some(v)).is_some_and(|v| v >= 0.0))
                && w.iter().any(|(_, v)| num(Some(v)).is_some_and(|v| v > 0.0))
        });
        if !ok {
            let keys = ETHNICITIES.iter().map(|k| format!("{k}?")).collect::<Vec<_>>().join(", ");
            return no(format!("look.ethnicity is {{ {keys} }}, each a number 0 or more, not all 0"));
        }
    }
    for (k, list) in [("hair_style", HAIR_STYLES), ("outfit", OUTFITS), ("footwear", FOOTWEAR)] {
        if look.get(k).is_some() && !one_of(list, look.get(k)) {
            return no(format!("look.{k} is one of {}", list.join(", ")));
        }
    }
    Ok(())
}

/// What is wrong with a goal (its fields besides `verb` and `unit`, and a routine's `wait`), or None.
pub fn goal_problem(g: &Json, besides: &[&str]) -> Option<String> {
    let Some(fields) = entries(g) else { return Some("a goal is an object".into()) };
    let kinds: Vec<&str> = GOAL_FIELDS.iter().map(|(k, _)| *k).collect();
    let Some((kind, own)) = g.get("do").and_then(Json::as_str).and_then(|d| GOAL_FIELDS.iter().find(|(k, _)| *k == d)) else {
        return Some(format!("do is one of {}", kinds.join(", ")));
    };
    for (k, _) in fields {
        if k != "do" && k != "plan" && k != "task" && !besides.contains(&k.as_str()) && !own.contains(&k.as_str()) {
            return Some(format!("{kind} has no field {k} (it has {})", [*own, &["plan", "task"]].concat().join(", ")));
        }
    }
    let has = |k: &str| g.get(k).is_some();
    if has("task") && !text(g.get("task")) {
        return Some("task is a text (the dispatcher's token)".into());
    }
    if VEHICLE_GOALS.contains(kind) {
        return vehicle_goal_problem(g, kind);
    }
    let route_ok = matches!(g.get("route"), Some(Json::Arr(r)) if (1..=64).contains(&r.len()) && r.iter().all(|p| point(Some(p), 2)));
    if *kind == "walk" && !route_ok {
        return Some("walk needs a route: 1 to 64 points [x, y] (metres)".into());
    }
    if has("speed") && !num(g.get("speed")).is_some_and(|v| v > 0.0 && v <= 1.6) {
        return Some("walk speed is metres a second, above 0 and at most 1.6".into());
    }
    if has("straight") && !matches!(g.get("straight"), Some(Json::Bool(_))) {
        return Some("walk straight is true or false".into());
    }
    if BODY_GOALS.contains(kind) && !text(g.get("body")) {
        return Some(format!("{kind} needs a body (what it takes, pushes, steers, inspects, hands over or rides on)"));
    }
    if has("hands") && !one_of(WORK_HANDS, g.get("hands")) {
        return Some("hands is both, left or right".into());
    }
    if *kind == "press" && !point(g.get("at"), 3) {
        return Some("press needs at [x, y, z] (metres): the button".into());
    }
    if *kind == "put" && has("at") && !point(g.get("at"), 3) {
        return Some("at is [x, y, z] (metres)".into());
    }
    if has("handle") && !point(g.get("handle"), 3) {
        return Some("handle is [x, y, z] (metres): where the hands close on it".into());
    }
    if *kind == "hand" && !text(g.get("to")) {
        return Some("hand needs to (the unit it hands the body to)".into());
    }
    if *kind == "step" && !point(g.get("to"), 2) {
        return Some("step needs to [x, y] (the point it steps to, metres)".into());
    }
    if *kind == "turn" && num(g.get("heading")).is_none() {
        return Some("turn needs a heading (radians about z, from +x)".into());
    }
    if *kind == "step" && has("heading") && num(g.get("heading")).is_none() {
        return Some("step heading is radians about z, from +x".into());
    }
    if *kind == "sit" && !(point(g.get("seat"), 3) && num(g.get("facing")).is_some()) {
        return Some("sit needs a seat [x, y, z] (where it sits, metres) and facing (the heading it sits facing, radians)".into());
    }
    if has("activity") && !one_of(IDLE_ACTIVITIES, g.get("activity")) {
        return Some(format!("idle activity is one of {}", IDLE_ACTIVITIES.join(", ")));
    }
    if *kind == "idle" && has("at") && !point(g.get("at"), 3) {
        return Some("idle at is a point [x, y, z] (where it types or looks, metres)".into());
    }
    if has("with") && !text(g.get("with")) {
        return Some("idle with is the unit it talks with".into());
    }
    if *kind == "reach" && !point(g.get("at"), 3) {
        return Some("reach needs at [x, y, z] (the point it reaches for, metres)".into());
    }
    if has("hand") && !one_of(REACH_HANDS, g.get("hand")) {
        return Some("reach hand is left, right or both".into());
    }
    if has("seconds") && !num(g.get("seconds")).is_some_and(|v| v > 0.0) {
        return Some("seconds is a positive number".into());
    }
    if let Some(plan) = g.get("plan") {
        let ok = plan.get("policy").and_then(Json::as_str).is_some() && num(plan.get("step")).is_some_and(|s| s.fract() == 0.0);
        if !ok {
            return Some("plan is { policy, step }".into());
        }
    }
    None
}

/// A vehicle goal's problem (drive, dock, lift, stop), or None.
fn vehicle_goal_problem(g: &Json, kind: &str) -> Option<String> {
    let has = |k: &str| g.get(k).is_some();
    let route_ok = matches!(g.get("route"), Some(Json::Arr(r)) if (1..=128).contains(&r.len()) && r.iter().all(|p| point(Some(p), 2)));
    if kind == "drive" && !route_ok {
        return Some("drive needs a route: 1 to 128 points [x, y] (metres)".into());
    }
    if kind == "dock" && !(point(g.get("at"), 2) && num(g.get("heading")).is_some()) {
        return Some("dock needs at [x, y] (metres) and heading (radians)".into());
    }
    if kind == "lift" && !num(g.get("height")).is_some_and(|v| v >= 0.0) {
        return Some("lift needs a height (metres, 0 or more)".into());
    }
    if has("speed") && !num(g.get("speed")).is_some_and(|v| v > 0.0 && v <= 10.0) {
        return Some("speed is metres a second, above 0 and at most 10".into());
    }
    if has("reverse") && !matches!(g.get("reverse"), Some(Json::Bool(_))) {
        return Some("reverse is true or false".into());
    }
    if has("load") && g.get("load").and_then(Json::as_str).is_none() {
        return Some("load names a body".into());
    }
    if let Some(plan) = g.get("plan") {
        let ok = plan.get("policy").and_then(Json::as_str).is_some() && num(plan.get("step")).is_some_and(|s| s.fract() == 0.0);
        if !ok {
            return Some("plan is { policy, step }".into());
        }
    }
    None
}

/// Refuses what the engine would refuse of a `goal` command.
pub fn check_goal(cmd: &Json) -> Result<(), String> {
    if cmd.get("verb").and_then(Json::as_str) != Some("goal") {
        return Err(r#"goal: a goal command is { verb: "goal", unit, do, … }"#.into());
    }
    if !text(cmd.get("unit")) {
        return Err("goal: unit is the unit it moves".into());
    }
    match goal_problem(cmd, &["verb", "unit"]) {
        Some(no) => Err(format!("goal: {no}")),
        None => Ok(()),
    }
}

/// Refuses what the engine would refuse of a humanoid policy's own fields.
pub fn check_humanoid_policy(p: &Json) -> Result<(), String> {
    let named = match p.get("id") {
        Some(Json::Str(s)) => s.clone(),
        Some(other) => other.text(),
        None => "undefined".into(),
    };
    let no = |why: String| Err(format!("policy {named}: {why}"));
    if !text(p.get("id")) {
        return no("id is a text".into());
    }
    if let Some((k, _)) = entries(p).and_then(|e| e.iter().find(|(k, _)| !HUMANOID_POLICY_FIELDS.contains(&k.as_str()))) {
        return no(format!("unknown field {k} (it has {})", HUMANOID_POLICY_FIELDS.join(", ")));
    }
    if p.get("kind").and_then(Json::as_str) != Some("humanoid") {
        return no("kind is humanoid".into());
    }
    if !text(p.get("unit")) {
        return no("unit is the robot it drives".into());
    }
    if p.get("hz").is_some() && !num(p.get("hz")).is_some_and(|v| v > 0.0) {
        return no("hz is a positive number".into());
    }
    if p.get("lookahead").is_some() && !num(p.get("lookahead")).is_some_and(|v| v >= 0.0) {
        return no("lookahead is seconds, 0 or more".into());
    }
    if let Some(routine) = p.get("routine") {
        let Json::Arr(steps) = routine else { return no("routine is a list of goals".into()) };
        if steps.is_empty() {
            return no("routine is a list of goals".into());
        }
        for (i, g) in steps.iter().enumerate() {
            // A step of the routine is a goal, and how long to wait once it is done (`wait`, seconds).
            let why = if entries(g).is_none() {
                Some("a goal is an object".to_string())
            } else if g.get("wait").is_some() && !num(g.get("wait")).is_some_and(|v| v >= 0.0) {
                Some("wait is seconds, 0 or more".to_string())
            } else {
                goal_problem(g, &["wait"])
            };
            if let Some(why) = why {
                return no(format!("routine[{i}]: {why}"));
            }
        }
    }
    if p.get("repeat").is_some() && !matches!(p.get("repeat"), Some(Json::Bool(_))) {
        return no("repeat is true or false".into());
    }
    Ok(())
}

/// Refuses what the engine would refuse of a `motors` command's shape.
pub fn check_motors(cmd: &Json) -> Result<(), String> {
    let no = |why: &str| Err(format!("motors {why}"));
    if cmd.get("verb").and_then(Json::as_str) != Some("motors") {
        return no(r#"is { verb: "motors", set?, path?, since? }"#);
    }
    if let Some(bad) = unknown(cmd, &["verb", "set", "path", "since"], "motors") {
        return Err(bad);
    }
    let empty_set = Json::Obj(Vec::new());
    let empty_path = Json::Arr(Vec::new());
    let (set, path) = (cmd.get("set").unwrap_or(&empty_set), cmd.get("path").unwrap_or(&empty_path));
    let needs = "needs set { <constraint>: { target | velocity, stiffness?, damping? } } or path [{ at, set: { <constraint>: target } }]";
    let (Some(motors), Json::Arr(points)) = (entries(set), path) else { return no(needs) };
    if motors.is_empty() && points.is_empty() {
        return no(needs);
    }
    for (id, m) in motors {
        let ok = entries(m).is_some_and(|e| e.iter().all(|(_, v)| num(Some(v)).is_some()))
            && unknown(m, &["target", "velocity", "stiffness", "damping"], "").is_none()
            && (m.get("target").is_some() || m.get("velocity").is_some());
        if !ok {
            return no(&format!("set: {id} is {{ target | velocity, stiffness?, damping? }}"));
        }
    }
    let mut before = f64::NEG_INFINITY;
    for pt in points {
        let at = num(pt.get("at")).filter(|a| *a >= 0.0 && *a > before && *a <= 10.0);
        let (Some(at), Some(targets)) = (at, pt.get("set").and_then(entries)) else {
            return no("path: each point is { at (seconds from now, increasing, at most 10), set: { <constraint>: target } }");
        };
        before = at;
        for (id, v) in targets {
            if num(Some(v)).is_none() {
                return no(&format!("path: {id} target {} is not a number", v.text()));
            }
        }
    }
    if cmd.get("since").is_some() && num(cmd.get("since")).is_none() {
        return no("since is the scene time its path starts at, not later than now");
    }
    Ok(())
}

/// Refuses what the engine would refuse of an `assist` command's shape.
pub fn check_assist(cmd: &Json) -> Result<(), String> {
    if cmd.get("verb").and_then(Json::as_str) != Some("assist") {
        return Err(r#"assist is { verb: "assist", body, heading }"#.into());
    }
    if let Some(bad) = unknown(cmd, &["verb", "body", "heading"], "assist") {
        return Err(bad);
    }
    if !text(cmd.get("body")) {
        return Err("assist needs a body (the body whose balance aid it turns)".into());
    }
    if !matches!(cmd.get("heading"), Some(Json::Null)) && num(cmd.get("heading")).is_none() {
        return Err("assist needs a heading (radians) or null".into());
    }
    Ok(())
}

/// ARKit's 52 face units (blendshapes), each from 0 (rest) to 1. A `face` command
/// (`{ verb: "face", unit, set?: { <unit>: 0..1 }, path?: [{ at, set }], since? }`) names them.
pub const FACE_UNITS: &[&str] = &[
    "browDownLeft", "browDownRight", "browInnerUp", "browOuterUpLeft", "browOuterUpRight", "cheekPuff", "cheekSquintLeft", "cheekSquintRight",
    "eyeBlinkLeft", "eyeBlinkRight", "eyeLookDownLeft", "eyeLookDownRight", "eyeLookInLeft", "eyeLookInRight", "eyeLookOutLeft", "eyeLookOutRight",
    "eyeLookUpLeft", "eyeLookUpRight", "eyeSquintLeft", "eyeSquintRight", "eyeWideLeft", "eyeWideRight", "jawForward", "jawLeft", "jawOpen",
    "jawRight", "mouthClose", "mouthDimpleLeft", "mouthDimpleRight", "mouthFrownLeft", "mouthFrownRight", "mouthFunnel", "mouthLeft",
    "mouthLowerDownLeft", "mouthLowerDownRight", "mouthPressLeft", "mouthPressRight", "mouthPucker", "mouthRight", "mouthRollLower",
    "mouthRollUpper", "mouthShrugLower", "mouthShrugUpper", "mouthSmileLeft", "mouthSmileRight", "mouthStretchLeft", "mouthStretchRight",
    "mouthUpperUpLeft", "mouthUpperUpRight", "noseSneerLeft", "noseSneerRight", "tongueOut",
];

fn face_weights(where_: &str, w: &[(String, Json)]) -> Result<(), String> {
    for (name, v) in w {
        if !FACE_UNITS.contains(&name.as_str()) {
            return Err(format!("face {where_}: {name} is not one of ARKit's 52 face units (browDownLeft … tongueOut)"));
        }
        if !num(Some(v)).is_some_and(|v| (0.0..=1.0).contains(&v)) {
            return Err(format!("face {where_}: {name} is a number from 0 to 1"));
        }
    }
    Ok(())
}

/// Refuses what the engine would refuse of a `face` command's shape.
pub fn check_face(cmd: &Json) -> Result<(), String> {
    let no = |why: &str| Err(format!("face {why}"));
    if cmd.get("verb").and_then(Json::as_str) != Some("face") {
        return no(r#"is { verb: "face", unit, set?, path?, since? }"#);
    }
    if let Some(bad) = unknown(cmd, &["verb", "unit", "set", "path", "since"], "face") {
        return Err(bad);
    }
    if !text(cmd.get("unit")) {
        return no("needs a unit (the person whose face it moves)");
    }
    let empty_set = Json::Obj(Vec::new());
    let empty_path = Json::Arr(Vec::new());
    let (set, path) = (cmd.get("set").unwrap_or(&empty_set), cmd.get("path").unwrap_or(&empty_path));
    let needs = "needs set { <face unit>: 0 to 1 } or path [{ at, set: { <face unit>: 0 to 1 } }]";
    let (Some(weights), Json::Arr(points)) = (entries(set), path) else { return no(needs) };
    if weights.is_empty() && points.is_empty() {
        return no(needs);
    }
    face_weights("set", weights)?;
    let mut before = f64::NEG_INFINITY;
    for pt in points {
        let at = num(pt.get("at")).filter(|a| *a >= 0.0 && *a > before && *a <= 10.0);
        let (Some(at), Some(targets)) = (at, pt.get("set").and_then(entries)) else {
            return no("path: each point is { at (seconds from now, increasing, at most 10), set: { <face unit>: 0 to 1 } }");
        };
        before = at;
        face_weights("path", targets)?;
    }
    if cmd.get("since").is_some() && num(cmd.get("since")).is_none() {
        return no("since is the scene time its path starts at, not later than now");
    }
    Ok(())
}
