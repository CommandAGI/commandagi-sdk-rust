// A test plan, a firmware release and a rollout in Rust against the documents the TypeScript SDK's testplans.test.ts
// pins for the same elements: steps in order, each site by its path, what a tag may not say refused by name, and a
// unit's serial and revision read from a world.

use commandagi::design::{declare, Declared, Json};

fn document(root: commandagi::design::El) -> (String, Json, Json) {
    match declare(root).unwrap() {
        Declared::Document { format, document, sources } => (format, document, sources),
        Declared::Graph(_) => panic!("a test plan, a release and a rollout are documents"),
    }
}

fn bad(root: commandagi::design::El) -> String {
    declare(root).unwrap_err()
}

fn j(text: &str) -> Json {
    Json::parse(text).unwrap()
}

const SHA: &str = "9f2c000000000000000000000000000000000000000000000000000000000000";

mod testplan {
    use super::{bad, document, j};
    use commandagi::design::testplans::*;

    #[test]
    fn a_test_plan_is_its_steps_in_order_each_site_by_its_path() {
        let supply = fixture().id("supply").kind("instrument").unit("psu-1").channel("output");
        let supply_site = supply.source_json().unwrap();
        let root = testplan([
            condition().id("ambient").quantity("temperature").min(15).max(35).unit("C"),
            supply,
            step().id("rail-3v3").channel("power").command(json(r#"{"verb": "read"}"#)).read("v3v3").min(3.2).max(3.4).unit("V").verifies(["req.tsx#rail"]),
            step().id("current").fixture("supply").read("current").max(0.4).settle_ms(500),
        ])
        .title("Cam-001 power rail")
        .device("../devices/cam/definition.tsx");
        let root_site = root.source_json().unwrap();
        let (format, doc, sources) = document(root);
        assert_eq!(format, "testplan");
        assert_eq!(
            doc,
            j(r#"{"title": "Cam-001 power rail", "device": "../devices/cam/definition.tsx",
                  "conditions": [{"id": "ambient", "quantity": "temperature", "min": 15, "max": 35, "unit": "C"}],
                  "fixtures": [{"id": "supply", "kind": "instrument", "unit": "psu-1", "channel": "output"}],
                  "steps": [{"id": "rail-3v3", "channel": "power", "command": {"verb": "read"}, "read": "v3v3", "min": 3.2, "max": 3.4, "unit": "V", "verifies": ["req.tsx#rail"]},
                            {"id": "current", "fixture": "supply", "read": "current", "max": 0.4, "settleMs": 500}]}"#)
        );
        assert_eq!(sources.get(""), Some(&root_site));
        assert_eq!(sources.get("fixture#supply"), Some(&supply_site));
        assert!(sources.get("step#rail-3v3").is_some());

        assert!(bad(testplan([step().read("x")])).contains("step() needs id"));
        assert!(bad(testplan([step().id("a").with_attr("limit", 3)])).contains("limit is not read"));
        assert!(bad(testplan([step().id("a").verifies("req.tsx")])).contains("verifies is a list of refs"));
        assert!(bad(testplan([step().id("a").command("read")])).contains("command is one record"));
        assert!(bad(testplan([step().id("a"), step().id("a")])).contains(r#"two step() in testplan() have id "a""#));
    }
}

mod firmware {
    use super::{bad, document, j, SHA};
    use commandagi::design::firmware::*;

    #[test]
    fn a_release_states_its_digest_sbom_device_and_revisions() {
        let root = firmware([supports().revision("B"), supports().revision("C")])
            .version("1.4.0")
            .image("build/cam.bin")
            .sha256(SHA)
            .size(1024)
            .sbom("build/cam.cdx.json")
            .device("../devices/cam/definition.tsx")
            .security_version(2);
        let (format, doc, _) = document(root);
        assert_eq!(format, "firmware");
        assert_eq!(
            doc,
            j(&format!(
                r#"{{"version": "1.4.0", "image": "build/cam.bin", "sha256": "{SHA}", "size": 1024, "sbom": "build/cam.cdx.json",
                     "device": "../devices/cam/definition.tsx", "securityVersion": 2, "supports": [{{"revision": "B"}}, {{"revision": "C"}}]}}"#
            ))
        );
        assert!(bad(firmware([]).version("1").image("a.bin").sha256(SHA).device("d.tsx")).contains("firmware() needs sbom"));
        assert!(bad(firmware([]).version("1").image("a.bin").sha256("ABC").sbom("s.json").device("d.tsx")).contains("64 lowercase hex digits"));
    }

    #[test]
    fn a_rollout_its_stages_health_checks_and_targets() {
        let root = rollout([
            health().id("no-crash").channel("state").command(json(r#"{"verb": "read"}"#)).read("crashes").max(0),
            target().unit("cam-1"),
            target().unit("cam-2"),
        ])
        .firmware("cam.firmware.tsx")
        .world("worlds/fleet/world.tsx")
        .channel("firmware")
        .canary(10)
        .waves([50, 100])
        .settle_ms(5000);
        let (format, doc, sources) = document(root);
        assert_eq!(format, "rollout");
        assert_eq!(
            doc,
            j(r#"{"firmware": "cam.firmware.tsx", "world": "worlds/fleet/world.tsx", "channel": "firmware", "canary": 10, "waves": [50, 100], "settleMs": 5000,
                  "health": [{"id": "no-crash", "channel": "state", "command": {"verb": "read"}, "read": "crashes", "max": 0}],
                  "targets": [{"unit": "cam-1"}, {"unit": "cam-2"}]}"#)
        );
        assert!(sources.get("target#cam-2").is_some());
        assert!(bad(rollout([]).firmware("a.firmware.tsx").world("w.tsx").channel("firmware").canary(10)).contains("rollout() needs waves"));
    }
}

mod unit {
    use super::{document, j};
    use commandagi::design::ontology::*;

    #[test]
    fn a_unit_says_its_serial_and_its_hardware_revision() {
        let root = world([unit().uid("cam-1").name("cam").device("d.tsx").serial("CAM-000123").revision("B").position([0, 0, 0]).rotation(0)])
            .name("fleet")
            .kind("physical");
        let (_, doc, _) = document(root);
        assert_eq!(
            doc.get("units"),
            Some(&j(r#"[{"uid": "cam-1", "name": "cam", "device": "d.tsx", "serial": "CAM-000123", "revision": "B", "position": [0, 0, 0], "rotation": 0}]"#))
        );
    }
}
