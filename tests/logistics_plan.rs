// A supplier and a logistics plan in Rust against the documents the TypeScript SDK's logistics-plan.test.ts pins for the
// same elements: a supplier's location, carriers and contacts; a plan's BOM designs, builder and legs, a leg with no
// place ending at the builder; what the vocabulary has no words for is refused by name.

use commandagi::design::logistics::*;
use commandagi::design::{declare, Declared};

fn document(root: El) -> (String, Json) {
    match declare(root).unwrap() {
        Declared::Document { format, document, .. } => (format, document),
        Declared::Graph(_) => panic!("a logistics plan is a document"),
    }
}

fn j(text: &str) -> Json {
    Json::parse(text).unwrap()
}

#[test]
fn a_supplier_is_its_location_carriers_and_contacts() {
    let (format, doc) = document(
        supplier([
            location().name("JLCPCB, Shenzhen").lat(22.543).lon(114.058).country("CN"),
            ships().carrier("DHL").service("Express Worldwide").transit_days(5),
            contact().role("sales").email("support@jlcpcb.com"),
        ])
        .name("JLCPCB")
        .url("https://jlcpcb.com")
        .lead_time_days(8),
    );
    assert_eq!(format, "supplier");
    assert_eq!(
        doc,
        j(r#"{"name":"JLCPCB","url":"https://jlcpcb.com","leadTimeDays":8,"location":{"name":"JLCPCB, Shenzhen","lat":22.543,"lon":114.058,"country":"CN"},"ships":[{"carrier":"DHL","service":"Express Worldwide","transitDays":5}],"contacts":[{"role":"sales","email":"support@jlcpcb.com"}]}"#)
    );
    let bad = declare(supplier([location().name("Y").lat(95).lon(0)]).name("X")).unwrap_err();
    assert!(bad.contains("lat is degrees north"), "{bad}");
}

#[test]
fn a_plan_is_its_boms_builder_and_legs() {
    let (format, doc) = document(
        logistics([
            bom().ref_("Tile kit.3d.tsx"),
            builder().name("Austin").lat(30.267).lon(-97.743).country("US"),
            route([
                leg().carrier("DHL").service("Express pickup").to("DHL hub, Hong Kong").lat(22.308).lon(113.918).country("HK").transit_days(1).cost(6).currency("USD"),
                leg().carrier("DHL").service("Express Worldwide").transit_days(4).cost(32).currency("USD"),
            ])
            .supplier("suppliers/jlcpcb.supplier.tsx"),
        ])
        .name("tile kit")
        .quantity(10),
    );
    assert_eq!(format, "logistics");
    assert_eq!(
        doc,
        j(r#"{"name":"tile kit","quantity":10,"boms":[{"ref":"Tile kit.3d.tsx"}],"builder":{"name":"Austin","lat":30.267,"lon":-97.743,"country":"US"},"routes":[{"supplier":"suppliers/jlcpcb.supplier.tsx","legs":[{"carrier":"DHL","service":"Express pickup","transitDays":1,"cost":6,"currency":"USD","to":{"name":"DHL hub, Hong Kong","lat":22.308,"lon":113.918,"country":"HK"}},{"carrier":"DHL","service":"Express Worldwide","transitDays":4,"cost":32,"currency":"USD"}]}]}"#)
    );
    let bad = declare(logistics([route([leg().carrier("UPS").cost(5)]).supplier("s.supplier.tsx")])).unwrap_err();
    assert!(bad.contains("cost states its currency"), "{bad}");
    let bad = declare(logistics([]).quantity(2.5)).unwrap_err();
    assert!(bad.contains("whole number"), "{bad}");
}
