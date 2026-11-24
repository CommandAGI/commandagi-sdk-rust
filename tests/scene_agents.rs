// The scene agents against scene_agents.json, the golden of the three SDKs (scene-agents.golden.json in the TypeScript
// SDK): a scene the engine runs passes, and each refusal is the engine's own words.

use commandagi::design::scene_agents::*;
use commandagi::design::Json;

fn golden() -> Json {
    Json::parse(include_str!("scene_agents.json")).unwrap()
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
    check_scene_agents(g.get("valid").unwrap()).unwrap();
    let Some(Json::Arr(refusals)) = g.get("refusals") else { panic!() };
    for r in refusals {
        let Some(Json::Arr(path)) = r.get("change") else { panic!() };
        let mut scene = g.get("valid").unwrap().clone();
        let mut node = &mut scene;
        for k in &path[..path.len() - 2] {
            node = at(node, k);
        }
        node.set(path[path.len() - 2].as_str().unwrap(), path[path.len() - 1].clone());
        let said = check_scene_agents(&scene).unwrap_err();
        let want = r.get("says").unwrap().as_str().unwrap();
        assert!(said.contains(want), "{want}: {said}");
    }
}
