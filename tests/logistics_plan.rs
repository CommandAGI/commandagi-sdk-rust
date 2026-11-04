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
        j(r#"{"name":"JLCPCB","url":"https://jlcpcb.com","leadTimeDays":8,"location":{"name":"JLCPCB, Shenzhen","lat":22.543,"lon":114.058,"country":"CN"},"ships":[{"carrier":"DHL","service":"Express Worldwide","transitDays":5}],"contacts":[{"role":"sales","email":"support@jlcpcb.com"}],"services":[]}"#)
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
        j(r#"{"name":"tile kit","quantity":10,"boms":[{"ref":"Tile kit.3d.tsx"}],"builder":{"name":"Austin","lat":30.267,"lon":-97.743,"country":"US"},"routes":[{"supplier":"suppliers/jlcpcb.supplier.tsx","legs":[{"carrier":"DHL","service":"Express pickup","transitDays":1,"cost":6,"currency":"USD","to":{"name":"DHL hub, Hong Kong","lat":22.308,"lon":113.918,"country":"HK"}},{"carrier":"DHL","service":"Express Worldwide","transitDays":4,"cost":32,"currency":"USD"}]}],"sources":[]}"#)
    );
    let bad = declare(logistics([route([leg().carrier("UPS").cost(5)]).supplier("s.supplier.tsx")])).unwrap_err();
    assert!(bad.contains("cost states its currency"), "{bad}");
    let bad = declare(logistics([]).quantity(2.5)).unwrap_err();
    assert!(bad.contains("whole number"), "{bad}");
}

#[test]
fn a_supplier_offers_services_with_their_days_and_fees() {
    let (_, doc) = document(
        supplier([service([fee().amount(20).currency("USD").per("line").note("connectors and batteries"), fee().amount(0.017).currency("USD").per("unit")])
            .id("global-sourcing")
            .name("Global sourcing")
            .review_days(1)
            .review_days_high(3)
            .lead_time_days(9)
            .lead_time_days_high(20)])
        .name("JLCPCB"),
    );
    assert_eq!(
        doc.get("services").unwrap(),
        &j(r#"[{"id":"global-sourcing","name":"Global sourcing","reviewDays":1,"reviewDaysHigh":3,"leadTimeDays":9,"leadTimeDaysHigh":20,"fees":[{"amount":20,"currency":"USD","per":"line","note":"connectors and batteries"},{"amount":0.017,"currency":"USD","per":"unit"}]}]"#)
    );
    let bad = declare(supplier([service([fee().amount(70).per("line").each(50)]).id("consignment")]).name("JLCPCB")).unwrap_err();
    assert!(bad.contains("currency"), "{bad}");
}

#[test]
fn a_plan_states_its_build_date_and_how_a_line_is_filled() {
    let (_, doc) = document(
        logistics([source([use_().offer("JLCPCB parts library").qty(1984), use_().offer("LCSC")]).part("KH-205580-DZ").strategy("split")])
            .name("helmets")
            .quantity(10)
            .build("2027-01-15"),
    );
    assert_eq!(doc.get("build").unwrap(), &j(r#""2027-01-15""#));
    assert_eq!(
        doc.get("sources").unwrap(),
        &j(r#"[{"part":"KH-205580-DZ","strategy":"split","uses":[{"offer":"JLCPCB parts library","qty":1984},{"offer":"LCSC"}]}]"#)
    );
    let bad = declare(logistics([source([use_().offer("LCSC")]).part("X").strategy("cheapest")])).unwrap_err();
    assert!(bad.contains("strategy is single, fill, split or dual"), "{bad}");
}

#[test]
fn a_leg_that_says_quote_this_states_its_want_parcel_and_postal_codes() {
    let (_, doc) = document(
        logistics([
            builder().name("the builder, San Francisco").lat(37.72).lon(-122.44).country("US").postal("94112"),
            route([leg().carrier("USPS").quote("cheapest").kg(2.5).mm([300, 250, 220]).to("a buyer, Berlin").lat(52.52).lon(13.4).country("DE").postal("10115")])
                .supplier("suppliers/x.supplier.tsx"),
        ])
        .name("helmet"),
    );
    assert_eq!(doc.get("builder").unwrap(), &j(r#"{"name":"the builder, San Francisco","lat":37.72,"lon":-122.44,"country":"US","postal":"94112"}"#));
    assert_eq!(
        doc.get("routes").unwrap(),
        &j(r#"[{"supplier":"suppliers/x.supplier.tsx","legs":[{"carrier":"USPS","quote":"cheapest","kg":2.5,"mm":[300,250,220],"to":{"name":"a buyer, Berlin","lat":52.52,"lon":13.4,"country":"DE","postal":"10115"}}]}]"#)
    );
    let bad = |l: El| declare(logistics([route([l]).supplier("s.supplier.tsx")])).err().unwrap();
    assert!(bad(leg().carrier("UPS").quote("the cheap one")).contains("quote is cheapest, fastest"));
    assert!(bad(leg().carrier("UPS").quote("ups/03")).contains("quote needs its parcel"));
    assert!(bad(leg().carrier("UPS").quote("fastest").kg(2).mm([300, 250])).contains("mm is the parcel"));
}
