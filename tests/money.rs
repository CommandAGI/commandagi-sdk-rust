// A money account in Rust against the document the TypeScript SDK's money.test.ts pins for the same elements: its
// connections, its counterparties and their payees, and its recurring payments.

use commandagi::design::money::*;
use commandagi::design::{declare, Declared};

fn operating(rent: El) -> El {
    money_account([
        connection().id("direct").provider("mercury").account("acc_test_operating"),
        connection().id("view").provider("plaid").account("acc_test_plaid").mode("sandbox"),
        counterparty([at().connection("direct").recipient("rcp_test_landlord")]).id("landlord").name("Harbor Properties LLC"),
        counterparty([]).id("reserve").name("Reserve").file("Reserve.money.tsx"),
        rent,
    ])
    .name("Operating")
    .kind("checking")
    .institution("Mercury")
    .last4("0000")
    .currency("USD")
    .ledger("Assets:Cash:Operating")
    .books("../books/main.journal")
}

fn rent() -> El {
    recurring().id("rent").to("landlord").amount("4200.00").rail("ach").every("month").day("1").by("person:mock-founder").starts("2027-01-01").memo("Rent")
}

#[test]
fn a_money_account_is_its_document() {
    let Declared::Document { format, document, .. } = declare(operating(rent())).unwrap() else {
        panic!("a money account is a document")
    };
    assert_eq!(format, "money");
    assert_eq!(
        document,
        Json::parse(r#"{"name": "Operating", "kind": "checking", "institution": "Mercury", "last4": "0000", "currency": "USD", "ledger": "Assets:Cash:Operating", "books": "../books/main.journal", "connections": [{"id": "direct", "provider": "mercury", "account": "acc_test_operating"}, {"id": "view", "provider": "plaid", "account": "acc_test_plaid", "mode": "sandbox"}], "counterparties": [{"id": "landlord", "name": "Harbor Properties LLC", "at": [{"connection": "direct", "recipient": "rcp_test_landlord"}]}, {"id": "reserve", "name": "Reserve", "file": "Reserve.money.tsx"}], "recurring": [{"id": "rent", "to": "landlord", "amount": "4200.00", "rail": "ach", "every": "month", "day": 1, "by": "person:mock-founder", "starts": "2027-01-01", "memo": "Rent"}]}"#).unwrap()
    );
}

#[test]
fn a_money_account_that_does_not_read_is_refused_with_why() {
    let bare = |kids: Vec<El>| money_account(kids).name("O").kind("checking");
    for (el, why) in [
        (money_account([]).name("O").kind("piggy-bank"), "kind is checking"),
        (money_account([]).name("O").kind("checking").last4("123456"), "last4 is the last four digits"),
        (bare(vec![connection().id("x").provider("acme").account("a")]), "provider is stripe-financial-connections"),
        (bare(vec![counterparty([]).id("a").name("A")]), "needs an <At connection recipient>"),
        (bare(vec![counterparty([at().connection("nowhere").recipient("r")]).id("a").name("A")]), "<At> connection is the id of a <Connection>"),
        (operating(rent().to("nobody")), "to is the id of a <Counterparty>"),
        (operating(rent().amount(4200)), "amount is a decimal text"),
        (operating(rent().rail("swift")), "rail is ach"),
        (operating(rent().every("week").day(9)), "day is a whole number from 1 to 7"),
        (operating(rent().by("jacob")), "by is the principal"),
    ] {
        let err = match declare(el) {
            Err(e) => e,
            Ok(_) => panic!("{why}: it read"),
        };
        assert!(err.contains(why), "{why}: {err}");
    }
}
