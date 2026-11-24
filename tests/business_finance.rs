// A company's finance in Rust against the `finance` the TypeScript SDK's business-finance.test.ts pins for the same
// elements; absent when the company names none of it; an unknown attribute refused by name.

use commandagi::design::business::*;
use commandagi::design::{declare, Declared};

fn document(root: El) -> Json {
    match declare(root).unwrap() {
        Declared::Document { document, .. } => document,
        Declared::Graph(_) => panic!("a company is a document"),
    }
}

#[test]
fn finance_is_the_company_finance() {
    let doc = document(
        company([
            budget().journal("Harbor/books/budget.journal"),
            bank_account().name("Operating").provider("First Bank").last4("4321").currency("USD").ledger("Assets:Cash:Operating").kind("checking").statements("Harbor/bank/"),
            payroll().provider("Gusto").ledger("Expenses:Payroll").register("Harbor/payroll/").schedule("semimonthly"),
            data_room([include().document("Harbor/Documents/Charter.pdf").section("corporate")]).name("Seed").expires("2026-12-31"),
        ])
        .name("Harbor"),
    );
    let want = Json::parse(
        r#"{"budget": "Harbor/books/budget.journal",
            "bankAccounts": [{"name": "Operating", "provider": "First Bank", "last4": "4321", "currency": "USD", "ledger": "Assets:Cash:Operating", "kind": "checking", "statements": "Harbor/bank/"}],
            "payroll": [{"provider": "Gusto", "ledger": "Expenses:Payroll", "register": "Harbor/payroll/", "schedule": "semimonthly"}],
            "dataRooms": [{"name": "Seed", "expires": "2026-12-31", "include": [{"document": "Harbor/Documents/Charter.pdf", "section": "corporate"}]}]}"#,
    )
    .unwrap();
    assert_eq!(doc.get("finance"), Some(&want));
    assert_eq!(document(company([]).name("Bare")).get("finance"), None, "no finance named: no field");
}

#[test]
fn a_finance_element_says_only_its_own_words() {
    let err = declare(company([bank_account().name("Op").attr("accountNumber", "000123")]).name("X")).unwrap_err();
    assert!(err.contains("bank_account() has no attribute account_number"), "{err}");
    let err = declare(company([data_room([include()]).name("Seed")]).name("X")).unwrap_err();
    assert!(err.contains("include() needs document"), "{err}");
}
