// A cart in Rust against the document the TypeScript SDK's carts.test.ts pins for the same elements: a buyer's lines at
// one seller, by SKU and by listing, and never a price.

use commandagi::design::carts::*;
use commandagi::design::{declare, Declared};

#[test]
fn the_example_is_its_document_each_site_by_its_key() {
    let first = item().sku("helmet-stl").qty(1);
    let site = first.source_json().unwrap();
    let Declared::Document { format, document, sources } =
        declare(cart([first, offer().listing("lst_print-farm-hour").qty(3)]).seller("acct:org:helmetworks")).unwrap()
    else {
        panic!("a cart is a document")
    };
    assert_eq!(format, "cart");
    assert_eq!(
        document,
        Json::parse(r#"{"seller": "acct:org:helmetworks", "items": [{"sku": "helmet-stl", "qty": 1}], "offers": [{"listing": "lst_print-farm-hour", "qty": 3}]}"#).unwrap()
    );
    assert_eq!(sources.get("item#helmet-stl"), Some(&site));
}

#[test]
fn a_cart_that_does_not_read_is_refused_with_why() {
    let u = |lines: Vec<El>| cart(lines).seller("acct:user:jacob");
    let many: Vec<El> = (0..101).map(|i| item().sku(format!("s{i}")).qty(1)).collect();
    for (c, why) in [
        (cart([]).seller("helmetworks"), "seller is an account, acct:user:<id> or acct:org:<id>"),
        (u(vec![item().sku("a").qty(0)]), "qty is a whole number from 1 to 999"),
        (u(vec![offer().listing("l").qty(1000)]), "qty is a whole number from 1 to 999"),
        (u(vec![item().sku("a").qty(1).with_attr("priceCents", 100)]), "is not read"),
        (u(vec![offer().listing("l").qty(1).with_attr("amountCents", 100)]), "is not read"),
        (u(vec![]).with_attr("totalCents", 100), "is not read"),
        (u(vec![item().sku("a").qty(1), item().sku("a").qty(2)]), r#"two item() in cart() have sku "a""#),
        (u(vec![offer().listing("l").qty(1), offer().listing("l").qty(2)]), r#"two offer() in cart() have listing "l""#),
        (u(many), "at most 100 lines"),
    ] {
        let said = declare(c).unwrap_err();
        assert!(said.contains(why), "{why}: {said}");
    }
}
