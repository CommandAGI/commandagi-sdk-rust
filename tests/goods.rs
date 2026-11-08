// A good in Rust against the document the TypeScript SDK's goods.test.ts pins for the same elements: the goods door's
// body, a digital good and a physical good with a pre-order deposit; a good that does not read is refused with why.

use commandagi::design::goods::*;
use commandagi::design::{declare, Declared};

fn document(root: El) -> (String, Json, Json) {
    match declare(root).unwrap() {
        Declared::Document { format, document, sources } => (format, document, sources),
        Declared::Graph(_) => panic!("a good is a document"),
    }
}

const SHIPPING: &str = r#"{"zones": [{"rule": "included", "countries": ["US"], "allowanceCents": 2000}]}"#;

fn phys() -> El {
    good([]).name("x").kind("physical").shipping(json(SHIPPING))
}

#[test]
fn the_examples_are_their_documents_each_site_by_its_sku() {
    let first = variant().sku("helmet-stl").name("STL pack").price_cents(1500).file("./helmet.zip");
    let site = first.source_json().unwrap();
    let (format, doc, sources) = document(good([first]).name("Helmet files").kind("digital").status("active"));
    assert_eq!(format, "good");
    assert_eq!(
        doc,
        Json::parse(r#"{"name": "Helmet files", "kind": "digital", "status": "active",
            "variants": [{"sku": "helmet-stl", "name": "STL pack", "priceCents": 1500, "file": "./helmet.zip"}]}"#)
        .unwrap()
    );
    assert_eq!(sources.get("variant#helmet-stl"), Some(&site));
    let (_, pre, _) = document(
        good([
            variant().sku("da-1").name("Assembled").price_cents(349600).inventory_qty(-1).weight_grams(4200),
            variant().sku("da-1-preorder").name("Pre-order, 20% now").price_cents(349600).deposit(json(r#"{"percent": 20}"#)),
        ])
        .name("Desk assistant")
        .kind("physical")
        .shipping(json(SHIPPING)),
    );
    assert_eq!(
        pre,
        Json::parse(&format!(
            r#"{{"name": "Desk assistant", "kind": "physical", "shipping": {SHIPPING}, "variants": [
                {{"sku": "da-1", "name": "Assembled", "priceCents": 349600, "inventoryQty": -1, "weightGrams": 4200}},
                {{"sku": "da-1-preorder", "name": "Pre-order, 20% now", "priceCents": 349600, "deposit": {{"percent": 20}}}}]}}"#
        ))
        .unwrap()
    );
}

#[test]
fn a_good_that_does_not_read_is_refused_with_why() {
    let v = || variant().sku("a").name("b").price_cents(100);
    let dig = |vs: Vec<El>| good(vs).name("x").kind("digital");
    let ph = |vs: Vec<El>| good(vs).name("x").kind("physical").shipping(json(SHIPPING));
    for (g, why) in [
        (good([v()]).name("x").kind("service"), "kind is physical or digital"),
        (dig(vec![v().file("f.zip")]).shipping(json(SHIPPING)), "is digital: it ships nothing"),
        (dig(vec![v()]), "is digital: file names the file it sells"),
        (dig(vec![v().file("f.zip").deposit(json(r#"{"percent": 10}"#))]), "is digital: it takes no deposit"),
        (dig(vec![v().file("f.zip").weight_grams(10)]), "is digital: it has no weightGrams"),
        (good([v()]).name("x").kind("physical"), "is physical: shipping is its shipping policy"),
        (ph(vec![v().file("f.zip")]), "is physical: it ships, so it names no file"),
        (ph(vec![v().deposit(json(r#"{"percent": 100}"#))]), "deposit.percent is a whole number from 1 to 99"),
        (ph(vec![v().deposit(json(r#"{"cents": 100}"#))]), "less than the price (100)"),
        (ph(vec![v().price_cents(10)]), "priceCents is a whole number of cents, at least 50"),
        (phys().with_attr("colour", "red"), "colour is not read"),
    ] {
        let said = declare(g).unwrap_err();
        assert!(said.contains(why), "{why}: {said}");
    }
    let twice = declare(ph(vec![v(), v()])).unwrap_err();
    assert!(twice.contains(r#"two variant() in good() have sku "a""#) || twice.contains("have sku"), "{twice}");
}
