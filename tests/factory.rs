// The factory module against factory_flow.json, the golden of the three SDKs (factory-flow.golden.json in the
// TypeScript SDK): a factory the engine runs passes, each refusal is the engine's own words, and the sizing gives the
// machines the engine's sizing gives.

use commandagi::design::factory::*;
use commandagi::design::Json;

fn golden() -> Json {
    Json::parse(include_str!("factory_flow.json")).unwrap()
}
fn at<'a>(j: &'a mut Json, key: &Json) -> &'a mut Json {
    match (j, key) {
        (Json::Obj(entries), Json::Str(k)) => &mut entries.iter_mut().find(|(x, _)| x == k).unwrap().1,
        (Json::Arr(items), Json::Num(i)) => &mut items[*i as usize],
        _ => panic!("no such path"),
    }
}

#[test]
fn the_engines_refusals() {
    let g = golden();
    check_flow_system(g.get("valid").unwrap()).unwrap();
    let Some(Json::Arr(refusals)) = g.get("refusals") else { panic!() };
    for r in refusals {
        let Some(Json::Arr(path)) = r.get("change") else { panic!() };
        let mut spec = g.get("valid").unwrap().clone();
        let mut node = &mut spec;
        for k in &path[..path.len() - 2] {
            node = at(node, k);
        }
        node.set(path[path.len() - 2].as_str().unwrap(), path[path.len() - 1].clone());
        let said = check_flow_system(&spec).unwrap_err();
        let want = r.get("says").unwrap().as_str().unwrap();
        assert!(said.contains(want), "{want}: {said}");
    }
}

#[test]
fn machines_for_sizes_as_the_engine_does() {
    let g = golden();
    let Some(Json::Arr(cases)) = g.get("machines") else { panic!() };
    for c in cases {
        let n = |k: &str, d: f64| c.get(k).and_then(Json::as_f64).unwrap_or(d);
        assert_eq!(machines_for(n("rate", 0.0), n("cycle_s", 0.0), n("batch", 1.0), n("availability", 1.0), 0.85).unwrap() as f64, n("machines", -1.0), "{}", c.text());
    }
    assert!(machines_for(1.0, 0.0, 1.0, 1.0, 0.85).is_err());
}
