// A market recording in Rust against the document the TypeScript SDK's market.test.ts pins for the same elements:
// `paper()` is the body of the shipped `Paper.market.rs`, and `market_paper.json` is the document its TypeScript and
// Python twins declare. Each field is text; what the vocabulary cannot read is refused by name.

use commandagi::design::market::*;
use commandagi::design::{declare, Declared};

fn document(root: El) -> (String, Json, Json) {
    match declare(root).unwrap() {
        Declared::Document { format, document, sources } => (format, document, sources),
        Declared::Graph(_) => panic!("a market recording is a document"),
    }
}

fn bad(root: El) -> String {
    declare(root).unwrap_err()
}

const T: &str = "2026-10-01T13:30:00.000Z";

#[rustfmt::skip]
fn paper() -> El {
    market([
        note().t("2026-10-01T13:30:00.000Z").text("Example data: a fictional instrument and a paper account. Not a market, not advice."),
        quote().t("2026-10-01T13:30:00.000Z").sym("NRTH").last("185.09").bid("185.05").ask("185.13").vol("400"),
        book([
            bid().price("185.05").qty("300"),
            bid().price("185.00").qty("120"),
            ask().price("185.13").qty("200"),
        ])
        .t("2026-10-01T13:30:01.000Z")
        .sym("NRTH"),
        trade().t("2026-10-01T13:30:02.000Z").sym("NRTH").price("185.10").qty("25").side("buy"),
        news().t("2026-10-01T13:30:03.000Z").sym("NRTH").headline("Northwind opens a second plant").tone("good"),
        position().t("2026-10-01T13:30:04.000Z").sym("NRTH").qty("120").avg("181.5"),
        order().t("2026-10-01T13:31:00.000Z").id("o-a1b2c3d4e5f6").sym("NRTH").side("buy").qty("10").type_("limit").limit("184.50").tif("day").venue("alpaca-paper").mode("paper").by("person:ada").grant("g-paper").connection("conn-1"),
        order_update().t("2026-10-01T13:31:01.000Z").order("o-a1b2c3d4e5f6").state("accepted").mode("paper").ref_("alp-1001"),
        amend().t("2026-10-01T13:32:00.000Z").id("m-a1b2c3d4e5f6").order("o-a1b2c3d4e5f6").sym("NRTH").side("buy").qty("10").type_("limit").limit("184.80").tif("day").venue("alpaca-paper").mode("paper").ref_("alp-1001").by("person:ada").grant("g-paper").connection("conn-1"),
        amend_update().t("2026-10-01T13:32:01.000Z").amend("m-a1b2c3d4e5f6").state("requested").mode("paper").ref_("alp-1002"),
        fill().t("2026-10-01T13:33:00.000Z").sym("NRTH").side("buy").price("184.80").qty("4").by("person:ada").order("o-a1b2c3d4e5f6").mode("paper").ref_("fill-1").at("2026-10-01T13:32:40.000Z"),
        order_update().t("2026-10-01T13:33:00.000Z").order("o-a1b2c3d4e5f6").state("partial").mode("paper").ref_("alp-1002").filled("4").avg("184.80").src("venue"),
        account([
            holding().sym("NRTH").side("long").qty("4").avg("184.80").price("185.10").value("740.40").pl("1.20").pl_pct("0.0016"),
            open_order().ref_("alp-1002").client("o-a1b2c3d4e5f6").order("o-a1b2c3d4e5f6").sym("NRTH").side("buy").type_("limit").tif("day").status("partially_filled").state("partial").qty("10").filled("4").avg_price("184.80").limit("184.80"),
            problem().text("Example: one page of executions was read."),
        ])
        .t("2026-10-01T13:33:00.000Z")
        .venue("alpaca-paper")
        .mode("paper")
        .connection("conn-1")
        .currency("USD")
        .status("ACTIVE")
        .value("100739.20")
        .buying_power("198521.60")
        .cash("99260.80")
        .day_change("-12.40")
        .day_change_pct("-0.000123"),
        cancel().t("2026-10-01T13:34:00.000Z").id("x-a1b2c3d4e5f6").venue("alpaca-paper").mode("paper").ref_("alp-1002").order("o-a1b2c3d4e5f6").connection("conn-1").by("person:ada").grant("g-paper"),
        cancel_update().t("2026-10-01T13:34:01.000Z").cancel("x-a1b2c3d4e5f6").state("requested").mode("paper"),
        order_update().t("2026-10-01T13:35:00.000Z").order("o-a1b2c3d4e5f6").state("cancelled").mode("paper").ref_("alp-1002").filled("4").avg("184.80").src("venue"),
        order().t("2026-10-01T13:36:00.000Z").id("o-0f1e2d3c4b5a").sym("NRTH").side("sell").qty("2").type_("market").tif("ioc").venue("sim").mode("sim").by("agent:trader").grant("g-sim"),
        fill().t("2026-10-01T13:36:00.000Z").sym("NRTH").side("sell").price("185.05").qty("2").by("agent:trader").order("o-0f1e2d3c4b5a").mode("sim"),
        order_update().t("2026-10-01T13:36:00.000Z").order("o-0f1e2d3c4b5a").state("filled").mode("sim"),
    ])
}

