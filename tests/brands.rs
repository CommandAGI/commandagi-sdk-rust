// A brand in Rust against the document the TypeScript SDK's brands.test.ts pins for the same elements: each list in
// the file's order, each element's site by its path; what the vocabulary has no words for is refused by name. A
// company names each brand file with `business::brand()` (tests/business.rs).

use commandagi::design::brands::*;
use commandagi::design::facts::{about, unknown};
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
    let first = domain().name("northwind.example").registrar("Porkbun LLC").expires("2027-05-18").auto_renew(true).site("../sites/northwind/").transfer_lock(about("2026-12-30"));
    let first_site = first.source_json().unwrap();
    let root = brand_surface([
        alias().name("Northwind Survey").kind("legal"),
        logo().file("../design/logo.svg").kind("mark").background("light"),
        first,
        site().folder("../sites/northwind/"),
        site().host("cloudflare-pages").project("northwind-www").url("https://www.northwind.example"),
        mailbox().address("support@northwind.example").kind("support"),
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
            "domains":[{"name":"northwind.example","registrar":"Porkbun LLC","expires":"2027-05-18","autoRenew":true,"site":"../sites/northwind/","transferLock":{"about":"2026-12-30"}}],
            "sites":[{"folder":"../sites/northwind/"},{"url":"https://www.northwind.example","host":"cloudflare-pages","project":"northwind-www"}],
            "mailboxes":[{"address":"support@northwind.example","kind":"support"}],
            "handles":[{"platform":"x","handle":"northwindsurvey","connection":"x"}],
            "marks":[{"title":"NORTHWIND","jurisdiction":"US","number":"99123456"}]}"#)
    );
    assert_eq!(sources.get("Domain#northwind.example"), Some(&first_site));
    assert!(sources.get("Handle@0").is_some());
}

#[test]
fn a_brand_refuses_by_name() {
    assert!(declare(brand_surface([])).unwrap_err().contains("needs name"));
    assert!(declare(brand_surface([domain().name("n.example").auto_renew("yes")]).name("N")).unwrap_err().contains("domain(): auto_renew is true or false"));
    assert!(declare(brand_surface([site()]).name("N")).unwrap_err().contains("site() names its folder, or the url or project"));
    assert!(declare(brand_surface([domain().name("n.example").registrar(unknown(""))]).name("N")).unwrap_err().contains("an unknown fact is"));
    let (_, doc, _) = document(brand_surface([domain().name("my-exocortex.com").registrar(unknown("not read")), alias().name("EXOCORTEX").kind("statement-descriptor")]).name("exocortex"));
    assert_eq!(doc.get("domains"), Some(&j(r#"[{"name":"my-exocortex.com","registrar":{"unknown":"not read"}}]"#)));
    assert!(declare(brand_surface([]).name("N").voice("warm")).unwrap_err().contains("voice is a list of words"));
    assert!(declare(brand_surface([domain().name("n.example"), domain().name("n.example")]).name("N")).unwrap_err().contains("n.example"));
}
