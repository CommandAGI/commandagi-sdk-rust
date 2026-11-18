// A storefront in Rust against the document the TypeScript SDK's storefronts.test.ts pins for the same elements: the
// store its holder lists, a mirrored store's listings or its offers by name.

use commandagi::design::storefronts::*;
use commandagi::design::{declare, Declared};

#[test]
fn a_relisted_store_is_its_document() {
    let mirror = offers().listings("Inventory/listings.csv").read("Inventory/read.mjs");
    let site = mirror.source_json().unwrap();
    let Declared::Document { format, document, sources } = declare(
        storefront([mirror]).name("DigiKey").venue("digikey").seller("DigiKey").url("https://www.digikey.com").checkout("agent").hold("10 minutes"),
    )
    .unwrap() else {
        panic!("a storefront is a document")
    };
    assert_eq!(format, "storefront");
    assert_eq!(
        document,
        Json::parse(r#"{"name": "DigiKey", "venue": "digikey", "seller": "DigiKey", "url": "https://www.digikey.com", "checkout": "agent", "hold": "10 minutes", "mirror": {"listings": "Inventory/listings.csv", "read": "Inventory/read.mjs"}, "offers": []}"#).unwrap()
    );
    assert_eq!(sources.get("Offers"), Some(&site));
}

#[test]
fn a_service_store_is_its_document() {
    let Declared::Document { document, .. } = declare(
        storefront([
            offer().name("Ride").kind("service").unit("ride").codes(["unspsc:78111804"]).tags(["transport"]),
            offer().name("Courier, one hour").kind("service").price("25.00 USD").unit("hour"),
        ])
        .name("Uber")
        .venue("uber")
        .seller("Uber Technologies")
        .seller_record("cloud://home/user-commandagi/records/companies/uber-technologies-inc-de/"),
    )
    .unwrap() else {
        panic!("a storefront is a document")
    };
    assert_eq!(
        document,
        Json::parse(r#"{"name": "Uber", "venue": "uber", "seller": "Uber Technologies", "sellerRecord": "cloud://home/user-commandagi/records/companies/uber-technologies-inc-de/", "offers": [{"name": "Ride", "kind": "service", "unit": "ride", "codes": ["unspsc:78111804"], "tags": ["transport"]}, {"name": "Courier, one hour", "kind": "service", "price": "25.00 USD", "unit": "hour"}]}"#).unwrap()
    );
}

#[test]
fn a_storefront_that_does_not_read_is_refused_with_why() {
    let s = |kids: Vec<El>| storefront(kids).name("Shop").venue("ebay").seller("A seller");
    for (el, why) in [
        (storefront([]).name("Shop").venue("eBay").seller("A seller"), "venue is the platform's word"),
        (s(vec![]).checkout("cart"), "checkout is agent or seller"),
        (s(vec![]).hold("ten minutes"), "hold is a term like \"10 minutes\""),
        (s(vec![]).url("http://x"), "url is an https:// address"),
        (s(vec![offer().name("A").kind("service").price("25 dollars")]), "price is an amount and an ISO currency"),
        (s(vec![offer().name("A").kind("thing")]), "kind is service or good"),
        (s(vec![offer().name("A").kind("good").codes(["43211500"])]), "codes: \"43211500\" is not a code: write it as scheme:code"),
        (s(vec![offer().name("A").kind("good"), offer().name("A").kind("good")]), "have name \"A\""),
        (s(vec![offers().listings("a.csv"), offers().listings("b.csv")]), "has one offers()"),
        (storefront([]).name("Shop").venue("ebay"), "needs seller"),
    ] {
        let err = declare(el).err().unwrap_or_default();
        assert!(err.contains(why), "{why}: {err}");
    }
}
