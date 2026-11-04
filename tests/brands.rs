// A brand in Rust against the document the TypeScript SDK's brands.test.ts pins for the same elements: each list in
// the file's order, each element's site by its path; what the vocabulary has no words for is refused by name. A
// company names each brand file with `business::brand()` (tests/business.rs).

use commandagi::design::brands::*;
use commandagi::design::{declare, Declared};

fn document(root: El) -> (String, Json, Json) {
    match declare(root).unwrap() {
        Declared::Document { format, document, sources } => (format, document, sources),
        Declared::Graph(_) => panic!("a brand is a document"),
    }
}

fn j(text: &str) -> Json {
    Json::parse(text).unwrap()
}

#[test]
fn a_brand_is_its_records_in_order_each_site_by_its_path() {
    let first = domain().name("northwind.example").registrar("Porkbun LLC").expires("2027-05-18").auto_renew(true).site("../sites/northwind/").mail(true);
    let first_site = first.source_json().unwrap();
    let root = brand_surface([
        alias().name("Northwind Survey").kind("legal"),
        logo().file("../design/logo.svg").kind("mark").background("light"),
        first,
        site().folder("../sites/northwind/"),
        handle().platform("x").handle("northwindsurvey").connection("x"),
        mark().title("NORTHWIND").jurisdiction("US").number("99123456"),
    ])
    .name("Northwind")
    .tagline("Drone surveys.")
    .voice(["plain", "warm"])
    .design_system("../design/brand-book.json");
    let (format, doc, sources) = document(root);
    assert_eq!(format, "brand");
    assert_eq!(
        doc,
        j(r#"{"name":"Northwind","tagline":"Drone surveys.","voice":["plain","warm"],"designSystem":"../design/brand-book.json",
            "aliases":[{"name":"Northwind Survey","kind":"legal"}],
            "logos":[{"file":"../design/logo.svg","kind":"mark","background":"light"}],
            "domains":[{"name":"northwind.example","registrar":"Porkbun LLC","expires":"2027-05-18","autoRenew":true,"site":"../sites/northwind/","mail":true}],
            "sites":[{"folder":"../sites/northwind/"}],
            "handles":[{"platform":"x","handle":"northwindsurvey","connection":"x"}],
            "marks":[{"title":"NORTHWIND","jurisdiction":"US","number":"99123456"}]}"#)
    );
    assert_eq!(sources.get("Domain#northwind.example"), Some(&first_site));
    assert!(sources.get("Handle@0").is_some());
}

#[test]
fn a_brand_refuses_by_name() {
    assert!(declare(brand_surface([])).unwrap_err().contains("needs name"));
    assert!(declare(brand_surface([domain().name("n.example").mail("yes")]).name("N")).unwrap_err().contains("domain(): mail is true or false"));
    assert!(declare(brand_surface([]).name("N").voice("warm")).unwrap_err().contains("voice is a list of words"));
    assert!(declare(brand_surface([domain().name("n.example"), domain().name("n.example")]).name("N")).unwrap_err().contains("n.example"));
}
