// A product in Rust against the document the TypeScript SDK's products.test.ts pins for the same elements: a digital
// product, and a physical product with options, stock, a pre-order deposit and a discount; a product that does not read
// is refused with why.

use commandagi::design::products::*;
use commandagi::design::{declare, Declared};

fn document(root: El) -> (String, Json, Json) {
    match declare(root).unwrap() {
        Declared::Document { format, document, sources } => (format, document, sources),
        Declared::Graph(_) => panic!("a product is a document"),
    }
}

const SHIPPING: &str = r#"{"zones": [{"rule": "included", "countries": ["US"], "allowanceCents": 2000}]}"#;

#[test]
fn the_examples_are_their_documents_each_site_by_its_key() {
    let first = variant().sku("helmet-stl").name("STL pack").price_cents(1500).file("./helmet.zip");
    let site = first.source_json().unwrap();
    let (format, doc, sources) = document(product([first]).name("Helmet files").kind("digital").status("active"));
    assert_eq!(format, "product");
    assert_eq!(
        doc,
        Json::parse(r#"{"name": "Helmet files", "kind": "digital", "status": "active",
            "variants": [{"sku": "helmet-stl", "name": "STL pack", "priceCents": 1500, "file": "./helmet.zip"}], "discounts": []}"#)
        .unwrap()
    );
    assert_eq!(sources.get("variant#helmet-stl"), Some(&site));
    let launch = discount().code("Launch10").percent(10).starts_at("2026-11-08").ends_at("2026-12-01T00:00:00Z").usage_limit(100);
    let launch_site = launch.source_json().unwrap();
    let (_, pre, sources) = document(
        product([
            variant()
                .sku("da-1-black")
                .name("Assembled, black")
                .price_cents(349600)
                .compare_at_cents(399600)
                .stock(12)
                .options(json(r#"{"colour": "black", "assembled": true}"#))
                .weight_grams(4200),
            variant().sku("da-1-preorder").name("Pre-order, 20% now").price_cents(349600).deposit(json(r#"{"percent": 20}"#)),
            launch,
        ])
        .name("Desk assistant")
        .kind("physical")
        .shipping(json(SHIPPING)),
    );
    assert_eq!(
        pre,
        Json::parse(&format!(
            r#"{{"name": "Desk assistant", "kind": "physical", "shipping": {SHIPPING}, "variants": [
                {{"sku": "da-1-black", "name": "Assembled, black", "priceCents": 349600, "compareAtCents": 399600, "stock": 12,
                  "options": {{"colour": "black", "assembled": true}}, "weightGrams": 4200}},
                {{"sku": "da-1-preorder", "name": "Pre-order, 20% now", "priceCents": 349600, "deposit": {{"percent": 20}}}}],
              "discounts": [{{"code": "Launch10", "percent": 10, "startsAt": "2026-11-08", "endsAt": "2026-12-01T00:00:00Z", "usageLimit": 100}}]}}"#
        ))
        .unwrap()
    );
    assert_eq!(sources.get("discount#Launch10"), Some(&launch_site));
}

#[test]
fn a_product_that_does_not_read_is_refused_with_why() {
    let v = || variant().sku("a").name("b").price_cents(100);
    let d = || discount().code("SAVE5").amount_cents(500);
    let dig = |vs: Vec<El>| product(vs).name("x").kind("digital");
    let ph = |vs: Vec<El>| product(vs).name("x").kind("physical").shipping(json(SHIPPING));
    for (g, why) in [
        (product([v()]).name("x").kind("service"), "kind is physical or digital"),
        (dig(vec![v().file("f.zip")]).shipping(json(SHIPPING)), "is digital: it ships nothing"),
        (dig(vec![v()]), "is digital: file names the file it sells"),
        (dig(vec![v().file("f.zip").deposit(json(r#"{"percent": 10}"#))]), "is digital: it takes no deposit"),
        (dig(vec![v().file("f.zip").weight_grams(10)]), "is digital: it has no weightGrams"),
        (dig(vec![v().file("f.zip").stock(3)]), "is digital: it has no stock"),
        (product([v()]).name("x").kind("physical"), "is physical: shipping is its shipping policy"),
        (ph(vec![v().file("f.zip")]), "is physical: it ships, so it names no file"),
        (ph(vec![v().stock(-1)]), "stock is a whole number, at least 0"),
        (ph(vec![v().with_attr("inventoryQty", 3)]), "is not read"),
        (ph(vec![v().compare_at_cents(100)]), "more than the price (100)"),
        (ph(vec![v().options(json(r#"{"colour": ["red"]}"#))]), "options is an object of at most 20 names"),
        (ph(vec![v().deposit(json(r#"{"percent": 100}"#))]), "deposit.percent is a whole number from 1 to 99"),
        (ph(vec![v().deposit(json(r#"{"cents": 100}"#))]), "less than the price (100)"),
        (ph(vec![v().price_cents(10)]), "priceCents is a whole number of cents, at least 50"),
        (ph(vec![v()]).fulfilment("ship"), "fulfilment is an object"),
        (ph(vec![v()]).with_attr("colour", "red"), "colour is not read"),
        (ph(vec![v()]).tax_code("txcd_1"), "taxCode is a Stripe product tax code like txcd_99999999"),
        (ph(vec![v(), discount().code("a b c").percent(5)]), "code is letters, digits, - and _ (3 to 40)"),
        (ph(vec![v(), discount().code("SAVE5")]), "takes off one of percent or amountCents, not neither"),
        (ph(vec![v(), d().percent(5)]), "takes off one of percent or amountCents, not both"),
        (ph(vec![v(), discount().code("SAVE5").percent(101)]), "percent is a whole number from 1 to 100"),
        (ph(vec![v(), discount().code("SAVE5").amount_cents(0)]), "amountCents is a whole number of cents, at least 1"),
        (ph(vec![v(), d().starts_at("next week")]), "startsAt is an ISO 8601 date or date-time"),
        (ph(vec![v(), d().ends_at("2026-02-30")]), "endsAt is an ISO 8601 date or date-time"),
        (ph(vec![v(), d().starts_at("2026-12-01").ends_at("2026-11-30T23:00:00+02:00")]), "is after startsAt"),
        (ph(vec![v(), d().usage_limit(0)]), "usageLimit is a whole number, at least 1"),
        (ph(vec![v(), d(), d().code("save5")]), r#"two discount()s with one code: "SAVE5" and "save5""#),
    ] {
        let said = declare(g).unwrap_err();
        assert!(said.contains(why), "{why}: {said}");
    }
    let twice = declare(ph(vec![v(), v()])).unwrap_err();
    assert!(twice.contains(r#"two variant() in product() have sku "a""#), "{twice}");
    declare(ph(vec![v(), d().starts_at("2026-12-01").ends_at("2026-11-30T23:00:00-03:00")])).expect("the offset moves the end past the start");
}