#[test]
fn the_paper_recording_declares_the_document_its_twins_declare() {
    let (format, doc, sources) = document(paper());
    assert_eq!(format, "market");
    assert_eq!(doc, Json::parse(include_str!("market_paper.json")).unwrap());
    assert!(sources.get("book@0/bid@1").is_some());
    assert!(sources.get("account@0/open-order@0").is_some());
}

#[test]
fn an_empty_recording_declares_no_records() {
    // The host writes an empty recording this way.
    #[rustfmt::skip]
    let root = market([
    ]);
    assert_eq!(document(root).1, empty());
    assert_eq!(empty(), Json::parse(r#"{"records": []}"#).unwrap());
}

#[test]
fn a_book_says_both_sides_and_an_account_only_what_it_holds() {
    let (_, doc, _) = document(market([book([]).t(T).sym("NRTH"), account([]).t(T).venue("sim").mode("sim")]));
    assert_eq!(
        doc,
        Json::parse(&format!(
            r#"{{"records": [{{"kind": "book", "t": "{T}", "sym": "NRTH", "bids": [], "asks": []}},
                             {{"kind": "account", "t": "{T}", "venue": "sim", "mode": "sim"}}]}}"#
        ))
        .unwrap()
    );
}

#[test]
fn what_the_vocabulary_cannot_read_is_refused_by_name() {
    // A float would round a price: a number is refused even where the method takes one.
    assert_eq!(bad(market([quote().t(T).sym("NRTH").last(185.09)])), r#"quote(): last is decimal text ("185.09"), not a number"#);
    assert_eq!(bad(market([note().t(T).text(3)])), "note(): text is text, not a number");
    assert_eq!(bad(market([trade().t(T).sym("NRTH").price("1e3").qty("1")])), r#"trade(): price is decimal text ("185.09"), not "1e3""#);
    for text in ["+1", "1,000", "1.", ".5", ""] {
        assert!(bad(market([book([bid().price(text).qty("1")]).t(T).sym("NRTH")])).contains("bid(): price is decimal text"), "{text:?}");
    }
    assert_eq!(bad(market([note().t("yesterday").text("x")])), r#"note(): t is an ISO time ("2026-10-01T13:30:00.000Z"), not "yesterday""#);
    for t in ["2026-10-01T13:30Z", "2026-10-01T13:30:00+02:00", "2026-10-01T13:30:00.5-05:00"] {
        assert!(declare(market([note().t(t).text("x")])).is_ok(), "{t}");
    }
    for t in ["2026-10-01", "2026-10-01T13:30", "2026-10-01T13:30:00.Z", "2026-10-01 13:30:00Z"] {
        assert!(bad(market([note().t(t).text("x")])).contains("t is an ISO time"), "{t}");
    }
    assert!(bad(market([quote().t(T).sym("NRTH").attr("price", "1")])).contains("quote(): price is not read (t, sym, last, bid, ask, vol)"));
    assert_eq!(bad(market([trade().t(T).sym("NRTH").qty("1")])), "trade() needs price");
    assert!(bad(market([bid().price("1").qty("1")])).contains("bid() stands in book(), not in market()"));
    assert!(bad(market([account([quote().t(T).sym("NRTH")]).t(T).venue("sim").mode("sim")])).contains("quote() stands in market(), not in account()"));
    assert!(bad(market([]).attr("venue", "sim")).contains("market(): venue is not read"));
}
