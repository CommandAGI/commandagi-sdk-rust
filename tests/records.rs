// Signed records in Rust against the documents the TypeScript SDK's records.test.ts pins for the same elements: a
// contract's terms come back as written (nulls and empty strings kept), parties and events are children in order; an
// event chain out of order and a field with no attribute are refused by name.

use commandagi::design::records::*;
use commandagi::design::{declare, Declared};

fn document(root: El) -> (String, Json, Json) {
    match declare(root).unwrap() {
        Declared::Document { format, document, sources } => (format, document, sources),
        Declared::Graph(_) => panic!("a record is a document"),
    }
}

fn bad(root: El) -> String {
    declare(root).unwrap_err()
}

fn j(text: &str) -> Json {
    Json::parse(text).unwrap()
}

const TERMS: &str = r#"{"subject": {"kind": "data_stream", "resourceType": "device", "resourceId": "dev-1"}, "title": "Bench camera", "description": "",
    "price": {"type": "metered", "rate": 1, "unit": "minute"}, "schedule": null, "takeRateBps": 1000, "channels": ["screen"]}"#;

#[test]
fn a_contract_is_its_body_terms_as_written_parties_in_order() {
    let offeror = party().principal("u-1").role("offeror").sig("ed25519:k:abc").signed_at(1780765664000_i64);
    let offeror_site = offeror.source_json().unwrap();
    let root = contract([offeror, party().principal("acct:org:o-2").role("acceptor").acted_by("u-3")])
        .id("c-1")
        .terms(json(TERMS))
        .created_at(1780765664000_i64);
    let root_site = root.source_json().unwrap();
    let (format, doc, sources) = document(root);
    assert_eq!(format, "contract");
    assert_eq!(sources.get(""), Some(&root_site));
    assert_eq!(sources.get("party@0"), Some(&offeror_site));
    assert_eq!(
        doc,
        j(&format!(
            r#"{{"id": "c-1", "terms": {TERMS}, "parties": [{{"principal": "u-1", "role": "offeror", "sig": "ed25519:k:abc", "signedAt": 1780765664000}},
                {{"principal": "acct:org:o-2", "role": "acceptor", "actedBy": "u-3"}}], "createdAt": 1780765664000}}"#
        ))
    );
    assert_eq!(doc.get("terms").unwrap().text(), j(TERMS).text(), "the signed object, null and \"\" kept, in its order");

    assert!(bad(contract([]).id("c").terms(json(TERMS))).contains("contract() needs created_at"));
    assert!(bad(contract([]).id("c").terms(json(TERMS)).created_at(1).with_attr("title", "x")).contains("title is not read"));
    assert!(bad(contract([]).id("c").terms(json(TERMS)).created_at("today")).contains(r#"created_at is epoch milliseconds, not "today""#));
    assert!(bad(contract([]).id("c").terms("x").created_at(1)).contains("terms is the object the parties sign"));
    assert!(bad(contract([party().role("offeror")]).id("c").terms(json(TERMS)).created_at(1)).contains("party() needs principal"));
}

#[test]
fn a_product_instance_is_its_chain_oldest_first_each_by_its_seq() {
    let first = event().seq(0).kind("manufacture").at(1).prev(json("null")).by("org-acme").data(json(r#"{"plate": "P-1"}"#)).sig("ed25519:k:s0");
    let first_site = first.source_json().unwrap();
    let root = instance([first, event().seq(1).kind("transfer").at(2).prev("h0").by("org-acme").to("u-2").contract_ref("c-1").sig("ed25519:k:s1")])
        .serial("SN-1")
        .product_id("fleet-unit");
    let (format, doc, sources) = document(root);
    assert_eq!(format, "instance");
    assert_eq!(sources.get("event#0"), Some(&first_site));
    assert_eq!(
        doc,
        j(r#"{"serial": "SN-1", "productId": "fleet-unit", "events": [
              {"seq": 0, "kind": "manufacture", "at": 1, "prev": null, "by": "org-acme", "data": {"plate": "P-1"}, "sig": "ed25519:k:s0"},
              {"seq": 1, "kind": "transfer", "at": 2, "prev": "h0", "by": "org-acme", "to": "u-2", "contractRef": "c-1", "sig": "ed25519:k:s1"}]}"#)
    );
    assert!(bad(instance([event().seq(1).kind("service").at(1).by("x")]).serial("S").product_id("p")).contains("events are written oldest first"));
    assert!(bad(instance([event().seq(0).kind("service").at(1).by("x").with_attr("note", "n")]).serial("S").product_id("p")).contains("note is not read"));
}
